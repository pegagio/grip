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
    Mapping(MappingArgs),
}

#[derive(Debug, clap::Args)]
pub struct MappingArgs {
    #[command(subcommand)]
    pub command: MappingCommand,
}

#[derive(Debug, Subcommand)]
pub enum MappingCommand {
    Add(MappingAddArgs),
    List,
    Show(MappingSourceArgs),
    Remove(MappingSourceArgs),
}

#[derive(Debug, clap::Args)]
pub struct MappingAddArgs {
    #[command(subcommand)]
    pub kind: MappingAddKind,
}

#[derive(Debug, Subcommand)]
pub enum MappingAddKind {
    File(MappingPairArgs),
    Tree(MappingPairArgs),
}

#[derive(Debug, clap::Args)]
pub struct MappingPairArgs {
    pub source: OsString,
    pub destination: OsString,
}

#[derive(Debug, clap::Args)]
pub struct MappingSourceArgs {
    pub source: OsString,
}

pub fn requests_json(args: &[OsString]) -> bool {
    args.windows(2)
        .any(|w| w[0] == "--output" && w[1] == "json")
        || args.iter().any(|a| a == "--output=json")
}
