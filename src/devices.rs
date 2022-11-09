use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
    net::IpAddr,
    path::PathBuf,
    str::FromStr,
    time::SystemTime,
};

use rust_tuyapi::{mesparse::Message, tuyadevice::TuyaDevice, Payload, PayloadStruct};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::utils::{parse_message, validate_device_command, SwitchCommand};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tuya {
    // pub api_key: String,
    pub api_secret: String,
    pub devices: Vec<Device>,
    pub groups: Vec<Group>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    pub name: String,
    pub id: String,
    pub key: String,
    pub sname: Option<String>,
    pub ip: String,
    #[serde(rename = "product_name")]
    pub product_name: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    #[serde(rename = "group_name")]
    pub group_name: String,
    pub devices: Vec<String>,
}

pub enum DeviceType {
    Switch,
    Blind,
}

impl DeviceType {
    fn from_device(s: &Device) -> Result<Self, String> {
        match s.product_name.to_lowercase().as_str() {
            "smart socket" => Ok(Self::Switch),
            "curtain switch" => Ok(Self::Blind),
            _ => Err(format!("Invalid device type: {}", s.product_name)),
        }
    }
}

impl Display for DeviceType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Switch => write!(f, "Switch"),
            Self::Blind => write!(f, "Blind"),
        }
    }
}

impl Tuya {
    pub fn from_file(path: PathBuf) -> Result<Tuya, Box<dyn std::error::Error>> {
        let file = std::fs::read_to_string(path)?;
        let tuya = toml::from_str(file.as_str())?;
        Ok(tuya)
    }
}

impl Device {
    pub fn execute_command(
        &self,
        command: &SwitchCommand,
        acces_token: &str,
    ) -> Option<SwitchCommand> {
        let device_type = DeviceType::from_device(self).unwrap_or_else(|e| {
            println!("{e}");
            std::process::exit(1)
        });

        if !validate_device_command(&device_type, command) {
            println!(
                "Invalid command `{}` for device type `{}`",
                command, device_type
            );
            std::process::exit(1);
        }
        // The dps value is device specific, this socket turns on with key "1"
        let dps = match command {
            SwitchCommand::Status => None,
            _ => {
                let mut dps = HashMap::new();
                dps.insert(
                    "1".to_string(),
                    match command {
                        SwitchCommand::On | SwitchCommand::Off => json!(match command {
                            SwitchCommand::On => true,
                            SwitchCommand::Off => false,
                            _ => unreachable!(),
                        }),
                        _ => json!(match command {
                            SwitchCommand::Open => "1",
                            SwitchCommand::Close => "2",
                            SwitchCommand::Stop => "3",
                            _ => unreachable!(),
                        }),
                    },
                );
                Some(dps)
            }
        };

        let current_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32;

        // Create the payload to be sent, this will be serialized to the JSON format
        let payload = Payload::Struct(PayloadStruct {
            dev_id: self.id.to_string(),
            gw_id: Some(acces_token.to_string()),
            uid: None,
            t: Some(current_time),
            dp_id: None,
            dps,
        });
        // Create a TuyaDevice, this is the type used to set/get status to/from a Tuya compatible
        // device.
        let tuya_device = TuyaDevice::create(
            "ver3.3",
            Some(&self.key),
            IpAddr::from_str(&self.ip).unwrap(),
        )
        .unwrap();

        // Set the payload state on the Tuya device, an error here will contain
        // the error message received from the device.
        let res: Option<Vec<Message>> = match command {
            SwitchCommand::Status => Some(tuya_device.get(payload, 0).unwrap_or_else(|x| {
                handle_tuya_err(x);
                std::process::exit(1);
            })),
            _ => {
                tuya_device.set(payload, 0).unwrap_or_else(|x| {
                    handle_tuya_err(x);
                    std::process::exit(1);
                });
                None
            }
        };

        if let Some(m) = res {
            parse_message(&m[0])
        } else {
            println!("Error: Invalid status response");
            std::process::exit(1);
        }
    }
}

fn handle_tuya_err(err: rust_tuyapi::error::ErrorKind) {
    if let rust_tuyapi::error::ErrorKind::TcpError(x) = err {
        match x.kind() {
            std::io::ErrorKind::TimedOut => {
                println!("Timeout Error: device not responding");
            }
            std::io::ErrorKind::ConnectionReset => {
                println!("Connection Error: Close Smart Life App");
            }
            _ => {
                println!("Error: {}", x);
            }
        };
    }
}
