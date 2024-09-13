#![warn(clippy::pedantic, clippy::perf)]
#![allow(clippy::question_mark, clippy::comparison_chain)]

use std::sync::atomic::{AtomicBool, Ordering};

use clap::Parser;

mod context;
mod formula;
use formula::langs::{self};

use context::Context;

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

    let mut context = Context::new(&langs::ImpNeg::lukasiewicz1());
    let runs = args.iterations;
    // let mut file = std::fs::File::create("outputs/imp_false/meredith1.txt").unwrap();

    for run in 0..runs {
        if let Some(search) = &search {
            let found = AtomicBool::new(false);
            context.step(|f1, f2, f| {
                if f == search {
                    println!("Found formula ({f}) after {run} iterations");
                    println!("From MP with ({f1}), ({f2})");
                    found.store(true, Ordering::Relaxed);
                }
            });
            if found.load(Ordering::Relaxed) {
                break;
            }
        }

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

        println!("run {run}, {num_entries} found");
    }
}
