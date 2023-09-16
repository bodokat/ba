use typed_arena::Arena;

#[derive(Debug, Clone)]
enum Formula<'a> {
    Var(usize),
    Implication(&'a Formula<'a>, &'a Formula<'a>),
    Not(&'a Formula<'a>),
}

fn unify_many<'a>(
    mut eqs: Vec<(&'a mut Formula<'a>, &'a mut Formula<'a>)>,
) -> Option<Vec<(usize, &'a Formula<'a>)>> {
    use Formula::{Implication, Not, Var};
    let mut solution: Vec<(usize, &mut Formula<'_>)> = Vec::new();
    while let Some((a, b)) = eqs.pop() {
        match (*a, *b) {
            (Implication(_, _), Not(_)) | (Not(_), Implication(_, _)) => return None,
            (Implication(a1, b1), Implication(a2, b2)) => {
                eqs.push((a1, a2));
                eqs.push((b1, b2));
            }
            (Not(a1), Not(a2)) => {
                eqs.push((a1, a2));
            }
            (t, Var(x)) | (Var(x), t) => {
                if occurs(x, t) {
                    return None;
                }
                for (f1, f2) in eqs.iter_mut() {
                    f1.substitute(x, t);
                    f2.substitute(x, t);
                }
                solution.push((x, t));
            }
        }
    }
    Some(solution)
}

fn occurs(x: usize, t: &Formula<'_>) -> bool {
    match *t {
        Formula::Var(y) => x == y,
        Formula::Implication(a, b) => occurs(x, a) || occurs(x, b),
        Formula::Not(a) => occurs(x, a),
    }
}

#[cfg(test)]
mod test {
    use typed_arena::Arena;

    #[test]
    fn basic() {
        let arena = Arena::new();
    }
}
