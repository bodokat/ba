#![warn(clippy::pedantic)]

use std::{fmt::Display, io::Write, str::FromStr};

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
        for (i, entry) in context.new_entries().iter().enumerate() {
            let i = i + context.new_entries_at;
            writeln!(
                file,
                "{i}: {formula} [{source}]",
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
    entries: Vec<Entry>,
    new_entries_at: usize,
    next_index: usize,
}

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

enum Source {
    Axiom,
    MP(usize, usize),
}

impl Default for Context {
    fn default() -> Self {
        let entries: Vec<Entry> = AXIOMS
            .iter()
            .map(|s| Formula::from_str(s).unwrap().into())
            .enumerate()
            .map(|(index, f)| Entry {
                index,
                source: Source::Axiom,
                formula: f,
            })
            .collect();
        let next_index = entries.len();
        Self {
            entries,
            new_entries_at: 0,
            next_index,
        }
    }
}

impl Context {
    fn new_entries(&self) -> &[Entry] {
        &self.entries[self.new_entries_at..]
    }

    pub fn append(&mut self, other: Vec<(Normal, Source)>) {
        self.entries.reserve(other.len());
        for (f, s) in other {
            if !self.entries.iter().any(|e| e.formula == f) {
                self.entries.push(Entry {
                    index: self.next_index,
                    source: s,
                    formula: f,
                });
                self.next_index += 1;
            }
        }
    }

    fn step(&mut self) {
        fn try_mp_all<'a>(
            a: &'a [Entry],
            b: &'a [Entry],
        ) -> impl ParallelIterator<Item = (Normal, Source)> + 'a {
            a.par_iter()
                .filter(|e| e.formula.is_implication())
                .flat_map(
                    |Entry {
                         formula: f,
                         index: f_idx,
                         ..
                     }| {
                        b.par_iter().filter_map(
                            |Entry {
                                 index: p_idx,
                                 formula: p,
                                 ..
                             }| {
                                Some((modus_ponens(p, f)?, Source::MP(*p_idx, *f_idx)))
                            },
                        )
                    },
                )
        }

        let next_idx = self.entries.len();

        let (res_a, res_b) = (
            try_mp_all(&self.entries, &self.entries[self.new_entries_at..]),
            try_mp_all(&self.entries[self.new_entries_at..], &self.entries),
        );

        let res = res_a.chain(res_b).collect::<Vec<_>>();

        self.new_entries_at = next_idx;
    }
}
