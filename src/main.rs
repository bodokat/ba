#![warn(clippy::pedantic)]
#![allow(clippy::question_mark, clippy::comparison_chain)]

use std::{io::Write, str::FromStr};

mod context;
mod formula;
use formula::langs;

use context::Context;

fn main() {
    let mut context = Context::new(&langs::ImpNeg::meredith());
    let runs: u64 = std::env::args()
        .nth(1)
        .and_then(|runs| u64::from_str(&runs).ok())
        .unwrap_or(5);
    let mut file = std::fs::File::create("outputs/meredith.txt").unwrap();

    for run in 0..runs {
        context.step();

        let num_entries = context.new_entries.len();
        writeln!(file, "added in run {run}: {num_entries} new entries").unwrap();
        for (formula, meta) in &context.new_entries {
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

        println!("run {run}, {num_entries} new");
    }
}
