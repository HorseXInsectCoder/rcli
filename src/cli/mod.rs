mod base64;
mod csv;
mod genpass_opts;

use std::path::Path;

use self::{csv::CsvOpts, genpass_opts::GenPassOpts};
use clap::Parser;

// 这里用 self::csv 的原因是，如果不用 self 的话，会与 Cargo.toml 里的 csv crate 冲突
pub use self::{
    base64::{Base64Format, Base64SubCommand},
    csv::OutputFormat,
};

#[derive(Debug, Parser)]
#[command(name = "cli", version, author, about, long_about = None)] // 这些信息会自动从 Cargo.toml 读取
pub struct Opts {
    #[command(subcommand)]
    pub cmd: Subcommand,
}

#[derive(Debug, Parser)]
pub enum Subcommand {
    // name 可以不指定，默认就是转成小写
    #[command(name = "csv", about = "Convert CSV to other format")]
    Csv(CsvOpts),

    #[command(name = "genpass", about = "generate a random password")]
    GenPass(GenPassOpts),

    #[command(subcommand)]
    Base64(Base64SubCommand),
}

// 把方法从 csv 模块提到这里，让 mod 下面所有模块都可以使用
fn verify_input_file(filename: &str) -> Result<String, &'static str> {
    // if input is "-" or file exists
    if filename == "-" || Path::new(filename).exists() {
        Ok(filename.into())
    } else {
        Err("File does not exists")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_input_file() {
        assert_eq!(verify_input_file("-"), Ok("-".into()));
        assert_eq!(verify_input_file("*"), Err("File does not exists"));
        assert_eq!(verify_input_file("Cargo.toml"), Ok("Cargo.toml".into()));
        assert_eq!(verify_input_file("not-exists"), Err("File does not exists"));
    }
}
