use std::{collections::HashMap, fmt::Display, sync::mpsc, thread};

use rayon::prelude::*;

use crate::formula::language::{modus_ponens, Language, Normal};

#[derive(Debug)]
pub struct Context<L: Language> {
    entries: HashMap<Normal<L>, Meta>,
    pub new_entries: HashMap<Normal<L>, Meta>,
    next_idx: usize,
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
        Self {
            entries: HashMap::new(),
            new_entries,
            next_idx,
        }
    }

    pub fn step(&mut self) {
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
                });
            });
        }

        let (tx, rx) = mpsc::channel();

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
