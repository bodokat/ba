use std::{fmt::Display, mem};

use super::language::{Language, Normal, Term};

#[derive(PartialEq, Eq, Clone, Hash)]
enum ImpNegEnum<S> {
    Implication([S; 2]),
    Negation([S; 1]),
}

#[derive(PartialEq, Eq, Clone, Hash)]
pub struct ImpNeg;

impl Language for ImpNeg {
    type Variant<S> = ImpNegEnum<S>;

    fn matches<S>(this: &Self::Variant<S>, other: &Self::Variant<S>) -> bool {
        mem::discriminant(this) == mem::discriminant(other)
    }

    fn children<S>(this: &Self::Variant<S>) -> &[S] {
        match this {
            ImpNegEnum::Implication(x) => x,
            ImpNegEnum::Negation(x) => x,
        }
    }

    fn match_implication<S>(this: &Self::Variant<S>) -> Option<&[S; 2]> {
        match this {
            ImpNegEnum::Implication(x) => Some(x),
            ImpNegEnum::Negation(_) => None,
        }
    }

    fn map<S, T, F: FnMut(S) -> T>(this: &Self::Variant<S>, f: F) -> Self::Variant<T> {
        match this {
            ImpNegEnum::Implication([a, b]) => ImpNegEnum::Implication([f(a), f(b)]),
            ImpNegEnum::Negation([a]) => ImpNegEnum::Negation([f(a)]),
        }
    }
}

impl Display for ImpNegEnum<()> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImpNegEnum::Implication(_) => "->",
            ImpNegEnum::Negation(_) => "~",
        }
    }
}

pub fn frege_axioms() -> Vec<Normal<ImpNeg>> {
    use ImpNegEnum::{Implication as I, Negation as N};
    use Term::{Term as T, Var as V};
    [
        [T(I(())), V(0), T(I(())), V(1), V(0)].into(),
        [
            T(I(())),
            T(I(())),
            V(0),
            T(I(())),
            V(1),
            V(2),
            T(I(())),
            T(I(())),
            V(0),
            V(1),
            T(I(())),
            V(0),
            V(2),
        ]
        .into(),
        [
            T(I(())),
            T(I(())),
            V(0),
            T(I(())),
            V(1),
            V(2),
            T(I(())),
            V(1),
            T(I(())),
            V(0),
            V(2),
        ]
        .into(),
        [
            T(I(())),
            T(I(())),
            V(0),
            V(1),
            T(I(())),
            T(N(())),
            V(1),
            T(N(())),
            V(0),
        ]
        .into(),
        [T(I(())), T(N(())), T(N(())), V(0), V(0)].into(),
        [T(I(())), V(0), T(N(())), T(N(())), V(0)].into(),
    ]
}
