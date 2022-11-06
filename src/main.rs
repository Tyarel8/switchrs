use devices::Tuya;
use utils::{execute_command, get_devices_path, SwitchCommand};

pub mod devices;
mod utils;
fn main() {
    let tuya = Tuya::from_file(get_devices_path())
        .expect("`devices.toml` not found in the same directory as the executable | invalid `devices.toml` file");

    let mut args: Vec<String> = std::env::args().skip(1).collect();

    if args.len() != 2 {
        println!("Usage: switchrs <device name> <on|off|status>");
        std::process::exit(1);
    }

    let device_name = args.remove(0).to_lowercase();
    let command: SwitchCommand = args.remove(0).parse().unwrap_or_else(|_| {
        println!("Invalid Command");
        std::process::exit(1)
    });

    let device = tuya
        .devices
        .iter()
        .find(|device| {
            device.name.to_lowercase() == device_name
                || if let Some(sname) = &device.sname {
                    sname.to_lowercase().as_str() == &device_name
                } else {
                    false
                }
        })
        .unwrap_or_else(|| {
            println!("Device not found");
            std::process::exit(1)
        });

    let result = execute_command(device, &command, &tuya.api_secret);
    if let Some(result) = result {
        match (&result, command) {
            (SwitchCommand::On, SwitchCommand::On) => println!("Switched {}", result),
            (SwitchCommand::Off, SwitchCommand::Off) => println!("Switched {}", result),
            (SwitchCommand::On, SwitchCommand::Status) => println!("Status: {}", result),
            (SwitchCommand::Off, SwitchCommand::Status) => println!("Status: {}", result),
            _ => println!("Unexpected result: {}", result),
        }
    } else {
        println!("Failed to execute command");
    }
}
