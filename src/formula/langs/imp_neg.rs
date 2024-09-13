#![allow(dead_code, clippy::enum_glob_use)]
use std::{fmt::Display, hash::Hash, mem};

use crate::formula::language::{Language, Normal, Simple, Term};

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub enum Variants<S> {
    Implication([S; 2]),
    Negation([S; 1]),
}

pub struct ImpNeg;

impl Language for ImpNeg {
    type Variant<S> = Variants<S>
        where S: Simple;

    fn matches<S: Simple>(this: &Self::Variant<S>, other: &Self::Variant<S>) -> bool {
        mem::discriminant(this) == mem::discriminant(other)
    }

    fn children<S: Simple>(this: &Self::Variant<S>) -> &[S] {
        match this {
            Variants::Implication(x) => x,
            Variants::Negation(x) => x,
        }
    }

    fn match_implication<S: Simple>(this: &Self::Variant<S>) -> Option<&[S; 2]> {
        match this {
            Variants::Implication(x) => Some(x),
            Variants::Negation(_) => None,
        }
    }

    fn map<S: Simple, T: Simple, F: FnMut(&S) -> T>(
        this: &Self::Variant<S>,
        mut f: F,
    ) -> Self::Variant<T> {
        match this {
            Variants::Implication([a, b]) => Variants::Implication([f(a), f(b)]),
            Variants::Negation([a]) => Variants::Negation([f(a)]),
        }
    }
}

impl Display for Variants<()> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Variants::Implication(_) => write!(f, "C"),
            Variants::Negation(_) => write!(f, "N"),
        }
    }
}

impl TryFrom<char> for Variants<()> {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'C' => Ok(Variants::Implication([(), ()])),
            'N' => Ok(Variants::Negation([()])),
            _ => Err(()),
        }
    }
}

impl ImpNeg {
    pub fn frege() -> Box<[Normal<ImpNeg>]> {
        [
            // a -> (b -> a)
            "CaCba",
            // (a -> (b -> c)) -> ((a -> b) -> (a -> c))
            "CCaCbcCCabCac",
            // (a -> (b -> c)) -> (b -> (a -> c))
            "CCaCbcCbCac",
            // (a -> b) -> (-b -> -a)
            "CCabCNbNa",
            // (--a -> a)
            "CNNaa",
            // (a -> --a)
            "CaNNa",
        ]
        .map(|x| x.parse().unwrap())
        .into()
    }

    pub fn lukasiewicz1() -> Vec<Normal<ImpNeg>> {
        [
            // (p -> q) -> ((q -> r) -> (p -> r))
            "CCpqCCqrCpr".parse().unwrap(),
            // (-p -> p) -> p
            "CCNppp".parse().unwrap(),
            // p -> (-p -> q)
            "CpCNpq".parse().unwrap(),
        ]
        .into()
    }

    pub fn lukasiewicz_tarski() -> Vec<Normal<ImpNeg>> {
        [
            // [(a -> (b -> a)) -> ([(-c -> (d -> -e)) -> [(c -> (d -> f)) -> ((e -> d) -> (e -> f))]] -> g)] -> (h -> g)
            "CCCaCbaCCCNcCdNeCCcCdfCCedCefgChg".parse().unwrap(),
        ]
        .into()
    }

    pub fn meredith() -> Vec<Normal<ImpNeg>> {
        ["CCCCCabCNcNdceCCeaCda".parse().unwrap()].into()
    }
}
