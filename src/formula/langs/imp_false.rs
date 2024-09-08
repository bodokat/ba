use std::{fmt::Display, mem};

use crate::formula::language::{Language, Normal, Simple, Term};

pub struct ImpFalse;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Variants<S> {
    Implication([S; 2]),
    False,
}

impl Language for ImpFalse {
    type Variant<S> = Variants<S>
    where
        S: Simple;

    fn matches<S: Simple>(this: &Self::Variant<S>, other: &Self::Variant<S>) -> bool {
        mem::discriminant(this) == mem::discriminant(other)
    }

    fn children<S: Simple>(this: &Self::Variant<S>) -> &[S] {
        match this {
            Variants::Implication(x) => x,
            Variants::False => &[],
        }
    }

    fn match_implication<S: Simple>(this: &Self::Variant<S>) -> Option<&[S; 2]> {
        match this {
            Variants::Implication(x) => Some(x),
            Variants::False => None,
        }
    }

    fn map<S: Simple, T: Simple, F: FnMut(&S) -> T>(
        this: &Self::Variant<S>,
        mut f: F,
    ) -> Self::Variant<T> {
        match this {
            Variants::Implication([a, b]) => Variants::Implication([f(a), f(b)]),
            Variants::False => Variants::False,
        }
    }
}

impl Display for Variants<()> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Variants::Implication(_) => write!(f, "C"),
            Variants::False => write!(f, "F"),
        }
    }
}

impl TryFrom<char> for Variants<()> {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'C' => Ok(Variants::Implication([(), ()])),
            'F' => Ok(Variants::False),
            _ => Err(()),
        }
    }
}

impl ImpFalse {
    pub fn church() -> Box<[Normal<ImpFalse>]> {
        [
            // a -> (b -> a)
            "CaCba".parse().unwrap(),
            // (a -> (b -> c)) -> ((a -> b) -> (a -> c))
            "CCaCbcCCabCac".parse().unwrap(),
            // ((a -> F) -> F) -> a
            "CCCaFFa".parse().unwrap(),
        ]
        .into()
    }

    pub fn meredith1() -> Box<[Normal<ImpFalse>]> {
        ["CCCCCabCcFdeCCeaCca".parse().unwrap()].into()
    }
}
