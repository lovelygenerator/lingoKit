//! CLI。help はソース上で英語で書き、表示時に FTL(cli-*)で上書きされる。

use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use core_lib::msg;
use lingokit::Localizer;

#[derive(Parser, Debug)]
#[command(version, about = "Sample CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Count words in a text file
    Count {
        /// Path to the text file
        path: PathBuf,
        /// Output format
        #[arg(long, value_enum, default_value_t = Format::Text)]
        format: Format,
    },
}

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum Format {
    /// Human readable text
    Text,
    /// JSON object
    Json,
}

pub fn run(cli: Cli, loc: &Localizer) -> core_lib::Result<()> {
    match cli.command {
        Commands::Count { path, format } => {
            let count = core_lib::count_words(&path)?;
            match format {
                Format::Text => println!("{}", loc.msg(&msg::result_word_count(count, path.display().to_string()))),
                Format::Json => println!("{}", serde_json::json!({ "path": path, "count": count })),
            }
        }
    }
    Ok(())
}
