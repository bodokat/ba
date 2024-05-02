#![warn(clippy::pedantic)]
#![feature(maybe_uninit_slice)]

use std::{io::Write, str::FromStr};

mod context;
mod formula;

use context::Context;

fn main() {
    let mut context = Context::default();
    let runs: u64 = std::env::args()
        .nth(1)
        .and_then(|runs| u64::from_str(&runs).ok())
        .unwrap_or(5);
    let mut file = std::fs::File::create("output.txt").unwrap();

    for run in 0..runs {
        writeln!(file, "added in run {run}:").unwrap();
        for (_, (formula, meta)) in context.new_entries.iter().enumerate() {
            writeln!(
                file,
                "{i}: {formula} [{source}]",
                i = meta.index,
                formula = formula,
                source = meta.source
            )
            .unwrap();
        }
        writeln!(file).unwrap();

        println!("run {run}");

        context.step();
    }
}
