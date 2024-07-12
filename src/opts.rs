use core::fmt;
use std::{path::Path, str::FromStr};

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

#[derive(Debug, Copy, Clone)]
pub enum OutputFormat {
    Json,
    Yaml,
}

#[derive(Debug, Parser)]
pub struct CsvOpts {
    // 对 input 做合法性检查，可以写自定义函数或者使用 clap 自带的
    #[arg(short, long, value_parser = verify_input_file)]
    pub input: String,

    // default_value 的展开是："output.json".into()，因为 "output.json" 是一个 &str，而 output 要求一个 String
    #[arg(short, long)]
    pub output: Option<String>,

    #[arg(short, long, default_value_t = ',')]
    pub delimiter: char,

    // 注意这种h开头的字母就不能用 short 了，因为每个cli命令都会默认有一个 -h 参数，会重叠
    #[arg(long, default_value_t = true)]
    // default_value_t 就是直接传一个字面量，即不会经过 from 转换
    pub header: bool,

    #[arg(long, default_value = "json", value_parser = parse_format)]
    pub format: OutputFormat,
}

fn verify_input_file(filename: &str) -> Result<String, String> {
    if Path::new(filename).exists() {
        Ok(filename.into())
    } else {
        Err("File does not exists".into())
    }
}

// anyhow::Error 可以转为 String 输出到命令行
fn parse_format(format: &str) -> Result<OutputFormat, anyhow::Error> {
    // 有 impl FromStr 后就不再需要使用这段了
    // match format.to_lowercase().as_str() {
    //     "json" => Ok(OutputFormat::Json),
    //     "yaml" => Ok(OutputFormat::Yaml),
    //     "toml" => Ok(OutputFormat::Toml),
    //     _ => Err("Invalid format")
    // }

    // parse 可以把 String 解析成其他的数据类型，前提是这个数据类型实现了 FromStr
    format.parse()
}

impl From<OutputFormat> for &'static str {
    fn from(format: OutputFormat) -> Self {
        match format {
            OutputFormat::Json => "json",
            OutputFormat::Yaml => "yaml",
        }
    }
}

impl FromStr for OutputFormat {
    type Err = anyhow::Error;

    fn from_str(format: &str) -> Result<Self, Self::Err> {
        match format.to_lowercase().as_str() {
            "json" => Ok(OutputFormat::Json),
            "yaml" => Ok(OutputFormat::Yaml),
            v => anyhow::bail!("Unsupported format: {}", v),
        }
    }
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Into::<&str>::into(*self))
    }
}
