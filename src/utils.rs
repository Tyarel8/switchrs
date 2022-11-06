use std::{
    collections::HashMap, fmt::Display, net::IpAddr, path::PathBuf, str::FromStr, time::SystemTime,
};

use colorize::AnsiColor;
use rust_tuyapi::{mesparse::Message, tuyadevice::TuyaDevice, Payload, PayloadStruct};
use serde_json::{json, Value};

use crate::devices::Device;

pub fn get_devices_path() -> PathBuf {
    if cfg!(debug_assertions) {
        PathBuf::from("devices.toml")
    } else {
        let exe_path = std::env::current_exe().unwrap();
        exe_path.parent().unwrap().join("devices.toml")
    }
}

#[derive(Debug, Clone)]
pub enum SwitchCommand {
    On,
    Off,
    Status,
}

impl FromStr for SwitchCommand {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "on" => Ok(Self::On),
            "off" => Ok(Self::Off),
            "status" => Ok(Self::Status),
            _ => Err(format!("Invalid command: {}", s)),
        }
    }
}

impl Display for SwitchCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::On => write!(f, "{}", "On".green()),
            Self::Off => write!(f, "{}", "Off".red()),
            Self::Status => write!(f, "Status"),
        }
    }
}

// #[derive(Serialize, Debug)]
// struct Payload {
//     commands: Vec<Command>,
// }

// #[derive(Serialize, Debug)]
// struct Command {
//     code: String,
//     value: bool,
// }

pub fn execute_command(
    device: &Device,
    command: &SwitchCommand,
    acces_token: &str,
) -> Option<SwitchCommand> {
    // let url = format!(
    //     "https://openapi.tuyaeu.com/v1.0/iot-03/devices/{}/{}",
    //     device.id,
    //     match command {
    //         SwitchCommand::On | SwitchCommand::Off => "commands",
    //         SwitchCommand::Status => "status",
    //     }
    // );
    // let time = format!(
    //     "{}",
    //     SystemTime::now()
    //         .duration_since(SystemTime::UNIX_EPOCH)
    //         .unwrap()
    //         .as_millis()
    // );
    // let mut headers = HeaderMap::new();
    // let sign = headers.insert("client_id", client_id.parse().unwrap());
    // headers.insert("access_token", acces_token.parse().unwrap());
    // headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
    // headers.insert("mode", "cors".parse().unwrap());
    // headers.insert("sign_method", "HMAC-SHA256".parse().unwrap());
    // headers.insert("t", time.parse().unwrap());

    // let client = reqwest::blocking::Client::new();
    // match command {
    //     SwitchCommand::On | SwitchCommand::Off => {
    //         let body = Payload {
    //             commands: vec![Command {
    //                 code: "switch".to_string(),
    //                 value: match command {
    //                     SwitchCommand::On => true,
    //                     SwitchCommand::Off => false,
    //                     SwitchCommand::Status => return None,
    //                 },
    //             }],
    //         };

    //         let res = client
    //             .post(url)
    //             .headers(headers)
    //             .json(&body)
    //             .send()
    //             .unwrap();
    //         // println!("{:?}", res.text());
    //     }
    //     SwitchCommand::Status => {
    //         let res = client.get(url).headers(headers).send().unwrap();
    //     }
    // }

    // The dps value is device specific, this socket turns on with key "1"
    let mut dps = HashMap::new();
    dps.insert(
        "1".to_string(),
        json!(match command {
            SwitchCommand::On => true,
            _ => false,
        }),
    );

    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;

    // Create the payload to be sent, this will be serialized to the JSON format
    let payload = Payload::Struct(PayloadStruct {
        dev_id: device.id.to_string(),
        gw_id: Some(acces_token.to_string()),
        uid: None,
        t: Some(current_time),
        dp_id: None,
        dps: match command {
            SwitchCommand::Status => None,
            _ => Some(dps),
        },
    });
    // Create a TuyaDevice, this is the type used to set/get status to/from a Tuya compatible
    // device.
    let tuya_device = TuyaDevice::create(
        "ver3.3",
        Some(&device.key),
        IpAddr::from_str(&device.ip).unwrap(),
    )
    .unwrap();

    // Set the payload state on the Tuya device, an error here will contain
    // the error message received from the device.
    let mut res: Option<Vec<Message>> = None;
    match command {
        SwitchCommand::Status => {
            res = Some(tuya_device.get(payload, 0).unwrap_or_else(|_| {
                println!("Error: Close Smart Life App");
                std::process::exit(0);
            }))
        }
        _ => tuya_device.set(payload, 0).unwrap_or_else(|_| {
            println!("Error: Close Smart Life App");
            std::process::exit(0);
        }),
    };

    if let Some(m) = res {
        match &m[0].payload {
            Payload::Struct(s) => {
                if let Some(dps) = &s.dps {
                    if let Some(value) = dps.get("1") {
                        if value.as_bool().unwrap() {
                            return Some(SwitchCommand::On);
                        } else {
                            return Some(SwitchCommand::Off);
                        }
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            }
            Payload::String(s) => {
                let json: Value = serde_json::from_str(s).unwrap();
                if json["dps"]["1"].as_bool().unwrap() {
                    return Some(SwitchCommand::On);
                } else {
                    return Some(SwitchCommand::Off);
                }
            }
        }
    } else {
        Some(command.clone())
    }
}
