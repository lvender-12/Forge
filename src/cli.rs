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
        #[arg(help = "Name assigned to the process with mold")]
        name: String,
    },
    #[command(about = "Register a process and start it immediately")]
    Mold {
        #[arg(help = "Name to assign to the process")]
        name: String,
        #[arg(value_name = "exec", value_parser = clap::builder::PossibleValuesParser::new(["exec"]))]
        _exec: String,
        #[arg(help = "Command to run, including its arguments")]
        command: String,
    },
    Quench {
        #[arg(help = "Name of the process to stop")]
        name: String,
    },
    #[command(about = "Show all managed processes")]
    Anvils,
    Smelt {
        #[arg(help = "Name of the process to stop and remove")]
        name: String,
    },
}

pub fn parse() -> Command {
    Args::parse().command
}
