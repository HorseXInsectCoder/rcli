use std::{fs, io::Read, path::Path, vec};

use anyhow::Result;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;

use crate::{get_reader, TextSignFormat};

use super::process_genpass;

// Blake3 和 Ed25519 都需要一个 sign 方法，所以可以抽取成 trait
pub trait TextSign {
    // data 不能用 &str 或者 &[u8]，因为这表示必须依赖 get_reader 或者必须把完整的data传给sigh
    // 当需要把 trait 放到一个更公开的位置时，会耦合
    // 而提供 reader 的话，会更加灵活。不管给 sign 传什么东西，只要它能被读取，那么就可以不断地从里面读取到数据
    // &[u8] implements Read, so we can test with &[u8] instead of File
    /// Sign the data from the reader and return the signature
    fn sign(&self, reader: &mut dyn Read) -> Result<Vec<u8>>;
}

pub trait TextVerify {
    // 在写业务代码的时候，不用考虑是动态还是静态分发。最重要是考虑 io 效率
    // 静态分发的不同写法
    /// Verify the data from reader with the signature
    fn verify(&self, reader: impl Read, sig: &[u8]) -> Result<bool>;
    // fn verify<R: Read>(&self, reader: R, sig: &[u8]) -> Result<bool>;
}

pub trait KeyLoader {
    // 返回固定长度的数据结构， str, [u8]这些是没有固定长度
    fn load(path: impl AsRef<Path>) -> Result<Self>
    where
        Self: Sized;
}

pub trait KeyGenerator {
    // Blake3 生成的是一个key，Ed25519生成出来的是一对key(signer key 和 verify key)，所以要再套一层Vec
    fn generate() -> Result<Vec<Vec<u8>>>;
}

pub struct Blake3 {
    key: [u8; 32],
}

pub struct Ed25519Signer {
    key: SigningKey,
}

pub struct Ed25519Verifier {
    key: VerifyingKey,
}

pub fn process_text_sign(input: &str, key: &str, format: TextSignFormat) -> anyhow::Result<String> {
    let mut reader = get_reader(input)?;
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;
    let signed = match format {
        TextSignFormat::Blake3 => {
            // let key = fs::read(key)?;
            // let key = &key[..32]; // 只要前面32位，防止后面的换行
            // let key = key.try_into()?; // 转成 [u8; 32]
            // let signer = Blake3 { key };

            let signer = Blake3::load(key)?;
            signer.sign(&mut reader)?
        }
        TextSignFormat::Ed25519 => {
            let signer = Ed25519Signer::load(key)?;
            signer.sign(&mut reader)?
        }
    };

    let signed = URL_SAFE_NO_PAD.encode(signed);

    Ok(signed)
}

pub fn process_text_verify(
    input: &str,
    key: &str,
    format: TextSignFormat,
    sig: &str,
) -> anyhow::Result<bool> {
    let mut reader = get_reader(input)?;

    let sig = URL_SAFE_NO_PAD.decode(sig)?;

    let verified = match format {
        TextSignFormat::Blake3 => {
            let verifier: Blake3 = Blake3::load(key)?;
            verifier.verify(&mut reader, &sig)?
        }
        TextSignFormat::Ed25519 => {
            let verifier = Ed25519Verifier::load(key)?;
            verifier.verify(&mut reader, &sig)?
        }
    };

    Ok(verified)
}

pub fn process_text_generate(format: TextSignFormat) -> Result<Vec<Vec<u8>>> {
    match format {
        TextSignFormat::Blake3 => Blake3::generate(),
        TextSignFormat::Ed25519 => Ed25519Signer::generate(),
    }
}

impl TextSign for Blake3 {
    fn sign(&self, reader: &mut dyn Read) -> Result<Vec<u8>> {
        // TODO: improve perf by reading in chunks
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        // println!("buf1: {:?}", String::from_utf8_lossy(&buf));
        let ret = blake3::keyed_hash(&self.key, &buf);
        Ok(ret.as_bytes().to_vec())
    }
}

impl TextVerify for Blake3 {
    fn verify(&self, mut reader: impl Read, sig: &[u8]) -> Result<bool> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;

        // 常见错误： blake3::hash(&buf).as_bytes(); 执行完blake3::hash(&buf)后由于没有东西指向它，会被 free 掉
        // 解决办法：其实报错的时候编译器会给出解决办法，就是再加一个中间变量
        // let hash = blake3::keyed_hash(&self.key, &buf);
        // let hash = hash.as_bytes();

        // Ok(hash == sig)

        let ret = blake3::keyed_hash(&self.key, &buf);
        Ok(ret.as_bytes() == sig)
    }
}

impl TextSign for Ed25519Signer {
    fn sign(&self, reader: &mut dyn Read) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let sig = self.key.sign(&buf);
        Ok(sig.to_bytes().to_vec())
    }
}

impl TextVerify for Ed25519Verifier {
    fn verify(&self, mut reader: impl Read, sig: &[u8]) -> Result<bool> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;

        // from_bytes 要求 SignatureBytes
        // type SignatureBytes = [u8; Signature::BYTE_SIZE];
        // 其实就是要把 &[u8] 的 sig 转为 [u8; Signature::BYTE_SIZE]; slice 可以被转换成数组
        let sig = Signature::from_bytes(sig.try_into()?);
        let ret = self.key.verify(&buf, &sig).is_ok();
        Ok(ret)
    }
}

impl Blake3 {
    // 直接提供数据
    pub fn new(key: [u8; 32]) -> Blake3 {
        Self { key }
    }

    // key 提供的是 [u8] 引用
    pub fn try_new(key: &[u8]) -> Result<Self> {
        let key = &key[..32];
        let key = key.try_into()?;
        let signer = Blake3::new(key);
        Ok(signer)
    }
}

impl Ed25519Signer {
    // 直接提供数据
    pub fn new(key: SigningKey) -> Self {
        Self { key }
    }

    // key 提供的是 [u8] 引用
    pub fn try_new(key: &[u8]) -> Result<Self> {
        let key = SigningKey::from_bytes(key.try_into()?);
        let signer = Ed25519Signer::new(key);
        Ok(signer)
    }
}

impl Ed25519Verifier {
    // 直接提供数据
    pub fn new(key: VerifyingKey) -> Self {
        Self { key }
    }

    // key 提供的是 [u8] 引用
    pub fn try_new(key: &[u8]) -> Result<Self> {
        let key = VerifyingKey::from_bytes(key.try_into()?)?;
        let verifyer = Ed25519Verifier::new(key);
        Ok(verifyer)
    }
}

impl KeyLoader for Blake3 {
    fn load(path: impl AsRef<Path>) -> Result<Self>
    where
        Self: Sized,
    {
        let key = fs::read(path)?;
        Self::try_new(&key)
    }
}

impl KeyLoader for Ed25519Signer {
    fn load(path: impl AsRef<Path>) -> Result<Self>
    where
        Self: Sized,
    {
        let key = fs::read(path)?;
        Self::try_new(&key)
    }
}

impl KeyLoader for Ed25519Verifier {
    fn load(path: impl AsRef<Path>) -> Result<Self>
    where
        Self: Sized,
    {
        let key = fs::read(path)?;
        Self::try_new(&key)
    }
}

impl KeyGenerator for Blake3 {
    fn generate() -> Result<Vec<Vec<u8>>> {
        let key = process_genpass(32, true, true, true, true)?;
        let key = key.as_bytes().to_vec();
        Ok(vec![key])
    }
}

impl KeyGenerator for Ed25519Signer {
    fn generate() -> Result<Vec<Vec<u8>>> {
        let mut csprng = OsRng;
        // generate 只有在 rand_core feature 才能使用，这是Rust常见的问题，使用这些crate得看清楚文档
        let signing_key = SigningKey::generate(&mut csprng);

        let public_key = signing_key.verifying_key().to_bytes().to_vec();

        let signing_key = signing_key.as_bytes().to_vec();

        Ok(vec![signing_key, public_key])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blake3_sign_verify() -> Result<()> {
        let blake3 = Blake3::load("fixtures/blake3.txt")?;

        let data = b"hello world";
        let sig = blake3.sign(&mut &data[..]).unwrap();
        println!("sig: {}", URL_SAFE_NO_PAD.encode(&sig)); // 编码后更容易读
        assert!(blake3.verify(&mut &data[..], &sig).unwrap());

        Ok(())
    }

    #[test]
    fn test_ed25519_sign_verify() -> Result<()> {
        let sk = Ed25519Signer::load("fixtures/ed25519.sk")?;
        let pk = Ed25519Verifier::load("fixtures/ed25519.pk")?;

        let data = b"hello world";
        let sig = sk.sign(&mut &data[..])?;
        assert!(pk.verify(&data[..], &sig)?);

        Ok(())
    }
}
