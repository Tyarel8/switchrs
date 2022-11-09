use std::{fmt::Display, path::PathBuf, str::FromStr};

use colored::Colorize;
use rust_tuyapi::{mesparse::Message, Payload};
use serde_json::Value;

use crate::devices::DeviceType;

pub fn get_devices_path() -> PathBuf {
    if cfg!(debug_assertions) {
        PathBuf::from("devices.toml")
    } else {
        let exe_path = std::env::current_exe().unwrap();
        exe_path.parent().unwrap().join("devices.toml")
    }
}

pub fn validate_device_command(device: &DeviceType, command: &SwitchCommand) -> bool {
    match device {
        DeviceType::Switch => matches!(
            command,
            SwitchCommand::On | SwitchCommand::Off | SwitchCommand::Status
        ),

        DeviceType::Blind => matches!(
            command,
            SwitchCommand::Open
                | SwitchCommand::Close
                | SwitchCommand::Stop
                | SwitchCommand::Status,
        ),
    }
}

#[derive(Debug, Clone)]
pub enum SwitchCommand {
    On,
    Off,
    Open,
    Close,
    Stop,
    Status,
}

impl FromStr for SwitchCommand {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "on" => Ok(Self::On),
            "off" => Ok(Self::Off),
            "open" => Ok(Self::Open),
            "close" => Ok(Self::Close),
            "stop" => Ok(Self::Stop),
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
            Self::Open => write!(f, "{}", "Open".green()),
            Self::Close => write!(f, "{}", "Close".red()),
            Self::Stop => write!(f, "{}", "Stop".yellow()),
            Self::Status => write!(f, "Status"),
        }
    }
}

pub fn parse_message(message: &Message) -> Option<SwitchCommand> {
    match &message.payload {
        Payload::Struct(s) => {
            if let Some(dps) = &s.dps {
                if let Some(value) = dps.get("1") {
                    parse_value(value)
                } else {
                    None
                }
            } else {
                None
            }
        }
        Payload::String(s) => {
            let json: Value = serde_json::from_str(s.as_str()).unwrap_or_else(|_| {
                println!("Version 3.4 of the Tuya API is not supported");
                std::process::exit(1);
            });
            let value = &json["dps"]["1"];
            parse_value(value)
        }
    }
}

fn parse_value(value: &Value) -> Option<SwitchCommand> {
    if let Some(b) = value.as_bool() {
        if b {
            Some(SwitchCommand::On)
        } else {
            Some(SwitchCommand::Off)
        }
    } else {
        match value.as_str() {
            Some("1") => Some(SwitchCommand::Open),
            Some("2") => Some(SwitchCommand::Close),
            Some("3") => Some(SwitchCommand::Stop),
            _ => None,
        }
    }
}
