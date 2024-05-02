use std::{fmt::Display, hash::Hash, mem};

use super::language::{Language, Normal, Term};

#[derive(PartialEq, Eq, Clone, Hash)]
enum ImpNegEnum<S> {
    Implication([S; 2]),
    Negation([S; 1]),
}

#[derive(PartialEq, Eq, Clone, Hash)]
pub struct ImpNeg;

impl Language for ImpNeg {
    type Variant<S> = ImpNegEnum<S>
        where S: Hash + Clone;

    fn matches<S: Hash + Clone>(this: &Self::Variant<S>, other: &Self::Variant<S>) -> bool {
        mem::discriminant(this) == mem::discriminant(other)
    }

    fn children<S: Hash + Clone>(this: &Self::Variant<S>) -> &[S] {
        match this {
            ImpNegEnum::Implication(x) => x,
            ImpNegEnum::Negation(x) => x,
        }
    }

    fn match_implication<S: Hash + Clone>(this: &Self::Variant<S>) -> Option<&[S; 2]> {
        match this {
            ImpNegEnum::Implication(x) => Some(x),
            ImpNegEnum::Negation(_) => None,
        }
    }

    fn map<S: Hash + Clone, T: Hash + Clone, F: FnMut(&S) -> T>(
        this: &Self::Variant<S>,
        mut f: F,
    ) -> Self::Variant<T> {
        match this {
            ImpNegEnum::Implication([a, b]) => ImpNegEnum::Implication([f(a), f(b)]),
            ImpNegEnum::Negation([a]) => ImpNegEnum::Negation([f(a)]),
        }
    }
}

impl Display for ImpNegEnum<()> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImpNegEnum::Implication(_) => write!(f, "->"),
            ImpNegEnum::Negation(_) => write!(f, "~"),
        }
    }
}

pub fn frege_axioms() -> Vec<Normal<ImpNeg>> {
    use ImpNegEnum::*;
    use Term::Var as V;
    fn i() -> Term<ImpNeg, ()> {
        Term::Term(Implication([(), ()]))
    }
    fn n() -> Term<ImpNeg, ()> {
        Term::Term(Negation([()]))
    }
    [
        [i(), V(0), i(), V(1), V(0)].into(),
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
        [i(), i(), V(0), i(), V(1), V(2), i(), V(1), i(), V(0), V(2)].into(),
        [i(), i(), V(0), V(1), i(), n(), V(1), n(), V(0)].into(),
        [i(), n(), n(), V(0), V(0)].into(),
        [i(), V(0), n(), n(), V(0)].into(),
    ]
    .into()
}
