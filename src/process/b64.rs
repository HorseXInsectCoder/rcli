use crate::cli::Base64Format;
use base64::{
    engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
    Engine as _,
};
use std::io::Read;

pub fn process_encode(reader: &mut dyn Read, format: Base64Format) -> anyhow::Result<String> {
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;

    let encoded = match format {
        Base64Format::Standard => STANDARD.encode(&buf),
        Base64Format::UrlSafe => URL_SAFE_NO_PAD.encode(&buf),
    };
    Ok(encoded)
}

pub fn process_decode(reader: &mut dyn Read, format: Base64Format) -> anyhow::Result<String> {
    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;

    let buf = buf.trim(); // 文本最后可能会有换行字符，处理掉

    let decoded = match format {
        Base64Format::Standard => STANDARD.decode(buf)?,
        Base64Format::UrlSafe => URL_SAFE_NO_PAD.decode(buf)?,
    };

    // // TODO: decoded data might not be string, but for this example, we assume it is
    // let decoded = String::from_utf8(decoded)?;
    // println!("decoded: {}", decoded);

    // 直接返回 Vec<u8>，不用再转成 String 了，在main转
    Ok(String::from_utf8(decoded)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_reader;

    #[test]
    fn test_process_encode() -> anyhow::Result<()> {
        let input = "Cargo.toml";
        let format: Base64Format = Base64Format::Standard;
        let mut reader = get_reader(input)?;
        assert!(process_encode(&mut reader, format).is_ok());

        Ok(())
    }

    #[test]
    fn test_process_decode() -> anyhow::Result<()> {
        // rcli/fixtrues 目录存放测试数据
        let input = "fixtures/b64.txt";
        let format = Base64Format::UrlSafe;
        let mut reader = get_reader(input)?;
        assert!(process_decode(&mut reader, format).is_ok());

        Ok(())
    }
}
