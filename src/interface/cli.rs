use clap::{self, Parser, Command};
use crate::infrastructure::error::TiResult;

pub struct CLI;

impl CLI {
    pub fn get_command() -> TiResult<String> {
        let matches = clap::Command::new("TrustedInstaller")
            .version("1.0")
            .author("IndritZ")
            .about("Runs commands with TrustedInstaller privileges")
            .arg(
                clap::Arg::new("command")
                    .help("Command to execute")
                    .num_args(1..)
                    .required(false)
            )
            .get_matches();

        let command = matches.get_many::<String>("command")
            .map(|values| values.map(|s| s.as_str()).collect::<Vec<&str>>().join(" "))
            .unwrap_or_else(|| "powershell.exe".to_string());

        Ok(command)
    }
} 