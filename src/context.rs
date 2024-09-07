use std::{collections::HashMap, fmt::Display, sync::atomic::AtomicUsize};

use rayon::prelude::*;

use crate::formula::language::{modus_ponens, Language, Normal};

#[derive(Debug)]
pub struct Context<L: Language> {
    entries: HashMap<Normal<L>, Meta>,
    pub new_entries: HashMap<Normal<L>, Meta>,
    next_idx: AtomicUsize,
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

impl<L: Language> Context<L> {
    pub fn new(axioms: &[Normal<L>]) -> Self {
        let new_entries = axioms
            .iter()
            .enumerate()
            .map(|(index, f)| {
                (
                    f.clone(),
                    Meta {
                        index,
                        source: Source::Axiom,
                    },
                )
            })
            .collect::<HashMap<_, _>>();

        let next_idx = new_entries.len();
        Self {
            entries: HashMap::new(),
            new_entries,
            next_idx: AtomicUsize::new(next_idx),
        }
    }

    pub fn step(&mut self) {
        let new_entries = self
            .entries
            .par_iter()
            .flat_map(|(f1, m1)| {
                self.entries.par_iter().filter_map(|(f2, m2)| {
                    modus_ponens(f1, f2).map(|res| (res, Source::MP(m1.index, m2.index)))
                })
            })
            .chain(self.entries.par_iter().flat_map(|(f1, m1)| {
                self.new_entries.par_iter().filter_map(|(f2, m2)| {
                    modus_ponens(f1, f2).map(|res| (res, Source::MP(m1.index, m2.index)))
                })
            }))
            .chain(self.new_entries.par_iter().flat_map(|(f1, m1)| {
                self.entries.par_iter().filter_map(|(f2, m2)| {
                    modus_ponens(f1, f2).map(|res| (res, Source::MP(m1.index, m2.index)))
                })
            }))
            .filter(|(f, _)| !self.entries.contains_key(f) && !self.new_entries.contains_key(f))
            .map(|(f, source)| {
                (
                    f,
                    Meta {
                        source,
                        index: self
                            .next_idx
                            .fetch_add(1, std::sync::atomic::Ordering::Relaxed),
                    },
                )
            })
            .collect::<HashMap<_, _>>();

        self.entries.par_extend(self.new_entries.par_drain());

        self.new_entries = new_entries;
    }
}
