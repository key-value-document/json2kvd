//! `json2kvd` — convert a JSON document to KVD (spec §4 grammar).

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

/// Convert JSON documents to KVD — and KVD to JSON with --reverse.
///
/// Without --reverse, reads JSON and writes KVD to stdout.
/// With --schema, the JSON schema is converted to a KVD schema node and
/// the converted document is verified against it before emitting.
/// With --reverse, reads KVD and writes JSON to stdout.
#[derive(Parser, Debug)]
#[command(name = "json2kvd", version, about)]
struct Cli {
    /// Input file (JSON by default, KVD when --reverse is set)
    input: PathBuf,

    /// JSON schema file to verify against (only for JSON → KVD)
    #[arg(long, value_name = "FILE", conflicts_with = "reverse")]
    schema: Option<PathBuf>,

    /// Reverse direction: read KVD and write JSON
    #[arg(long)]
    reverse: bool,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match run(&cli.input, cli.schema.as_deref(), cli.reverse) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("json2kvd: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(
    path: &std::path::Path,
    schema_path: Option<&std::path::Path>,
    reverse: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let text = std::fs::read_to_string(path)?;
    if reverse {
        // KVD → JSON
        print!("{}", json2kvd::kvd_text_to_json(&text)?);
    } else {
        // JSON → KVD
        let schema_text = schema_path.map(std::fs::read_to_string).transpose()?;
        print!(
            "{}",
            json2kvd::json_text_to_kvd(&text, schema_text.as_deref())?
        );
    }
    Ok(())
}
