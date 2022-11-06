use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tuya {
    // pub api_key: String,
    pub api_secret: String,
    pub devices: Vec<Device>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    pub name: String,
    pub id: String,
    pub key: String,
    pub sname: Option<String>,
    pub ip: String,
}

impl Tuya {
    pub fn from_file(path: PathBuf) -> Result<Tuya, Box<dyn std::error::Error>> {
        let file = std::fs::read_to_string(path)?;
        let tuya = toml::from_str(file.as_str())?;
        Ok(tuya)
    }
}
