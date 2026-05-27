use std::fs;
use std::path::PathBuf;

use clap::Parser;
use walkdir::WalkDir;

use runtime_bindgen::generate_bindings;

#[derive(Parser)]
#[command(name = "runtime-bindgen", about = "Generate TypeScript bindings from Rust commands")]
struct Cli {
    /// Source directory to scan for Rust files
    source_dir: PathBuf,

    /// Output file path
    #[arg(long, default_value = "build/bindings.ts")]
    output: PathBuf,
}

fn main() {
    let cli = Cli::parse();

    let mut combined_source = String::new();

    for entry in WalkDir::new(&cli.source_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "rs") {
            if let Ok(contents) = fs::read_to_string(path) {
                combined_source.push_str(&contents);
                combined_source.push('\n');
            }
        }
    }

    let bindings = generate_bindings(&combined_source);

    if let Some(parent) = cli.output.parent() {
        fs::create_dir_all(parent).expect("failed to create output directory");
    }

    fs::write(&cli.output, bindings).expect("failed to write output file");

    println!("Bindings written to {}", cli.output.display());
}
