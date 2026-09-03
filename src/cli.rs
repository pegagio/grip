use crate::result::OutputMode;
use clap::{Parser, Subcommand, ValueEnum};
use std::ffi::OsString;

#[derive(Debug, Parser)]
#[command(name = "grip", version, about = "Safely manage per-user file mappings")]
pub struct Cli {
    #[arg(long, global = true, value_enum, default_value = "human")]
    pub output: OutputArg,
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputArg {
    Human,
    Json,
}
impl From<OutputArg> for OutputMode {
    fn from(value: OutputArg) -> Self {
        match value {
            OutputArg::Human => Self::Human,
            OutputArg::Json => Self::Json,
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Version,
    Validate,
}

pub fn requests_json(args: &[OsString]) -> bool {
    args.windows(2)
        .any(|w| w[0] == "--output" && w[1] == "json")
        || args.iter().any(|a| a == "--output=json")
}
