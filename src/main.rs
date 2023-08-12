#![doc = include_str!("../README.md")]
use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
/// Create machine-readable JSON diffs
struct Args {
    /// Path to the JSON file you wish to compare against
    base_json: PathBuf,

    /// Path to the JSON file to be compared
    compare_json: PathBuf,
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("Failed to load '{0}': {1}")]
    FileRead(PathBuf, std::io::Error),

    #[error("Failed to parse '{0}' as JSON: {1}")]
    SerdeParse(PathBuf, serde_json::Error),
}

fn read_to_json(path: PathBuf) -> Result<serde_json::Map<String, serde_json::Value>, Error> {
    serde_json::from_str(
        std::fs::read_to_string(&path)
            .map_err(|err| Error::FileRead(path.clone(), err))?
            .as_str(),
    )
    .map_err(|err| Error::SerdeParse(path.clone(), err))
}

fn run() -> Result<Option<String>, Error> {
    let args = Args::parse();

    let base_json = read_to_json(args.base_json)?;
    let compare_json = read_to_json(args.compare_json)?;

    let possible_pretty_diff = serde_json_compare::objects(base_json, compare_json)
        .map(|diff| serde_json::to_string_pretty(&diff))
        .transpose()
        .expect("Failure to serialize should not be possible here");

    Ok(possible_pretty_diff)
}

fn main() {
    match run() {
        Ok(Some(pretty_diff)) => {
            println!("{pretty_diff}");
            std::process::exit(1);
        }
        Ok(None) => std::process::exit(0),
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(2);
        }
    }
}
