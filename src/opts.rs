use std::path::Path;

use clap::Parser;

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
}

#[derive(Debug, Parser)]
pub struct CsvOpts {
    // 对 input 做合法性检查，可以写自定义函数或者使用 clap 自带的
    #[arg(short, long, value_parser = verify_input_file)]
    pub input: String,

    // default_value 的展开是："output.json".into()，因为 "output.json" 是一个 &str，而 output 要求一个 String
    #[arg(short, long, default_value = "output.json")]
    pub output: String,

    #[arg(short, long, default_value_t = ',')]
    pub delimiter: char,

    // 注意这种h开头的字母就不能用 short 了，因为每个cli命令都会默认有一个 -h 参数，会重叠
    #[arg(long, default_value_t = true)]
    // default_value_t 就是直接传一个字面量，即不会经过 from 转换
    pub header: bool,
}

fn verify_input_file(filename: &str) -> Result<String, String> {
    if Path::new(filename).exists() {
        Ok(filename.into())
    } else {
        Err("File does not exists".into())
    }
}
