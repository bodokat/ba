use std::{
    fmt::Display,
    sync::atomic::{AtomicUsize, Ordering},
};

use hashbrown::HashSet;

use ahash::RandomState;
use rayon::prelude::*;

use crate::formula::language::{modus_ponens, Language, Normal};

const MAX_LEN: usize = 32;

#[derive(Debug)]
pub struct Context<L: Language> {
    pub entries: HashSet<Normal<L>, RandomState>,
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
        let entries = axioms.iter().cloned().collect();

        Self { entries }
    }

    pub fn step<F: Fn(&Normal<L>, &Normal<L>, &Normal<L>) + Send + Sync>(
        &mut self,
        for_each_new: F,
    ) {
        let max_len = AtomicUsize::new(0);
        let new_entries = self
            .entries
            .par_iter()
            .flat_map_iter(|f1| {
                self.entries.iter().filter_map(|f2| {
                    modus_ponens(f1, f2)
                        .inspect(|f| {
                            max_len.fetch_max(f.len(), Ordering::Relaxed);
                            for_each_new(f1, f2, f);
                        })
                        .filter(|f| f.len() <= MAX_LEN)
                })
            })
            // .filter(
            //     #[inline(never)]
            //     |f| !self.entries.contains(f),
            // )
            .collect_vec_list();

        println!("max len: {}", max_len.load(Ordering::Relaxed));
        self.entries
            .par_extend(new_entries.into_par_iter().flatten());
    }
}
