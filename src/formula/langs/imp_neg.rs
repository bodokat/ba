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
            Variants::Implication(_) => write!(f, "->"),
            Variants::Negation(_) => write!(f, "~"),
        }
    }
}

impl ImpNeg {
    pub fn frege_axioms() -> Box<[Normal<ImpNeg>]> {
        use Term::Var as V;
        use Variants::*;
        fn i() -> Term<ImpNeg, ()> {
            Term::Term(Implication([(), ()]))
        }
        fn n() -> Term<ImpNeg, ()> {
            Term::Term(Negation([()]))
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
            // (0 -> (1 -> 2)) -> (1 -> (0 -> 2))
            [i(), i(), V(0), i(), V(1), V(2), i(), V(1), i(), V(0), V(2)].into(),
            // (0 -> 1) -> (-1 -> -0)
            [i(), i(), V(0), V(1), i(), n(), V(1), n(), V(0)].into(),
            // (--0 -> 0)
            [i(), n(), n(), V(0), V(0)].into(),
            // (0 -> --0)
            [i(), V(0), n(), n(), V(0)].into(),
        ]
        .into()
    }

    pub fn lukasiewicz_tarski() -> Vec<Normal<ImpNeg>> {
        use Term::Var as V;
        use Variants::*;
        fn i() -> Term<ImpNeg, ()> {
            Term::Term(Implication([(), ()]))
        }
        fn n() -> Term<ImpNeg, ()> {
            Term::Term(Negation([()]))
        }
        [
            /*
               (
                   (0 -> (1 -> 0))
                   ->
                   (
                       (
                           (-2 -> (3 -> -4))
                           ->
                           (
                               (2 -> (3 -> 5))
                               ->
                               ((4 -> 3) -> (4 -> 5)))
                           )
                       ->
                       6
                       )
                   )
               ->
               (7 -> 6)
            */
            [
                i(),
                i(),
                i(),
                V(0),
                i(),
                V(1),
                V(0),
                i(),
                i(),
                i(),
                n(),
                V(2),
                i(),
                V(3),
                n(),
                V(4),
                i(),
                i(),
                V(2),
                i(),
                V(3),
                V(5),
                i(),
                i(),
                V(4),
                V(3),
                i(),
                V(4),
                V(5),
                V(6),
                i(),
                V(7),
                V(6),
            ]
            .into(),
        ]
        .into()
    }

    pub fn meredith() -> Vec<Normal<ImpNeg>> {
        use Term::Var as V;
        use Variants::*;
        fn i() -> Term<ImpNeg, ()> {
            Term::Term(Implication([(), ()]))
        }
        fn n() -> Term<ImpNeg, ()> {
            Term::Term(Negation([()]))
        }
        [[
            i(),
            i(),
            i(),
            i(),
            i(),
            V(0),
            V(1),
            i(),
            n(),
            V(2),
            n(),
            V(3),
            V(2),
            V(4),
            i(),
            i(),
            V(4),
            V(0),
            i(),
            V(3),
            V(0),
        ]
        .into()]
        .into()
    }
}
