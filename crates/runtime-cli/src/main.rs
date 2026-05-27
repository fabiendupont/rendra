use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "runtime-cli", about = "CLI for the Servo-based application runtime")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new application project
    Init {
        /// Project name
        name: String,

        /// Directory to create the project in (defaults to ./<name>)
        #[arg(long)]
        path: Option<PathBuf>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { name, path } => {
            let project_path = path.unwrap_or_else(|| PathBuf::from(&name));

            match runtime_cli::init::scaffold_project(&project_path, &name) {
                Ok(()) => {
                    println!("Created project '{}' at {}", name, project_path.display());
                }
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }
    }
}
