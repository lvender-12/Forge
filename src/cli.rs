use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "forge",
    version,
    about = "Manage processes supervised by Forge"
)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Ignite {
        /// Name assigned to the process with mold
        name: String,
    },
    /// Register a process and start it immediately
    Mold {
        /// Name to assign to the process
        name: String,
        #[arg(value_name = "exec", value_parser = clap::builder::PossibleValuesParser::new(["exec"]))]
        _exec: String,
        /// Command to run, including its arguments
        command: String,
    },
    Quench {
        /// Name of the process to stop
        name: String,
    },
    /// Show all managed processes
    Anvils,
    Smelt {
        /// Name of the process to stop and remove
        name: String,
    },
}

pub fn parse() -> Command {
    Args::parse().command
}
