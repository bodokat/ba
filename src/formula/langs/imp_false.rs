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
            Variants::Implication(_) => write!(f, "->"),
            Variants::False => write!(f, "!"),
        }
    }
}

impl ImpFalse {
    pub fn church() -> Box<[Normal<ImpFalse>]> {
        use Term::Var as V;
        fn i() -> Term<ImpFalse, ()> {
            Term::Term(Variants::Implication([(), ()]))
        }
        fn f() -> Term<ImpFalse, ()> {
            Term::Term(Variants::False)
        }
        [
            // 0 -> (1 -> 0)
            [i(), V(0), i(), V(1), V(0)].into(),
            // (0 -> (1 -> 2)) -> ((0 -> 1) -> (0 -> 2))
            [
                i(),
                i(),
                V(0),
                i(),
                V(1),
                V(2),
                i(),
                i(),
                V(0),
                V(1),
                i(),
                V(0),
                V(2),
            ]
            .into(),
            // ((0 -> F) -> F) -> 0
            [i(), i(), i(), V(0), f(), f(), V(0)].into(),
        ]
        .into()
    }
}
