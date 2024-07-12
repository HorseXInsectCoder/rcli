use serde::{Deserialize, Serialize};

use csv::Reader;
use serde_json::Value;

use std::fs;

use anyhow::Result;

use crate::opts::OutputFormat;

// juventus.csv 的字段结构
#[derive(Debug, Deserialize, Serialize)]
struct Player {
    #[serde(rename = "Name")]
    name: String,

    #[serde(rename = "Position")]
    position: String,

    #[serde(rename = "DOB")]
    dob: String, // Date of Birth

    #[serde(rename = "Nationality")]
    nationality: String,

    #[serde(rename = "Kit Number")]
    kit: u8, // 球衣号
}

pub fn process_csv(input: &str, output: String, format: OutputFormat) -> Result<()> {
    let mut reader: Reader<fs::File> = Reader::from_path(input)?;

    let mut ret = Vec::with_capacity(128);

    let headers: csv::StringRecord = reader.headers()?.clone();

    for result in reader.records() {
        let record: csv::StringRecord = result?;
        // headers.iter() -> 使用 headers 的迭代器
        // record.iter() -> 使用 record 的迭代器
        // zip() -> 将两个迭代器合并为一个元组的迭代器 [(header, record), ..]
        // collect::<Value>() -> 将元组的迭代器转换为 JSON Value
        // 生成的是 Json Value，但是因为 Json Value 实现了 Deserialize, Serialize，所以最后是 Value
        let json_value = headers.iter().zip(record.iter()).collect::<Value>();
        // println!("{:?}", record);
        ret.push(json_value);
    }

    let content = match format {
        OutputFormat::Json => serde_json::to_string_pretty(&ret)?,
        OutputFormat::Yaml => serde_yaml::to_string(&ret)?,
    };

    fs::write(output, content)?;

    Ok(())
}
