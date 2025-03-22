#![warn(clippy::pedantic, clippy::perf)]
#![allow(clippy::question_mark, clippy::comparison_chain)]

use std::{
    io::{self, Write},
    ops::AddAssign,
};

use ahash::{HashMap, HashMapExt};
use clap::Parser;

mod context;
mod formula;
use formula::langs;

use context::{Context, Source};
use itertools::Itertools;
use rayon::iter::{ParallelDrainFull, ParallelExtend, ParallelIterator};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Number of iterations
    #[arg(short, long, default_value_t = 5)]
    iterations: u32,

    #[arg(short, long)]
    search: Option<String>,

    #[arg(long)]
    stats: Option<String>,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let search = args.search.map(|f| f.parse().unwrap());

    let mut context = Context::new(&langs::ImpNeg::lukasiewicz_tarski());
    let runs = args.iterations;

    let mut found = None;

    for run in 0..runs {
        context.step(&(|_, _, _| ()));

        let num_entries = context.entries.len();

        println!("run {run}, {num_entries} new");

        if let Some(f) = &search {
            if let Some(formula) = context.entries.get(f) {
                println!("Found formula ({f}) after {run} iterations");
                found = Some(formula.clone());
                break;
                // if let Source::MP(s1, s2) = found.source {
                //     let prev = context.entries.iter().filter(|(f,m)| m.index == s1 || m.index == s2).collect();
                // }
            } else {
                println!("Formula ({f}) not found in iteration {run}");
            }
        }

        // if let Some(search) = &search {
        //     let found = AtomicBool::new(false);
        //     context.step(|f1, f2, f| {
        //         if f == search {
        //             println!("Found formula ({f}) after {run} iterations");
        //             println!("From MP with ({f1}), ({f2})");
        //             found.store(true, Ordering::Relaxed);
        //         }
        //     });
        //     if found.load(Ordering::Relaxed) {
        //         break;
        //     }
        // }

        println!("run {run} complete");
    }

    if let Some(search) = search {
        if let Some(found) = found {
            let mut derivation = HashMap::new();
            derivation.insert(found.index, (found.clone(), search));

            let mut to_find: Vec<_> = found
                .sources
                .iter()
                .filter_map(|s| {
                    if let &Source::MP(a, b) = s {
                        Some([a, b].into_iter())
                    } else {
                        None
                    }
                })
                .flatten()
                .collect();

            while !to_find.is_empty() {
                let new = context
                    .entries
                    .iter()
                    .filter(|(_, m)| to_find.contains(&m.index))
                    .flat_map(|(e, m)| {
                        derivation.insert(m.index, (m.clone(), e.clone()));
                        m.sources
                            .iter()
                            .filter_map(|s| {
                                if let &Source::MP(a, b) = s {
                                    Some([a, b].into_iter())
                                } else {
                                    None
                                }
                            })
                            .flatten()
                    })
                    .collect();

                to_find = new;
            }

            let mut derivation = derivation.drain().collect::<Vec<_>>();
            derivation.sort_by_key(|(i, _)| *i);

            for (i, (meta, formula)) in derivation {
                println!("{i}: {formula} ({s})", s = meta.sources.iter().join("; "));
            }
        }
    }

    if let Some(path) = args.stats {
        let mut file = std::fs::File::create(path).unwrap();

        println!("generating stats...");

        let stats = context
            .new_entries_iter(&(|_, _, _| ()))
            .fold(HashMap::new, |mut acc, val| {
                acc.entry(val.0.len()).or_insert(0).add_assign(1);
                acc
            })
            .reduce(HashMap::new, |mut a, mut b| {
                a.par_extend(b.par_drain());
                a
            });

        writeln!(file, "len,amount")?;

        for (len, amount) in &stats {
            writeln!(file, "{len},{amount}")?;
        }

        println!("size: {}", stats.len());
    }
    Ok(())
}
