use clap::{Parser, ArgAction};

// const MODEL_PATH: &str = "models/multilingual-MiniLM";
// const CONFIG_PATH: &str = "endpoints.yaml";
use crate::constants::{MODEL_PATH, CONFIG_PATH};

#[derive(Parser)]
#[command(
    name = "matcher",
    about = "Match natural language queries to endpoints",
    long_about = "A tool for semantically matching natural language queries to API endpoints using embeddings. \
                  You must specify either --server mode or provide a --query.",
    version,
    author = "Your Name <your.email@example.com>",
)]
pub struct Args {
    /// Reload the vector database
    #[arg(
        long,
        help = "Force reload of the vector database",
        default_value = "false"
    )]
    pub reload: bool,

    /// The natural language query to match
    #[arg(
        short,
        long,
        help = "The natural language query to match against endpoints",
        required_unless_present = "server"
    )]
    pub query: Option<String>,

    /// Enable debug output
    #[arg(
        long,
        help = "Enable debug logging",
        action = ArgAction::SetTrue
    )]
    pub debug: bool,

    /// Show all matching results
    #[arg(
        long,
        help = "Show all matching endpoints instead of just the best match",
        action = ArgAction::SetTrue
    )]
    pub all: bool,

    /// Specify the language for matching
    #[arg(
        short,
        long,
        help = "Language code for query matching (e.g., 'fr' for French, 'en' for English)",
        default_value = "fr"
    )]
    pub language: String,

    /// Run in server mode
    #[arg(
        long,
        help = "Run as a gRPC server instead of CLI mode",
        conflicts_with = "query",
        action = ArgAction::SetTrue
    )]
    pub server: bool,
}

pub fn parse_args() -> Args {
    Args::parse()
}

pub fn get_paths() -> (&'static str, &'static str) {
    (MODEL_PATH, CONFIG_PATH)
}
