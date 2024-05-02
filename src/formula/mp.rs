use super::{Arena, Entry, Idx, Normalized, NormalEntry};

pub fn modus_ponens(p: &Normalized, f: &Normalized) -> Option<Normalized> {
    if f.elems[0] == NormalEntry::Implication {
        let mut arena = Arena(Vec::with_capacity(p.elems.len() + f.elems.len()));
        p.write_into(&mut arena);
        make_disjoint(&mut arena, f);

        let p1idx = Idx(0);
        let fidx = Idx(arena.0.len());
        f.write_into(&mut arena);
        let Entry::Implication(p2idx, qidx) = arena[fidx] else {
            panic!("no longer an implication?");
        };
        if let Some(_) = unify::unify(&mut arena, p1idx, p2idx) {
            return Some(Normalized::from_arena(&arena, qidx));
        }
    }
    None
}

fn make_disjoint(arena: &mut Arena, f: &Normalized) {
    let max_var = f
        .elems
        .iter()
        .filter_map(|e| {
            if let NormalEntry::Var(x) = e {
                Some(*x)
            } else {
                None
            }
        })
        .max()
        .unwrap_or(0);
    arena.0.iter_mut().for_each(|e| {
        if let Entry::Var(x) = e {
            *x += max_var + 1;
        }
    })
}

mod unify {

    use super::{Arena, Entry, Idx};

    pub fn unify(arena: &mut Arena, a: Idx, b: Idx) -> Option<Vec<(usize, Entry)>> {
        unify_many(arena, vec![(a, b)])
    }

    fn occurs(arena: &Arena, var: usize, formula: &Entry) -> bool {
        match formula {
            Entry::Var(x) => var == *x,
            &Entry::Implication(a, b) => {
                occurs(arena, var, &arena[a]) | occurs(arena, var, &arena[b])
            }
            &Entry::Not(a) => occurs(arena, var, &arena[a]),
        }
    }

    fn unify_many(arena: &mut Arena, mut eqs: Vec<(Idx, Idx)>) -> Option<Vec<(usize, Entry)>> {
        let mut solution: Vec<(usize, Entry)> = Vec::new();
        while let Some((a, b)) = eqs.pop() {
            match (&arena[a], &arena[b]) {
                (Entry::Implication(_, _), Entry::Not(_))
                | (Entry::Not(_), Entry::Implication(_, _)) => {
                    return None;
                }
                (Entry::Implication(a1, b1), Entry::Implication(a2, b2)) => {
                    eqs.push((*a1, *a2));
                    eqs.push((*b1, *b2));
                }
                (Entry::Not(a1), Entry::Not(a2)) => {
                    eqs.push((*a1, *a2));
                }
                (t, &Entry::Var(x)) | (&Entry::Var(x), t) => {
                    let t = t.clone();
                    if occurs(&arena, x, &t) {
                        return None;
                    }
                    arena.substitute(x, &t);
                    solution.push((x, t.clone()))
                }
            }
        }
        Some(solution)
    }
}

#[cfg(test)]
mod test {
    use crate::formula::{modus_ponens, Normalized};

    #[test]
    fn mp() {
        let p = Normalized::from_formula(&"1 -> --1".parse().unwrap());
        let q = Normalized::from_formula(&"1 -> 2 -> 1".parse().unwrap());
        let f = modus_ponens(&p, &q).unwrap();
        assert_eq!(f, Normalized::from_formula(&"2 -> 4 -> --4".parse().unwrap()));
    }

    #[test]
    fn eq() {
        let p = Normalized::from_formula(&"1 -> 2 -> 1".parse().unwrap());
        let q = Normalized::from_formula(&"1 -> 2 -> 3 -> 2".parse().unwrap());
        assert_eq!(
            modus_ponens(&p, &q).unwrap(),
            Normalized::from_formula(&"1 -> 2 -> 1".parse().unwrap())
        )
    }
}
