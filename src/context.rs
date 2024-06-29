use std::{collections::HashMap, fmt::Display, sync::mpsc, thread};

use rayon::prelude::*;

use crate::formula::language::{modus_ponens, Language, Normal};

pub static AXIOMS: &[&str] = &[
    "1 -> (2 -> 1)",
    "(1 -> 2 -> 3) -> (1 -> 2) -> (1 -> 3)",
    "(1 -> 2 -> 3) -> (2 -> 1 -> 3)",
    "(1 -> 2) -> (-2 -> -1)",
    "--1 -> 1",
    "1 -> --1",
];

#[derive(Debug)]
pub struct Context<L: Language> {
    entries: HashMap<Normal<L>, Meta>,
    pub new_entries: HashMap<Normal<L>, Meta>,
    next_idx: usize,
    max_size: usize,
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct Meta {
    pub index: usize,
    pub source: Source,
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum Source {
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

// impl<L: Language> Default for Context<L> {
//     fn default() -> Self {
//         let new_entries: HashMap<_, _> = AXIOMS
//             .iter()
//             .map(|s| Formula::from_str(s).unwrap().into())
//             .enumerate()
//             .map(|(index, f): (usize, Normal<L>)| {
//                 (
//                     f,
//                     Meta {
//                         index,
//                         source: Source::Axiom,
//                     },
//                 )
//             })
//             .collect();
//         let next_idx = new_entries.len();
//         let max_size = new_entries.iter().map(|(f, _)| f.size()).max().unwrap();
//         Self {
//             entries: HashMap::new(),
//             new_entries,
//             next_idx,
//             max_size,
//         }
//     }
// }

impl<L: Language> Context<L> {
    pub fn new(axioms: Vec<Normal<L>>) -> Self {
        let new_entries = axioms
            .into_iter()
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
            .collect::<HashMap<_, _>>();

        let next_idx = new_entries.len();
        let max_size = new_entries.iter().map(|(f, _)| f.len()).max().unwrap();
        Self {
            entries: HashMap::new(),
            new_entries,
            next_idx,
            max_size,
        }
    }

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

    pub fn step(&mut self) {
        let (tx, rx) = mpsc::channel();

        fn try_mp_all<'a, L: Language>(
            a: &'a (impl IntoParallelRefIterator<'a, Item = (&'a Normal<L>, &'a Meta)> + Send + Sync),
            b: &'a (impl IntoParallelRefIterator<'a, Item = (&'a Normal<L>, &'a Meta)> + Send + Sync),
            chan: &mpsc::Sender<(Normal<L>, Source)>,
        ) {
            a.par_iter().for_each(|(f1, m1): (&Normal<L>, &Meta)| {
                b.par_iter().for_each(|(f2, m2): (&Normal<L>, &Meta)| {
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

// fn uninit_slice<L: Language>(n: usize) -> Box<[MaybeUninit<Entry>]> {
//     std::iter::repeat_with(MaybeUninit::uninit)
//         .take(n)
//         .collect::<Vec<_>>()
//         .into_boxed_slice()
// }
