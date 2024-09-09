#![warn(clippy::pedantic, clippy::perf)]
#![allow(clippy::question_mark, clippy::comparison_chain)]

use clap::Parser;
use std::{io::Write, str::FromStr};

mod context;
mod formula;
use formula::{
    langs::{self, ImpFalse},
    language::Normal,
};

use context::Context;

type Lang = ImpFalse;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Number of iterations
    #[arg(short, long, default_value_t = 5)]
    iterations: u32,

    #[arg(short, long)]
    search: Option<String>,
}

fn main() {
    let args = Args::parse();

    let search = args.search.map(|f| f.parse().unwrap());

    dbg!(&search);

    let mut context = Context::new(&langs::ImpFalse::church());
    let runs = args.iterations;
    // let mut file = std::fs::File::create("outputs/imp_false/meredith1.txt").unwrap();

    for run in 0..runs {
        context.step();

        let num_entries = context.entries.len();
        // writeln!(file, "added in run {run}: {num_entries} new entries").unwrap();
        // for (formula, meta) in &context.new_entries {
        //     writeln!(
        //         file,
        //         "{i}: {formula} [{source}]",
        //         i = meta.index,
        //         formula = formula,
        //         source = meta.source
        //     )
        //     .unwrap();
        // }
        // writeln!(file).unwrap();

        println!("run {run}, {num_entries} new");

        if let Some(f) = &search {
            if let Some(found) = context.entries.get(f) {
                println!("Found formula ({f}) after {run} iterations");
                return;
            } else {
                println!("Formula ({f}) not found in iteration {run}");
            }
        }
    }
}
