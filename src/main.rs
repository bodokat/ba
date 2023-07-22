use std::{cmp::max, str::FromStr};

use formula::Formula;

mod formula;

pub static AXIOMS: &[&str] = &[
    "1 -> (2 -> 1)",
    "(1 -> 2 -> 3) -> (1 -> 2) -> (1 -> 3)",
    "(1 -> 2 -> 3) -> (2 -> 1 -> 3)",
    "(1 -> 2) -> (-2 -> -1)",
    "--1 -> 1",
    "1 -> --1",
];

fn main() {
    let context = Context::default();

    let new = context.apply_mp_all();
    println!("{:?}", context.formulas);
    for f in new {
        println!("{f}");
    }
}

struct Context {
    formulas: Vec<Formula>,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            formulas: AXIOMS
                .iter()
                .map(|s| Formula::from_str(s).unwrap())
                .collect(),
        }
    }
}

impl Context {
    fn apply_mp_all(&self) -> Vec<Formula> {
        let mut result = Vec::new();

        for f in self.formulas.iter() {
            if let Formula::Implication(a, b) = f {
                for p in self.formulas.iter() {
                    if let Some(new) = try_apply_mp(p, a, b) {
                        result.push(new);
                    }
                }
            }
        }
        result
    }
}

/// Applies modus ponens to the formulas p and (a -> b)
/// by attempting to unify p and a
fn try_apply_mp(p: &Formula, a: &Formula, b: &Formula) -> Option<Formula> {
    let mut p_disj = p.clone();
    make_disjoint(&mut p_disj, a);
    if let Some(subst) = try_unify(p, a) {
        Some(b.subst(&subst))
    } else {
        None
    }
}

fn occurs(x: usize, t: &Formula) -> bool {
    match t {
        Formula::Var(y) => x == *y,
        Formula::Implication(a, b) => occurs(x, a) || occurs(x, b),
        Formula::Not(a) => occurs(x, a),
    }
}

fn make_disjoint(a: &mut Formula, b: &Formula) {
    fn max_var(f: &Formula) -> usize {
        match f {
            Formula::Var(x) => *x,
            Formula::Implication(a, b) => max(max_var(a), max_var(b)),
            Formula::Not(a) => max_var(a),
        }
    }
    fn add_to_var(f: &mut Formula, n: usize) {
        match f {
            Formula::Var(x) => *x += n,
            Formula::Implication(a, b) => {
                add_to_var(a, n);
                add_to_var(b, n);
            }
            Formula::Not(a) => add_to_var(a, n),
        }
    }

    let m = max_var(b);
    add_to_var(a, m);
}

fn try_unify(a: &Formula, b: &Formula) -> Option<Vec<(usize, Formula)>> {
    try_unify_many(vec![(a.clone(), b.clone())])
}

fn try_unify_many(mut eqs: Vec<(Formula, Formula)>) -> Option<Vec<(usize, Formula)>> {
    let mut solution = Vec::new();
    while let Some(eq) = eqs.pop() {
        match eq {
            (t, Formula::Var(x)) | (Formula::Var(x), t)   => {
                if occurs(x, &t) {
                    return None;
                } else {
                    solution.push((x, t))
                }
            }
            (Formula::Implication(_, _), Formula::Not(_))
            | (Formula::Not(_), Formula::Implication(_, _)) => return None,
            (Formula::Implication(a1, b1), Formula::Implication(a2, b2)) => {
                eqs.extend_from_slice(&[(*a1, *a2), (*b1, *b2)])
            }
            (Formula::Not(a), Formula::Not(b)) => eqs.push((*a, *b)),
        }
    }
    Some(solution)
}


#[cfg(test)]
mod test {
    use crate::try_apply_mp;


    #[test]
    fn mp() {
        let p1 = "1".parse().unwrap();
        let p2 = "2".parse().unwrap();
        let q = "1 -> 3".parse().unwrap();
        assert_eq!(try_apply_mp(&p1, &p2, &q), Some(q))
    }
}