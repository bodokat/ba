use super::{Arena, Entry, Formula, Idx, Normal, NormalEntry};

pub fn modus_ponens(p: &Normal, f: &Normal) -> Option<Normal> {
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
            return Some(Normal::from_arena(&arena, qidx));
        }
    }
    None
}

fn clone_children_to(from: &Arena, from_e: &Entry, to: &mut Arena) -> Entry {
    match from_e {
        e @ Entry::Var(_) => e.clone(),
        Entry::Implication(a, b) => {
            let a1 = Idx(to.0.len());
            clone_to(from, *a, to);
            let b1 = Idx(to.0.len());
            clone_to(from, *b, to);
            Entry::Implication(a1, b1)
        }
        Entry::Not(a) => {
            let a1 = Idx(to.0.len());
            clone_to(from, *a, to);
            Entry::Not(a1)
        }
    }
}

fn clone_to(from: &Arena, from_i: Idx, to: &mut Arena) {
    match &from[from_i] {
        e @ Entry::Var(_) => to.0.push(e.clone()),
        e @ Entry::Implication(_, _) => {
            let i = Idx(to.0.len());
            to.0.push(e.clone()); //Sentinel value
            let new_e = clone_children_to(from, e, to);
            to[i] = new_e;
        }
        e @ Entry::Not(_) => {
            let i = Idx(to.0.len());
            to.0.push(e.clone()); //Sentinel value
            let new_e = clone_children_to(from, e, to);
            to[i] = new_e;
        }
    }
}

fn substitute_many_to(from: &Arena, to: &mut Formula, mut s: Vec<(usize, Entry)>) {
    s.iter_mut().for_each(|(_, e)| {
        let new_entry = clone_children_to(from, &e, &mut to.arena);
        *e = new_entry;
    });
    to.arena.0.iter_mut().for_each(|entry| {
        if let Entry::Var(v) = entry {
            if let Some((_, r)) = s.iter().find(|(var, _)| var == v) {
                *entry = r.clone();
            }
        }
    })
}

fn make_disjoint(arena: &mut Arena, f: &Normal) {
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
    use crate::formula::{modus_ponens, Normal};

    #[test]
    fn mp() {
        let p = Normal::from_formula(&"1 -> --1".parse().unwrap());
        let q = Normal::from_formula(&"1 -> 2 -> 1".parse().unwrap());
        let f = modus_ponens(&p, &q).unwrap();
        assert_eq!(f, Normal::from_formula(&"2 -> 4 -> --4".parse().unwrap()));
    }

    #[test]
    fn eq() {
        let p = Normal::from_formula(&"1 -> 2 -> 1".parse().unwrap());
        let q = Normal::from_formula(&"1 -> 2 -> 3 -> 2".parse().unwrap());
        assert_eq!(
            modus_ponens(&p, &q).unwrap(),
            Normal::from_formula(&"1 -> 2 -> 1".parse().unwrap())
        )
    }
}
