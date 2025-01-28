use shim_maker::*;
use anyhow::{Context, Result};
use clap::Parser;
use std::io::Write;
use std::{fs::File, path::PathBuf};

#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// Path to the DLL file to create a shim for
    library: PathBuf,
    /// Explicit name of the DLL to import, will default to the name of the DLL provided with 'target'
    name: Option<String>,
}


fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let data = std::fs::read(&args.library)?;
    let pe = goblin::pe::PE::parse(&data).context("Failed to parse PE file")?;
    let exports = parse_exports(&pe).context("Unable to parse exports")?;

    let name = args
        .name
        .or(args
            .library
            .file_name()
            .and_then(|os| os.to_str())
            .map(|s| s.to_owned()))
        .context("Unable to get DLL name")?;
    let code = code_gen(&exports, &name)?;
    std::io::stdout().write_all(&code)?;
    Ok(())
}


