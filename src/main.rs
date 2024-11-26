//@Author: IndritZ
//@Description: Windows TrustedInstaller elevation toolkit


use clap::Parser;
use tracing::info;

mod core;
mod domain;
mod utils;

use core::elevation::trusted_installer::TrustedInstallerElevation;
use utils::{config::Config, security::SecurityContext};

#[derive(Parser)]
#[clap(
    name = "ElevateKit",
    about = "Windows TrustedInstaller elevation toolkit",
    version
)]
struct Cli {
    /// command and arguments to execute (defaults to "powershell.exe" if not provided)
    #[clap(trailing_var_arg = true)]
    args: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting ElevateKit");

    let cli = Cli::parse();
    let _config = Config::load()?;
    let security_context = SecurityContext::new()?;

    //  default to powershell.exe if no command is provided
    let (command, args) = if cli.args.is_empty() {
        ("powershell.exe".to_string(), Vec::new())
    } else {
        (cli.args[0].clone(), cli.args[1..].to_vec())
    };

    info!("Running command: {} {:?}", command, args);

    // execute with TrustedInstaller privileges
    let ti = TrustedInstallerElevation::new(&security_context);
    ti.execute(&command, &args)?;

    info!("Operation completed successfully");
    Ok(())
}