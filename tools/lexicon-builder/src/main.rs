mod build;
mod model;
mod preloaded_db;
mod preloaded_rows;

use std::path::PathBuf;

use anyhow::Context;

fn main() -> anyhow::Result<()> {
    let mut arguments = std::env::args_os().skip(1);
    let source_directory = arguments
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("data/source"));
    let output = arguments
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("data/generated/content.db"));
    if arguments.next().is_some() {
        anyhow::bail!("usage: polarbear-lexicon-builder [source-directory] [output-db]");
    }
    build::build(&source_directory, &output)
        .with_context(|| format!("failed to build {}", output.display()))?;
    println!("Built Polarbear Lexicon at {}", output.display());
    Ok(())
}
