use std::fs;
use std::io::{self, Read, Write};
use std::process::ExitCode;

use clap::Parser;
use rsomics_common::CommonFlags;

use rsomics_link_prediction::{format_text, link_prediction_from_edge_list, to_json, Method};

/// Score node pairs by neighborhood overlap (link prediction), value-exact
/// against networkx 3.6.1.
#[derive(Parser, Debug)]
#[command(name = "rsomics-link-prediction", version, about, long_about = None)]
struct Cli {
    /// Scoring method.
    #[arg(
        long,
        default_value = "jaccard",
        value_parser = ["jaccard", "adamic-adar", "resource-allocation", "preferential-attachment"],
    )]
    method: String,

    /// File of `u v` node pairs to score (one per line). Omit to score all
    /// non-adjacent pairs (nx `ebunch=None`).
    #[arg(long)]
    pairs: Option<String>,

    #[command(flatten)]
    common: CommonFlags,
}

fn read_pairs(path: &str) -> anyhow::Result<Vec<(String, String)>> {
    let text = fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("cannot read --pairs file {path}: {e}"))?;
    let mut pairs = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut it = line.split_whitespace();
        match (it.next(), it.next()) {
            (Some(a), Some(b)) => pairs.push((a.to_string(), b.to_string())),
            _ => anyhow::bail!("--pairs line is not a `u v` pair: {line:?}"),
        }
    }
    Ok(pairs)
}

fn run(cli: &Cli) -> anyhow::Result<()> {
    let method = Method::parse(&cli.method).map_err(|e| anyhow::anyhow!(e))?;

    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let pairs = match &cli.pairs {
        Some(p) => Some(read_pairs(p)?),
        None => None,
    };

    let preds = link_prediction_from_edge_list(&input, method, pairs.as_deref())?;

    let stdout = io::stdout();
    let mut w = stdout.lock();
    if cli.common.json {
        serde_json::to_writer(&mut w, &to_json(method, &preds))?;
        writeln!(w)?;
    } else {
        w.write_all(format_text(method, &preds).as_bytes())?;
    }

    if !cli.common.quiet {
        eprintln!(
            "scored {} pair(s) with --method {}",
            preds.len(),
            cli.method
        );
    }
    Ok(())
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(&cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_debug_assert() {
        Cli::command().debug_assert();
    }
}
