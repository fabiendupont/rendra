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

    /// Build the application in release mode
    Build {
        /// Project directory
        #[arg(short, long, default_value = ".", help = "Project directory")]
        path: PathBuf,
    },

    /// Build and run an application in development mode
    Dev {
        /// Project directory
        #[arg(short, long, default_value = ".", help = "Project directory")]
        path: PathBuf,
    },

    /// Package the application as an AppImage
    Package {
        /// Project directory
        #[arg(short, long, default_value = ".", help = "Project directory")]
        path: PathBuf,
    },

    /// Check permissions: audit app.toml vs actual API usage
    CheckPermissions {
        /// Project directory
        #[arg(short, long, default_value = ".", help = "Project directory")]
        path: PathBuf,
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
        Commands::Build { path } => {
            if let Err(e) = runtime_cli::build::run_build(&path) {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }
        Commands::Dev { path } => {
            if let Err(e) = runtime_cli::dev::run_dev(&path) {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }
        Commands::Package { path } => {
            match runtime_cli::package::run_package(&path) {
                Ok(output) => {
                    println!("Package ready: {}", output.display());
                }
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }
        Commands::CheckPermissions { path } => {
            match runtime_cli::check_permissions::run_check(&path) {
                Ok(result) => {
                    let over = result.over_permissioned();
                    let under = result.under_permissioned();

                    if !over.is_empty() {
                        println!("Warnings (declared but unused):");
                        for p in &over {
                            println!("  - {p}");
                        }
                    }
                    if !under.is_empty() {
                        eprintln!("Errors (used but not declared in app.toml):");
                        for p in &under {
                            eprintln!("  - {p}");
                        }
                        std::process::exit(1);
                    }

                    if over.is_empty() && under.is_empty() {
                        println!("Permissions OK: declared permissions match API usage.");
                    }
                }
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }
    }
}
