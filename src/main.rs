#![feature(backtrace, byte_slice_trim_ascii, once_cell)]

mod xdcc;

use xdcc::search;

use std::backtrace::BacktraceStatus;
use std::io::Write;

use anyhow::{anyhow, Context, Result};
use clap::Parser;

#[derive(Parser)]
#[clap(version)]
struct Args {
    /// Anime search query
    #[clap(required = true)]
    query: Vec<String>,

    /// Set episode number
    #[clap(short, long, value_name = "NUM")]
    episode: Option<usize>,

    /// Set resolution
    #[clap(short, long, value_name = "RES", default_value_t = 1080)]
    resolution: usize,
}

fn run(args: &Args) -> Result<()> {
    let query = args.query.join(" ");
    let mut packs = search(&query, args.episode)?;

    if args.resolution != 0 {
        let res = format!("{}p", args.resolution);
        packs.retain(|p| p.name.contains(&res));
    }

    if packs.is_empty() {
        return Err(anyhow!("No results"));
    }

    println!("\x1b[;1mAvailable packs:\x1b[0m");
    for (i, pack) in packs.iter().enumerate() {
        println!(
            "    \x1b[;1m{:<3}\x1b[0m \x1b[34;1m{:<20}\x1b[0m \x1b[32;1m{:<6}\x1b[0m {}",
            i, pack.bot.name, pack.size, pack.name,
        );
    }
    print!("Enter selection [0]: ");
    let mut buf = String::new();
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut buf)?;

    let selection = if buf.trim().is_empty() {
        0
    } else if buf.trim() == "q" {
        return Ok(());
    } else {
        buf.trim()
            .parse::<usize>()
            .context("failed to parse selection")?
    };

    if selection > packs.len() - 1 {
        return Err(anyhow!("selection out of range"));
    }

    packs
        .swap_remove(selection)
        .download()
        .context("failed to download pack")?;

    Ok(())
}

fn main() {
    let result = run(&Args::parse());
    match result {
        Ok(_) => {}
        Err(error) => {
            eprintln!("adldl: {:#}", &error);
            if error.backtrace().status() == BacktraceStatus::Captured {
                eprint!("\nStack backtrace:\n{}", error.backtrace());
            }
            std::process::exit(1);
        }
    }
}
