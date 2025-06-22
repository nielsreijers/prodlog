use std::{path::PathBuf, sync::OnceLock};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)] // Add metadata
pub struct CliArgs {
    #[arg(
        long,
        value_name = "DIR",
        default_value = ".local/share/prodlog",
        help = "Directory to store production logs"
    )]
    pub dir: PathBuf,

    #[arg(long, value_name = "PORT", default_value = "5000", help = "Port to run the UI on")]
    pub port: u16,

    #[arg(
        long,
        value_name = "IMPORT",
        default_value = None,
        help = "Import a prodlog json or sqlite file"
    )]
    pub import: Option<String>,

    #[arg(
        long,
        value_name = "CMD",
        default_value = "/bin/bash",
        help = "Initial command to run. Defaults to starting bash, but you can use something like 'ssh <host>' to go to a remote directly."
    )]
    pub cmd: String,

    #[arg(
        long,
        value_name = "HEX_COLOUR",
        default_value = "#FFFFFF",
        help = "Background colour for the UI."
    )]
    pub ui_background: String,
}

static CONFIG: OnceLock<CliArgs> = OnceLock::new();

fn init_config() -> CliArgs {
    CliArgs::parse()
}

pub fn get_config() -> &'static CliArgs {
    CONFIG.get_or_init(init_config)
}