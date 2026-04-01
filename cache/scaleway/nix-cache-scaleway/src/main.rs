mod cli;
mod commands;
mod config;
mod contract;
mod error;
#[cfg(test)]
mod test_support;

use std::process::ExitCode;

use clap::Parser;
use cli::{Cli, Command};
use config::{BootstrapConfig, PublishConfig, VerifyConfig};

fn main() -> ExitCode {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::Bootstrap(args) => {
            BootstrapConfig::try_from(args).and_then(|config| commands::bootstrap::run(&config))
        }
        Command::Publish(args) => {
            PublishConfig::try_from(args).and_then(|config| commands::publish::run(&config))
        }
        Command::Verify(args) => {
            VerifyConfig::try_from(args).and_then(|config| commands::verify::run(&config))
        }
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}
