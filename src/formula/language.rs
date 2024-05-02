use std::{fmt::Display, hash::Hash, mem::MaybeUninit};

pub trait Language: PartialEq + Eq + Clone + Hash {
    type Variant<S>: PartialEq + Eq + Clone + Hash;

    fn matches<S>(this: &Self::Variant<S>, other: &Self::Variant<S>) -> bool;

    fn children<S>(this: &Self::Variant<S>) -> &[S];

    fn match_implication<S>(this: &Self::Variant<S>) -> Option<&[S; 2]>;

    fn map<S, T, F: FnMut(S) -> T>(this: &Self::Variant<S>, f: F) -> Self::Variant<T>;
}

#[derive(PartialEq, Eq, Clone, Hash)]
pub enum Term<L: Language, S> {
    Term(L::Variant<S>),
    Var(usize),
}

pub struct Normal<L: Language>(Box<[Term<L, ()>]>);

impl<L: Language> Display for Normal<L>
where
    L::Variant<()>: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn inner<L: Language>(
            val: &[Term<L, ()>],
            f: &mut std::fmt::Formatter<'_>,
        ) -> Result<usize, std::fmt::Error>
        where
            L::Variant<()>: Display,
        {
            match &val[0] {
                &Term::Var(x) => {
                    write!(f, "{x}")?;
                    Ok(1)
                }
                Term::Term(t) => {
                    write!(f, "({t}")?;
                    let mut idx = 1;
                    for () in L::children(&t) {
                        idx = inner(&val[idx..], f)?;
                        write!(f, " ")?;
                    }
                    write!(f, ")")?;
                    Ok(idx)
                }
            }
        }
        inner(&self.0, f)?;
        Ok(())
    }
}

impl<L: Language> Normal<L> {
    fn normalize_vars(&mut self) {
        let mut current_var = 1;
        for i in 0..self.0.len() {
            if let Term::Var(new_var) = self.0[i] {
                if current_var < new_var {
                    for elem in self.0[i..].iter_mut() {
                        if let Term::Var(x) = elem {
                            if *x == current_var {
                                *x = new_var;
                            } else if *x == new_var {
                                *x = current_var;
                            }
                        }
                    }
                    current_var += 1;
                } else if current_var == new_var {
                    current_var += 1;
                }
            }
        }
    }

    pub fn from_arena(arena: &Arena<L>, idx: usize) -> Self {
        let mut v = Vec::new();
        fn inner<L: Language>(v: &mut Vec<Term<L, ()>>, arena: &Arena<L>, idx: usize) {
            match &arena.0[idx] {
                &Term::Var(x) => v.push(Term::Var(x)),
                Term::Term(t) => {
                    v.push(Term::Term(L::map(&t, |_: usize| ())));
                    for &c in L::children(&t) {
                        inner(v, arena, c);
                    }
                }
            }
        }
        inner(&mut v, arena, idx);
        let mut result = Self(v.into());
        result.normalize_vars();
        result
    }

    pub fn write_into<'a>(
        &self,
        arena: &'a mut [MaybeUninit<Term<L, usize>>],
    ) -> &'a mut [Term<L, usize>] {
        // recursively writes `val` into `arena`, adding `offset` to each index
        // returns the number of `Term`s written
        fn inner<L: Language>(
            val: &[Term<L, ()>],
            arena: &mut [MaybeUninit<Term<L, usize>>],
            offset: usize,
        ) -> usize {
            match &val[0] {
                &Term::Var(x) => {
                    arena[0].write(Term::Var(x));
                    return 1;
                }
                Term::Term(t) => {
                    let mut index = 1;
                    let t_new = L::map(t, |()| {
                        let w = inner(&val[index..], &mut arena[index..], offset + index);
                        let last_index = index;
                        index += w;
                        offset + last_index
                    });
                    arena[0].write(Term::Term(t_new));
                    return index;
                }
            }
        }
        let written = inner(&self.0, arena, 0);
        let (init, _) = arena.split_at_mut(written);
        unsafe { MaybeUninit::slice_assume_init_mut(init) }
    }
}

pub struct Arena<L: Language>(Box<[Term<L, usize>]>);

impl<L: Language> Arena<L> {
    fn substitute(&mut self, var: usize, term: &Term<L, usize>) {
        self.0.iter_mut().for_each(|t| {
            if *t == Term::Var(var) {
                *t = term.clone();
            }
        })
    }
}

fn occurs<L: Language>(arena: &Arena<L>, var: usize, term: &Term<L, usize>) -> bool {
    match term {
        Term::Var(x) => *x == var,
        Term::Term(t) => L::children(t)
            .iter()
            .any(|&idx| occurs(arena, var, &arena.0[idx])),
    }
}

fn unify_many<L: Language>(
    arena: &mut Arena<L>,
    mut eqs: Vec<(usize, usize)>,
) -> Option<Vec<(usize, Term<L, usize>)>> {
    let mut solution = Vec::new();
    while let Some((a, b)) = eqs.pop() {
        match (&arena.0[a], &arena.0[b]) {
            (&Term::Var(x), t @ &Term::Var(y)) => {
                if x != y {
                    let t = t.clone();
                    arena.substitute(x, &t);
                    solution.push((x, t.clone()));
                }
            }
            (&Term::Var(x), t @ &Term::Term(_)) | (t @ &Term::Term(_), &Term::Var(x)) => {
                let t = t.clone();
                if occurs(arena, x, &t) {
                    return None;
                }
                arena.substitute(x, &t);
                solution.push((x, t));
            }
            (Term::Term(t1), Term::Term(t2)) => {
                if !L::matches(t1, t2) {
                    return None;
                }
                L::children(t1)
                    .iter()
                    .zip(L::children(t2))
                    .for_each(|(&a, &b)| {
                        eqs.push((a, b));
                    })
            }
        }
    }
    return Some(solution);
}
