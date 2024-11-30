// #![allow(unused)]

use clap::Parser;
use devices::{Device, Tuya, TuyaResult};
use utils::{get_devices_path, SwitchCommand};

mod cli;
mod devices;
mod utils;

fn main() {
    let cli = cli::Cli::parse();

    let Ok(tuya) = Tuya::from_file(cli.path.unwrap_or_else(|| get_devices_path("devices.toml")))
    else {
        println!("`devices.toml` not found | invalid `devices.toml` file");
        std::process::exit(1);
    };

    // let mut args: Vec<String> = std::env::args().skip(1).collect();

    // if args.len() != 2 {
    //     println!("Usage: switchrs <on|off|open|close|stop|status> <device name|group name>");
    //     std::process::exit(1);
    // }

    // let Ok(command) = args.remove(0).parse() else {
    //     println!("Invalid Command");
    //     std::process::exit(1)
    // };
    // let device_arg = args.remove(0).to_lowercase();
    let command = cli.command;
    let device_arg = cli.device.to_lowercase();

    // find if the argument is a group or a device
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
    // let group_name: Option<&str> = if input_devices.contains(&device_arg.as_str()) {
    //     None
    // } else {
    //     Some(&device_arg)
    // };

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

    // if found_devices.len() > 1 {
    //     let fdevice = &found_devices.first().unwrap().product_name.to_lowercase();
    //     if !found_devices
    //         .iter()
    //         .skip(1)
    //         .all(|x| &x.product_name.to_lowercase() == fdevice)
    //     {
    //         println!(
    //             "All devices must be of the same type in group `{}`",
    //             group_name.unwrap()
    //         );
    //         std::process::exit(1)
    //     }
    // }

    // Get delay and batch size if the argument is a group
    let (delay, batch) = tuya
        .groups
        .iter()
        .find_map(|x| {
            if x.group_name.to_lowercase() == device_arg {
                Some((x.delay.unwrap_or(0), x.batch.unwrap_or(1)))
            } else {
                None
            }
        })
        .unwrap_or((0, 1));

    // Iterate over the devies and execute the command
    for (i, device) in found_devices.iter().enumerate() {
        let mut retries = 3;
        while retries > 0 {
            let result = device.execute_command(&command, &tuya.api_secret, retries);

            match result {
                TuyaResult::Retry => {
                    retries -= 1;
                    println!("Retrying ({})...", device.name);
                    continue;
                }
                TuyaResult::Failure => println!("Failed to execute command on {}", device.name),
                TuyaResult::Success(result) => println!("{} status: {}", device.name, result),
                TuyaResult::EmptySuccess => println!("{} -> {}", device.name, command),
            }

            break;
        }

        // Sleep for the delay when batch size is reached
        if (i + 1) as u64 % batch == 0
            && !matches!(command, SwitchCommand::Status | SwitchCommand::Stop)
            && i != found_devices.len() - 1
        {
            std::thread::sleep(std::time::Duration::from_millis(delay));
        }
    }
}
