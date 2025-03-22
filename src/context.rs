use std::{
    fmt::Display,
    sync::atomic::{AtomicUsize, Ordering},
};

use ahash::HashMap;
use rayon::prelude::*;

use crate::formula::language::{modus_ponens, Language, Normal};

const MAX_LEN: usize = 64;

#[derive(Debug)]
pub struct Context<L: Language> {
    pub entries: HashMap<Normal<L>, Meta>,
    next_idx: AtomicUsize,
}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct Meta {
    pub index: usize,
    pub sources: Vec<Source>,
}

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
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
        let entries = axioms
            .iter()
            .enumerate()
            .map(|(index, f)| {
                (
                    f.clone(),
                    Meta {
                        index,
                        sources: vec![Source::Axiom],
                    },
                )
            })
            .collect::<HashMap<_, _>>();

        let next_idx = entries.len();
        Self {
            entries,
            next_idx: AtomicUsize::new(next_idx),
        }
    }

    pub fn new_entries_iter<'a, F: Fn(&Normal<L>, &Normal<L>, &Normal<L>) + Send + Sync + 'a>(
        &'a mut self,
        for_each_new: &'a F,
    ) -> impl ParallelIterator<Item = (Normal<L>, Source, usize)> + 'a {
        self.entries.par_iter().flat_map_iter(|(f1, m1)| {
            self.entries.iter().filter_map(|(f2, m2)| {
                modus_ponens(f1, f2)
                    .filter(|f|
                        // f.len() < MAX_LEN &&
                        !self.entries.contains_key(f))
                    .inspect(|f| for_each_new(f1, f2, f))
                    .map(|res| {
                        (
                            res,
                            Source::MP(m1.index, m2.index),
                            self.next_idx.fetch_add(1, Ordering::Relaxed),
                        )
                    })
            })
        })
    }

    pub fn step<F: Fn(&Normal<L>, &Normal<L>, &Normal<L>) + Send + Sync>(
        &mut self,
        for_each_new: &F,
    ) {
        let new_entries = self.new_entries_iter(for_each_new).collect_vec_list();

        new_entries
            .into_iter()
            .flatten()
            .for_each(|(f, source, index)| {
                let entry = self.entries.entry(f);
                let meta = entry.or_insert(Meta {
                    index,
                    sources: Vec::new(),
                });
                if !meta.sources.contains(&source) {
                    meta.sources.push(source);
                }
            });
        // println!("max len: {}", max_len.load(Ordering::Relaxed));
    }
}
