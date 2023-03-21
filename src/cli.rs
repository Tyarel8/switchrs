use std::path::PathBuf;

use clap::Parser;

use crate::utils::SwitchCommand;

#[derive(Parser)]
pub struct Cli {
    /// Command to run
    #[arg(value_enum)]
    pub command: SwitchCommand,
    /// Device or group to run the command on
    pub device: String,
    #[arg(short = 'p', long)]
    /// Optional path to the devices.toml file, if not present
    /// searches in the same directory as the executable
    pub path: Option<PathBuf>,
}
