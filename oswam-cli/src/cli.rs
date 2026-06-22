use clap::{Parser, Subcommand};
use oswam_core::delete::Disposition;

#[derive(Parser, Debug)]
#[command(name = "oswam", about = "Безопасная очистка места на macOS", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Scan {
        #[arg(long)]
        json: bool,
    },
    Clean {
        #[arg(long)]
        safe: bool,
        #[arg(long, value_delimiter = ',')]
        category: Vec<String>,
        #[arg(long)]
        dry_run: bool,
        #[arg(long, conflicts_with = "delete")]
        trash: bool,
        #[arg(long)]
        delete: bool,
        #[arg(long)]
        yes: bool,
    },
}

pub fn disposition(trash: bool, delete: bool) -> Disposition {
    if delete {
        Disposition::Permanent
    } else {
        let _ = trash;
        Disposition::Trash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_subcommand_is_tui() {
        let cli = Cli::try_parse_from(["oswam"]).unwrap();
        assert!(cli.command.is_none());
    }

    #[test]
    fn scan_json_flag() {
        let cli = Cli::try_parse_from(["oswam", "scan", "--json"]).unwrap();
        assert!(matches!(cli.command, Some(Command::Scan { json: true })));
    }

    #[test]
    fn clean_category_list() {
        let cli = Cli::try_parse_from(["oswam", "clean", "--category", "dev,browsers"]).unwrap();
        match cli.command {
            Some(Command::Clean { category, .. }) => {
                assert_eq!(category, vec!["dev", "browsers"]);
            }
            _ => panic!("expected clean"),
        }
    }

    #[test]
    fn trash_and_delete_conflict() {
        let res = Cli::try_parse_from(["oswam", "clean", "--trash", "--delete"]);
        assert!(res.is_err());
    }

    #[test]
    fn disposition_defaults_to_trash() {
        assert_eq!(disposition(false, false), Disposition::Trash);
        assert_eq!(disposition(true, false), Disposition::Trash);
        assert_eq!(disposition(false, true), Disposition::Permanent);
    }
}
