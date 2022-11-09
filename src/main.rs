// #![allow(dead_code, unused_variables)]

use devices::{Device, Tuya};
use utils::{get_devices_path, SwitchCommand};

pub mod devices;
mod utils;
fn main() {
    let tuya = Tuya::from_file(get_devices_path())
        .expect("`devices.toml` not found in the same directory as the executable | invalid `devices.toml` file");

    let mut args: Vec<String> = std::env::args().skip(1).collect();

    if args.len() != 2 {
        println!("Usage: switchrs <on|off|open|close|stop|status> <device name|group name>");
        std::process::exit(1);
    }

    let command: SwitchCommand = args.remove(0).parse().unwrap_or_else(|_| {
        println!("Invalid Command");
        std::process::exit(1)
    });
    let device_arg = args.remove(0).to_lowercase();

    let input_devices: Vec<&str> = tuya
        .groups
        .iter()
        .find_map(|x| {
            if x.group_name.to_lowercase() == device_arg {
                Some(x.devices.iter().map(|x| x.as_str()).collect())
            } else {
                None
            }
        })
        .unwrap_or_else(|| vec![device_arg.as_str()]);

    // Get the name of the group if that was the argument
    let group_name: Option<&str> = if input_devices.contains(&device_arg.as_str()) {
        None
    } else {
        Some(&device_arg)
    };

    let mut found_devices: Vec<&Device> = vec![];
    for idevice in input_devices {
        let devi = tuya
            .devices
            .iter()
            .find(|device| {
                device.name.to_lowercase() == idevice
                    || if let Some(sname) = &device.sname {
                        sname.to_lowercase().as_str() == idevice
                    } else {
                        false
                    }
            })
            .unwrap_or_else(|| {
                println!("`{}` device not found", idevice);
                std::process::exit(1)
            });
        found_devices.push(devi);
    }

    if found_devices.len() > 1 {
        let fdevice = &found_devices.first().unwrap().product_name.to_lowercase();
        if !found_devices
            .iter()
            .skip(1)
            .all(|x| &x.product_name.to_lowercase() == fdevice)
        {
            println!(
                "All devices must be of the same type in group `{}`",
                group_name.unwrap()
            );
            std::process::exit(1)
        }
    }

    for device in found_devices {
        let result = device.execute_command(&command, &tuya.api_secret);
        if let Some(result) = result {
            println!("{} status: {}", device.name, result);
        } else {
            println!("{} -> {}", device.name, command);
        }
    }
}
