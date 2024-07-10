use serde::{Deserialize, Serialize};

use csv::Reader;

use std::fs;

use anyhow::Result;

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

pub fn process_csv(input: &str, output: &str) -> Result<()> {
    let mut reader = Reader::from_path(input)?;

    let mut ret = Vec::with_capacity(128);

    for result in reader.deserialize() {
        let record: Player = result?;
        // println!("{:?}", record);
        ret.push(record);
    }
    let json = serde_json::to_string_pretty(&ret)?;
    fs::write(output, json)?;

    Ok(())
}
