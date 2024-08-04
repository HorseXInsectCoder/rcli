use rand::seq::SliceRandom;

use zxcvbn::zxcvbn;

const UPPER: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ"; // 大写的 I 和小写的 l 也不做为密码, O和0也一样
const LOWER: &[u8] = b"abcdefghijkmnopqrstuvwxyz";
const NUMBER: &[u8] = b"123456789";
const SYMBOL: &[u8] = b"!@#$%^&*_"; // 选不容易产生歧义的特殊字符

// 函数不要跟CLI传进来的数据结构绑定得太紧，所以这里不直接使用 GenPassOpts 来传参。可以方便以后拆出来单独使用
pub fn process_genpass(
    length: u8,
    upper: bool,
    lower: bool,
    number: bool,
    symbol: bool,
) -> anyhow::Result<String> {
    let mut rng = rand::thread_rng();

    let mut password = Vec::new();

    let mut chars: Vec<u8> = Vec::new();

    if upper {
        chars.extend_from_slice(UPPER);
        password.push(*UPPER.choose(&mut rng).expect("UPPER won't be empty"));
    }
    if lower {
        chars.extend_from_slice(LOWER);
        password.push(*LOWER.choose(&mut rng).expect("LOWER won't be empty"));
    }
    if number {
        chars.extend_from_slice(NUMBER);
        password.push(*NUMBER.choose(&mut rng).expect("NUMBER won't be empty"));
    }
    if symbol {
        chars.extend_from_slice(SYMBOL);
        password.push(*SYMBOL.choose(&mut rng).expect("SYMBOL won't be empty"));
    }

    for _ in 0..(length - password.len() as u8) {
        // 由于 choose 返回的是一个 Option，不会empty报错，所以这里可以直接使用 expect
        // 如果得到的数据是引用类型的话，要 clone，但这里是 u8，所以不用 clone
        let c = chars
            .choose(&mut rng)
            .expect("chars won't be empty in this context");
        password.push(*c)
    }

    password.shuffle(&mut rng);

    let password = String::from_utf8(password)?;
    println!("{}", password);

    let estimate = zxcvbn(&password, &[])?;

    // 用 eprintln 是为了输出到 std error，如果程序需要输出密码到文件，如 cargo run -- genpass > out.txt
    // 不会与 std out 的数据混合，即运行 cargo run -- genpass > out.txt，只会输出 Password strength
    // 如果用 println 的话，在输出到文件时，会把 println 的内容也输出到文件
    eprintln!("Password strength: {}", estimate.score()); // 16位的长度是4，4表示足够强了

    // Ok(String::from_utf8(password)?)
    Ok(password)
}
