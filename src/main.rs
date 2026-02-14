use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "obsidx", version, about = "Obsidian vault indexer")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize an index directory
    Init {
        #[arg(long)]
        vault: String,
        #[arg(long)]
        index: String,
    },
    /// Build or update the index
    Index {
        #[arg(long)]
        vault: String,
        #[arg(long, default_value = "./.obsidx")]
        index: String,
        #[arg(long, default_value_t = false)]
        incremental: bool,
    },
    /// Search the index
    Search {
        #[arg(long)]
        query: String,
        #[arg(long, default_value_t = 20)]
        limit: usize,
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// Get a note by path
    Get {
        #[arg(long)]
        path: String,
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// List tags
    Tags {
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// Link graph queries
    Links {
        #[arg(long)]
        from: String,
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// Index stats
    Stats {
        #[arg(long, default_value_t = false)]
        json: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { .. } => {
            println!("init: not implemented yet");
        }
        Commands::Index { .. } => {
            println!("index: not implemented yet");
        }
        Commands::Search { .. } => {
            println!("search: not implemented yet");
        }
        Commands::Get { .. } => {
            println!("get: not implemented yet");
        }
        Commands::Tags { .. } => {
            println!("tags: not implemented yet");
        }
        Commands::Links { .. } => {
            println!("links: not implemented yet");
        }
        Commands::Stats { .. } => {
            println!("stats: not implemented yet");
        }
    }
}
