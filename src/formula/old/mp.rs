use std::cmp::max;

use crate::formula::Formula;

/// Applies modus ponens to the formulas p and (a -> b)
/// by attempting to unify p and a
pub fn try_apply_mp(p: &Formula, a: &Formula, b: &Formula) -> Option<Formula> {
    let mut p_disj = p.clone();
    make_disjoint(&mut p_disj, a, b);
    if let Some(subst) = try_unify(p_disj, a.clone()) {
        let mut b_new = b.clone();
        b_new.subst_many(&subst);
        Some(b_new)
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

fn make_disjoint(p: &mut Formula, a: &Formula, b: &Formula) {
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

    let m = max(max_var(a), max_var(b));
    add_to_var(p, m);
}

fn try_unify(a: Formula, b: Formula) -> Option<Vec<(usize, Formula)>> {
    try_unify_many(vec![(a, b)])
}

fn try_unify_many(mut eqs: Vec<(Formula, Formula)>) -> Option<Vec<(usize, Formula)>> {
    let mut solution: Vec<(usize, Formula)> = Vec::new();
    while let Some(eq) = eqs.pop() {
        match eq {
            (t, Formula::Var(x)) | (Formula::Var(x), t) => {
                if occurs(x, &t) {
                    return None;
                } else {
                    let s = (x, t);
                    for (a, b) in eqs.iter_mut() {
                        a.subst(&s);
                        b.subst(&s);
                    }
                    for (_, a) in solution.iter_mut() {
                        a.subst(&s);
                    }
                    solution.push(s);
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
    use crate::{
        formula::Formula,
        mp::{make_disjoint, try_unify},
        try_apply_mp,
    };

    #[test]
    fn disjoint() {
        let mut p1 = "1 -> --1".parse().unwrap();
        let p2 = "1".parse().unwrap();
        let q = "2 -> 1".parse().unwrap();
        make_disjoint(&mut p1, &p2, &q);
        assert_eq!(p1, "3 -> --3".parse().unwrap());
    }

    fn test_unify(p1: Formula, p2: Formula, s: Vec<(usize, Formula)>) {
        assert_eq!(try_unify(p1, p2), Some(s))
    }

    #[test]
    fn unify() {
        let test_cases = [("3 -> --3", "1", &[(1, "3 -> --3")])];
    }

    #[test]
    fn substitute() {
        let mut p: Formula = "2 -> 1".parse().unwrap();
        let a: Formula = "3 -> --3".parse().unwrap();
        let b: Formula = "2 -> 3 -> --3".parse().unwrap();
        p.subst(&(1, a));
        assert_eq!(p, b);
    }

    #[test]
    fn mp() {
        let p1 = "1 -> --1".parse().unwrap();
        let p2 = "1".parse().unwrap();
        let q = "2 -> 1".parse().unwrap();
        assert_eq!(
            try_apply_mp(&p1, &p2, &q),
            Some("2 -> 3 -> --3".parse().unwrap())
        )
    }
}
