mod cli;
mod commands;
mod context;
mod output;
mod perform;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command};
use commands::{cmd_clean, cmd_scan, cmd_tui};
use context::load_env;

fn main() -> Result<()> {
    let args = Cli::parse();
    let env = load_env()?;
    match args.command {
        None => cmd_tui(&env),
        Some(Command::Scan { json }) => cmd_scan(&env, json),
        Some(Command::Clean {
            safe,
            category,
            dry_run,
            trash,
            delete,
            yes,
        }) => cmd_clean(&env, safe, category, dry_run, trash, delete, yes),
    }
}
