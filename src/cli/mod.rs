mod base64;
mod csv;
mod genpass_opts;
mod http;
mod text;

use clap::Parser;
use enum_dispatch::enum_dispatch;
use std::path::{Path, PathBuf};

// 这里用 self::csv 的原因是，如果不用 self 的话，会与 Cargo.toml 里的 csv crate 冲突
pub use self::{base64::*, csv::*, genpass_opts::*, http::*, text::*};

#[derive(Debug, Parser)]
#[command(name = "cli", version, author, about, long_about = None)] // 这些信息会自动从 Cargo.toml 读取
pub struct Opts {
    #[command(subcommand)]
    pub cmd: Subcommand,
}

#[derive(Debug, Parser)]
#[enum_dispatch(CmdExector)]
pub enum Subcommand {
    // name 可以不指定，默认就是转成小写
    #[command(name = "csv", about = "Convert CSV to other format")]
    Csv(CsvOpts),

    #[command(name = "genpass", about = "generate a random password")]
    GenPass(GenPassOpts),

    #[command(subcommand, about = "Base encode/decode")]
    Base64(Base64SubCommand),

    #[command(subcommand, about = "Text Sign/verify")]
    Text(TextSubCommand),

    #[command(subcommand, about = "HTTP Server")]
    Http(HttpSubCommand),
}

// 可以删除
// impl CmdExector for Subcommand {
//     async fn execute(self) -> anyhow::Result<()> {
//         match self {
//             Subcommand::Csv(opts) => opts.execute().await,
//             Subcommand::GenPass(opts) => opts.execute().await,
//             Subcommand::Base64(cmd) => cmd.execute().await,
//             Subcommand::Text(cmd) => cmd.execute().await,
//             Subcommand::Http(cmd) => cmd.execute().await,
//         };
//         Ok(())
//     }
// }

// 把方法从 csv 模块提到这里，让 mod 下面所有模块都可以使用
fn verify_file(filename: &str) -> Result<String, &'static str> {
    // if input is "-" or file exists
    if filename == "-" || Path::new(filename).exists() {
        Ok(filename.into())
    } else {
        Err("File does not exists")
    }
}

fn verify_path(path: &str) -> Result<PathBuf, &'static str> {
    let p = Path::new(path);
    if p.exists() && p.is_dir() {
        Ok(path.into())
    } else {
        Err("Path is not exists or is not a directory")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_input_file() {
        assert_eq!(verify_file("-"), Ok("-".into()));
        assert_eq!(verify_file("*"), Err("File does not exists"));
        assert_eq!(verify_file("Cargo.toml"), Ok("Cargo.toml".into()));
        assert_eq!(verify_file("not-exists"), Err("File does not exists"));
    }
}
