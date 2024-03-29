#![warn(clippy::pedantic)]

use std::{
    collections::HashMap,
    fmt::Display,
    io::Write,
    str::FromStr,
    sync::mpsc,
    thread,
};

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

struct Context {
    entries: HashMap<Normal, Meta>,
    new_entries: HashMap<Normal, Meta>,
    next_idx: usize,
}

#[derive(PartialEq, Eq, Hash)]
struct Meta {
    index: usize,
    source: Source,
}

#[derive(PartialEq, Eq, Hash)]
enum Source {
    Axiom,
    MP(usize, usize),
}

impl Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Source::Axiom => write!(f, "AXIOM"),
            Source::MP(a, b) => write!(f, "MP {a}, {b}"),
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        let new_entries: HashMap<_, _> = AXIOMS
            .iter()
            .map(|s| Formula::from_str(s).unwrap().into())
            .enumerate()
            .map(|(index, f)| {
                (
                    f,
                    Meta {
                        index,
                        source: Source::Axiom,
                    },
                )
            })
            .collect();
        let next_idx = new_entries.len();
        Self {
            entries: HashMap::new(),
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
        let (tx, rx) = mpsc::channel();

        fn try_mp_all<'a>(
            a: &'a (impl IntoParallelRefIterator<'a, Item = (&'a Normal, &'a Meta)> + Send + Sync),
            b: &'a (impl IntoParallelRefIterator<'a, Item = (&'a Normal, &'a Meta)> + Send + Sync),
            chan: &mpsc::Sender<(Normal, Source)>,
        ) {
            a.par_iter().for_each(|(f1, m1): (&Normal, &Meta)| {
                b.par_iter().for_each(|(f2, m2): (&Normal, &Meta)| {
                    if let Some(res) = modus_ponens(f1, f2) {
                        chan.send((res, Source::MP(m1.index, m2.index))).unwrap();
                    }
                })
            })
        }

        let mut next_idx = self.next_idx;

        let t = thread::spawn(move || {
            let mut new_entries = HashMap::new();

            for (f, source) in rx {
                new_entries.entry(f).or_insert_with(|| Meta {
                    index: next_idx,
                    source,
                });
                next_idx += 1;
            }


            (new_entries, next_idx)
        });

        rayon::join(
            || {
                rayon::join(
                    || try_mp_all(&self.entries, &self.new_entries, &tx),
                    || try_mp_all(&self.new_entries, &self.entries, &tx),
                )
            },
            || try_mp_all(&self.new_entries, &self.new_entries, &tx),
        );
        drop(tx);

        let (new_entries, next_idx) = t.join().unwrap();

        self.entries.extend(self.new_entries.drain());
        self.next_idx = next_idx;

        self.new_entries = new_entries;
    }
}
