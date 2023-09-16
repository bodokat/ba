use std::{fmt::Display, str::FromStr};

use self::parse::FormulaParseError;

mod arena;
mod parse;
mod v2;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Formula {
    Var(usize),
    Implication(Box<Formula>, Box<Formula>),
    Not(Box<Formula>),
}

impl FromStr for Formula {
    type Err = FormulaParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse::parse(s)
    }
}

impl Display for Formula {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Formula::Var(x) => write!(f, "{x}"),
            Formula::Implication(a, b) => {
                if a.is_implication() {
                    write!(f, "({a}) -> {b}")
                } else {
                    write!(f, "{a} -> {b}")
                }
            }
            Formula::Not(a) => {
                if a.is_implication() {
                    write!(f, "-({a})")
                } else {
                    write!(f, "-{a}")
                }
            }
        }
    }
}

pub fn implication(a: Formula, b: Formula) -> Formula {
    Formula::Implication(Box::new(a), Box::new(b))
}
pub fn not(a: Formula) -> Formula {
    Formula::Not(Box::new(a))
}

impl Formula {
    fn is_implication(&self) -> bool {
        matches!(self, Self::Implication(_, _))
    }

    pub fn subst_many(&mut self, s: &Vec<(usize, Formula)>) {
        match self {
            Formula::Var(v) => {
                if let Some((_, sub)) = s.iter().find(|(x, _)| x == v) {
                    *self = sub.clone()
                }
            }
            Formula::Implication(a, b) => {
                a.subst_many(s);
                b.subst_many(s);
            }
            Formula::Not(a) => {
                a.subst_many(s);
            }
        }
    }

    pub fn subst(&mut self, s @ (x, t): &(usize, Formula)) {
        match self {
            Formula::Var(v) => {
                if v == x {
                    *self = t.clone()
                }
            }
            Formula::Implication(a, b) => {
                a.subst(s);
                b.subst(s);
            }
            Formula::Not(a) => {
                a.subst(s);
            }
        }
    }
}
