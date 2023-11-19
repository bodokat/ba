#![warn(clippy::pedantic)]

use std::{collections::HashSet, fmt::Display, io::Write, str::FromStr, sync::atomic::AtomicUsize};

use formula::{modus_ponens, Formula, Normal};

use rayon::prelude::*;

mod formula;

pub static AXIOMS: &[&str] = &[
    "1 -> (2 -> 1)",
    "(1 -> 2 -> 3) -> (1 -> 2) -> (1 -> 3)",
    "(1 -> 2 -> 3) -> (2 -> 1 -> 3)",
    "(1 -> 2) -> (-2 -> -1)",
    "--1 -> 1",
    "1 -> --1",
];

fn main() {
    let mut context = Context::default();
    let runs: u64 = std::env::args()
        .nth(1)
        .and_then(|runs| u64::from_str(&runs).ok())
        .unwrap_or(5);
    let mut file = std::fs::File::create("output.txt").unwrap();

    for run in 0..runs {
        writeln!(file, "added in run {run}:").unwrap();
        for (_, entry) in context.entries.iter().enumerate() {
            writeln!(
                file,
                "{i}: {formula} [{source}]",
                i = entry.index,
                formula = entry.formula,
                source = entry.source
            )
            .unwrap();
        }
        writeln!(file).unwrap();

        context.step();
    }
}

struct Context {
    entries: HashSet<Entry>,
    new_entries: Vec<Entry>,
    next_idx: AtomicUsize,
}

#[derive(PartialEq, Eq, Hash)]
struct Entry {
    index: usize,
    source: Source,
    formula: Normal,
}

impl Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Source::Axiom => write!(f, "AXIOM"),
            Source::MP(a, b) => write!(f, "MP {a}, {b}"),
        }
    }
}

#[derive(PartialEq, Eq, Hash)]
enum Source {
    Axiom,
    MP(usize, usize),
}

impl Default for Context {
    fn default() -> Self {
        let new_entries: Vec<Entry> = AXIOMS
            .iter()
            .map(|s| Formula::from_str(s).unwrap().into())
            .enumerate()
            .map(|(index, f)| Entry {
                index,
                source: Source::Axiom,
                formula: f,
            })
            .collect();
        let next_idx = AtomicUsize::new(new_entries.len());
        Self {
            entries: HashSet::new(),
            new_entries,
            next_idx,
        }
    }
}

impl Context {
    // fn new_entries(&self) -> &[Entry] {
    //     &self.entries[self.new_entries_at..]
    // }

    // pub fn append(&mut self, other: Vec<(Normal, Source)>) {
    //     self.entries.reserve(other.len());
    //     for (f, s) in other {
    //         if !self.entries.iter().any(|e| e.formula == f) {
    //             self.entries.insert(Entry {
    //                 index: self.next_index,
    //                 source: s,
    //                 formula: f,
    //             });
    //             self.next_index += 1;
    //         }
    //     }
    // }

    fn step(&mut self) {
        fn try_mp_all<'a>(
            a: &'a (impl IntoParallelRefIterator<'a, Item = &'a Entry> + Sync),
            b: &'a (impl IntoParallelRefIterator<'a, Item = &'a Entry> + Sync),
        ) -> impl ParallelIterator<Item = (Normal, Source)> + 'a {
            a.par_iter()
                .filter(|e| e.formula.is_implication())
                .flat_map(|e1: &Entry| {
                    b.par_iter().filter_map(|e2: &Entry| {
                        Some((
                            modus_ponens(&e1.formula, &e2.formula)?,
                            Source::MP(e1.index, e2.index),
                        ))
                    })
                })
        }
        let res = try_mp_all(&self.entries, &self.new_entries)
            .chain(try_mp_all(&self.new_entries, &self.entries))
            .chain(try_mp_all(&self.new_entries, &self.new_entries));

        let new_entries = res
            .map(|(f, s)| Entry {
                index: self
                    .next_idx
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed),
                source: s,
                formula: f,
            })
            .collect();

        self.entries.extend(self.new_entries.drain(..));

        self.new_entries = new_entries;
    }
}
