use crate::result::OutputMode;
use clap::{Parser, Subcommand, ValueEnum};
use std::ffi::OsString;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "grip", version, about = "Safely manage per-user file mappings")]
pub struct Cli {
    #[arg(short = 'p', long, global = true, value_name = "PATH")]
    pub project: Option<PathBuf>,
    #[arg(short = 'o', long, global = true, value_enum, value_name = "FORMAT")]
    pub output: Option<OutputArg>,
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputArg {
    Json,
}
impl From<Option<OutputArg>> for OutputMode {
    fn from(value: Option<OutputArg>) -> Self {
        match value {
            Some(OutputArg::Json) => Self::Json,
            None => Self::Human,
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Initialize an existing directory as a Grip project.
    Init(InitArgs),
    Version,
    /// Record a source-to-destination mapping without copying either endpoint.
    Add(AddArgs),
    /// List mappings, optionally selecting one source path.
    List(ListArgs),
    /// Remove a mapping declaration without changing endpoint payloads.
    Remove(RemoveArgs),
    /// Report synchronization state and validate Grip-owned metadata.
    Status(StatusArgs),
    /// Show detailed source and destination differences without mutation.
    Diff(InspectionArgs),
    /// Copy an eligible source-side change to its mapped destination.
    Push(PushArgs),
    /// Copy an eligible destination-side change back to its mapped source.
    Pull(PullArgs),
    /// Synchronize all unambiguous non-absent changes in both directions.
    Sync(SyncArgs),
}

#[derive(Debug, clap::Args)]
pub struct AddArgs {
    #[arg(value_name = "SOURCE")]
    pub source: OsString,
    #[arg(value_name = "DESTINATION")]
    pub destination: OsString,
}

#[derive(Debug, clap::Args)]
pub struct ListArgs {
    #[arg(value_name = "SOURCE")]
    pub source: Option<OsString>,
}

#[derive(Debug, clap::Args)]
pub struct RemoveArgs {
    #[arg(value_name = "SOURCE")]
    pub source: OsString,
}

#[derive(Debug, clap::Args)]
pub struct InitArgs {
    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,
}

/// Arguments shared by push preview and execution.
#[derive(Debug, clap::Args)]
pub struct PushArgs {
    /// Preview the complete plan without locking or mutation.
    #[arg(short = 'n', long = "dry-run")]
    pub dry_run: bool,
    /// Interpret PATH in destination space without reversing push direction.
    #[arg(short = 'f', long)]
    pub force: bool,
    #[arg(short = 'd', long)]
    pub destination: bool,
    /// Select one mapping, entry, or component-boundary subtree.
    #[arg(value_name = "PATH")]
    pub path: Option<OsString>,
}

/// Arguments shared by pull preview and execution.
#[derive(Debug, clap::Args)]
pub struct PullArgs {
    /// Preview the complete plan without locking or mutation.
    #[arg(short = 'n', long = "dry-run")]
    pub dry_run: bool,
    /// Interpret PATH in destination space without changing pull direction.
    #[arg(short = 'f', long)]
    pub force: bool,
    #[arg(short = 'd', long)]
    pub destination: bool,
    /// Select one mapping, entry, or component-boundary subtree.
    #[arg(value_name = "PATH")]
    pub path: Option<OsString>,
}

/// Arguments shared by bidirectional sync preview and execution.
#[derive(Debug, clap::Args)]
pub struct SyncArgs {
    /// Preview the complete mixed-direction plan without locking or mutation.
    #[arg(short = 'n', long = "dry-run")]
    pub dry_run: bool,
    /// Interpret PATH in destination space without changing action directions.
    #[arg(short = 'd', long)]
    pub destination: bool,
    /// Select one mapping, entry, or component-boundary subtree.
    #[arg(value_name = "PATH")]
    pub path: Option<OsString>,
}

#[derive(Debug, clap::Args)]
pub struct InspectionArgs {
    #[arg(short = 'd', long)]
    pub destination: bool,
    #[arg(value_name = "PATH")]
    pub path: Option<OsString>,
}

#[derive(Debug, clap::Args)]
pub struct StatusArgs {
    #[arg(short = 'e', long)]
    pub exit_code: bool,
    #[arg(short = 'd', long)]
    pub destination: bool,
    #[arg(value_name = "PATH")]
    pub path: Option<OsString>,
}

pub fn requests_json(args: &[OsString]) -> bool {
    args.windows(2)
        .any(|w| (w[0] == "--output" || w[0] == "-o") && w[1] == "json")
        || args
            .iter()
            .any(|a| a == "--output=json" || a == "-ojson" || a == "-o=json")
}
