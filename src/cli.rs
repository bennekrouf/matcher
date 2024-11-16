// src/cli.rs
use clap::Parser;

#[derive(Parser)]
#[command(
    name = "matcher",
    about = "Match natural language queries to endpoints",
    long_about = "A tool for semantically matching natural language queries to API endpoints using embeddings",
    version,
    author = "Your Name <your.email@example.com>",
    help_template = "{about}\n\nUSAGE:\n    {usage}\n\n{options}"
)]
pub struct Args {
    #[arg(long, default_value = "false")]
    pub reload: bool,
    #[arg(short, long)]
    pub query: Option<String>,
    #[arg(long)]
    pub debug: bool,
    #[arg(long)]
    pub all: bool,
    #[arg(short, long, default_value = "fr")]
    pub language: String,
    #[arg(long)]
    pub server: bool,
}

pub fn parse_args() -> Args {
    Args::parse()
}
