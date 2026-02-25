use crate::cli::init_project;
use crate::cli::run;
use crate::core::persistence::loader::SnapshotSelection;
use clap::builder::Str;
use clap::{Parser, Subcommand};
use std::env;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "vivarium-cli", about = "Vivarium project utilities")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(name = "init-project", alias = "init", about = "Create a new Vivarium project structure")]
    InitProject {
        #[arg(value_name = "target-dir")]
        target_dir: PathBuf,
    },
    Run {
        #[arg(value_name = "project-dir", default_value = ".")]
        project_dir: PathBuf,
        #[arg(value_name = "steps", help = "Number of simulation steps to run")]
        steps: u32,
        #[arg(default_value = "latest", help = "Specify which snapshot to load: 'latest' or a specific snapshot name")]
        snapshot: SnapshotSelection,
        #[arg(value_name = "save-snapshot", default_value = None, help = "Optionally specify a name to save the snapshot after running")]
        save_snapshot: Option<String>,
    },
}

pub fn run_from_env() -> Result<(), String> {
    let cli = Cli::try_parse_from(env::args()).map_err(|e| e.to_string())?;

    match cli.command {
        Commands::InitProject { target_dir } => init_project::init_project(&target_dir),
        Commands::Run {
            project_dir,
            steps,
            snapshot,
            save_snapshot,
        } => run::run_project(project_dir, steps, snapshot, save_snapshot),
    }
}
