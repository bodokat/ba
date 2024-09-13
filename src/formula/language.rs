use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    hash::Hash,
    str::FromStr,
};

pub trait Simple: Clone + Hash + PartialEq + Eq + Send + Sync + Debug {}

impl<T> Simple for T where T: Clone + Hash + PartialEq + Eq + Send + Sync + Debug {}

pub trait Language: 'static {
    type Variant<S>: Simple
    where
        S: Simple;

    fn matches<S: Simple>(this: &Self::Variant<S>, other: &Self::Variant<S>) -> bool;

    fn children<S: Simple>(this: &Self::Variant<S>) -> &[S];

    fn match_implication<S: Simple>(this: &Self::Variant<S>) -> Option<&[S; 2]>;

    fn map<S: Simple, T: Simple, F: FnMut(&S) -> T>(
        this: &Self::Variant<S>,
        f: F,
    ) -> Self::Variant<T>;
}

pub enum Term<L: Language, S: Simple> {
    Term(L::Variant<S>),
    Var(u16),
}

impl<L: Language, S: Simple> Debug for Term<L, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Term(arg0) => f.debug_tuple("Term").field(arg0).finish(),
            Self::Var(arg0) => f.debug_tuple("Var").field(arg0).finish(),
        }
    }
}

impl<L: Language, S: Simple> Eq for Term<L, S> {}

impl<L: Language, S: Simple> Clone for Term<L, S> {
    fn clone(&self) -> Self {
        match self {
            Self::Term(x) => Self::Term(x.clone()),
            Self::Var(x) => Self::Var(*x),
        }
    }
}

impl<L: Language, S: Simple> Hash for Term<L, S> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
        match self {
            Term::Term(x) => x.hash(state),
            Term::Var(x) => x.hash(state),
        }
    }
}

impl<L: Language, S: Simple> PartialEq for Term<L, S> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Term(l0), Self::Term(r0)) => l0 == r0,
            (Self::Var(l0), Self::Var(r0)) => l0 == r0,
            _ => false,
        }
    }
}

pub struct Normal<L: Language>(Box<[Term<L, ()>]>);

impl<L: Language> Debug for Normal<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Normal").field(&self.0).finish()
    }
}

impl<L: Language> Hash for Normal<L> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<L: Language> Eq for Normal<L> {}

impl<L: Language> PartialEq for Normal<L> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<L: Language> Clone for Normal<L> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<L: Language> Display for Normal<L>
where
    L::Variant<()>: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for t in &self.0 {
            match t {
                Term::Term(v) => write!(f, "{v}")?,
                Term::Var(x) => write!(f, "{x}")?,
            }
        }
        Ok(())
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum ParseError {
    WrongArguments(isize),
}

impl<L: Language> FromStr for Normal<L>
where
    L::Variant<()>: TryFrom<char>,
{
    type Err = ParseError;

    #[allow(clippy::cast_possible_wrap)]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut num_left: isize = 1;
        let mut next_var = 0;
        let mut vars_map = HashMap::new();
        let f: Normal<_> = s
            .chars()
            .map(|c| {
                num_left -= 1;
                if let Ok(v) = c.try_into() {
                    num_left += L::children(&v).len() as isize;
                    Term::Term(v)
                } else {
                    let e = vars_map.entry(c);
                    let n = e.or_insert_with(|| {
                        let n = next_var;
                        next_var += 1;
                        n
                    });
                    Term::Var(*n)
                }
            })
            .collect::<Box<[_]>>()
            .into();
        if num_left == 0 {
            Ok(f)
        } else {
            Err(ParseError::WrongArguments(num_left))
        }
    }
}

impl<L: Language> Normal<L> {
    fn normalize_vars(&mut self) {
        let mut current_var = 0;
        for i in 0..self.0.len() {
            if let Term::Var(new_var) = self.0[i] {
                if current_var < new_var {
                    for elem in &mut self.0[i..] {
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

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn from_arena(arena: &Arena<L>, idx: usize) -> Self {
        fn inner<L: Language>(v: &mut Vec<Term<L, ()>>, arena: &Arena<L>, idx: usize) {
            match &arena.0[idx] {
                &Term::Var(x) => v.push(Term::Var(x)),
                Term::Term(t) => {
                    v.push(Term::Term(L::map(t, |_| ())));
                    for &c in L::children(t) {
                        inner(v, arena, c as usize);
                    }
                }
            }
        }
        let mut v = Vec::new();
        inner(&mut v, arena, idx);
        let mut result = Self(v.into());
        result.normalize_vars();
        result
    }

    // recursively writes `val` into `arena`, adding `offset` to each index
    // returns the number of `Term`s written
    fn write_with_offset(
        val: &[Term<L, ()>],
        arena: &mut Vec<Term<L, u16>>,
        offset: u16,
        var_increment: u16,
    ) -> u16 {
        match &val[0] {
            &Term::Var(x) => {
                arena.push(Term::Var(x + var_increment));
                1
            }
            Term::Term(t) => {
                let mut index = 1u16;
                let t_idx = arena.len();
                arena.push(Term::Var(0)); // Sentinel
                let t_new = L::map(t, |()| {
                    let w = Self::write_with_offset(
                        &val[(index as usize)..],
                        arena,
                        offset + index,
                        var_increment,
                    );
                    let last_index = index;
                    index += w;
                    offset + last_index
                });
                arena[t_idx] = Term::Term(t_new);
                index
            }
        }
    }

    pub fn write_into(&self, arena: &mut Vec<Term<L, u16>>, var_increment: u16) -> u16 {
        let start = u16::try_from(arena.len()).unwrap();

        Self::write_with_offset(&self.0, arena, start, var_increment);

        start
    }
}

// fn slice<L: Language>(formula: &[Term<L, ()>]) -> &[Term<L,()>] {
//     let mut next_var = 0;
//     let mut remaining = &[];
//     for index in (0..(formula.len() / 2)).map(|x| 2 * x) {
//         match (formula[index], formula[index + 1]) {
//             (Term::Term(t), Term::Var(v))
//                 if L::match_implication(&t).is_some() && v == next_var =>
//             {
//                 next_var += 1;
//             }
//             _ => remaining = formula.split_at(index * 2),
//         }
//     }

//     unreachable!("not a well-formed term")
// }

impl<L: Language, const N: usize> From<[Term<L, ()>; N]> for Normal<L> {
    fn from(value: [Term<L, ()>; N]) -> Self {
        Self(Box::new(value))
    }
}

impl<L: Language> From<Box<[Term<L, ()>]>> for Normal<L> {
    fn from(value: Box<[Term<L, ()>]>) -> Self {
        Self(value)
    }
}

pub struct Arena<L: Language>(Box<[Term<L, u16>]>);

impl<L: Language> Arena<L> {
    fn substitute(&mut self, var: u16, term: &Term<L, u16>) {
        self.0.iter_mut().for_each(|t| match *t {
            Term::Var(v) if v == var => {
                *t = term.clone();
            }
            _ => {}
        });
    }
}

fn occurs<L: Language>(arena: &Arena<L>, var: u16, term: &Term<L, u16>) -> bool {
    match term {
        Term::Var(x) => *x == var,
        Term::Term(t) => L::children(t)
            .iter()
            .any(|&idx| occurs(arena, var, &arena.0[idx as usize])),
    }
}

fn unify_many<L: Language>(arena: &mut Arena<L>, mut eqs: Vec<(u16, u16)>) -> bool {
    while let Some((a, b)) = eqs.pop() {
        match (&arena.0[a as usize], &arena.0[b as usize]) {
            (&Term::Var(x), t @ &Term::Var(y)) => {
                if x != y {
                    let t = t.clone();
                    arena.substitute(x, &t);
                }
            }
            (&Term::Var(x), t @ &Term::Term(_)) | (t @ &Term::Term(_), &Term::Var(x)) => {
                let t = t.clone();
                if occurs(arena, x, &t) {
                    return false;
                }
                arena.substitute(x, &t);
            }
            (Term::Term(t1), Term::Term(t2)) => {
                if !L::matches(t1, t2) {
                    return false;
                }
                L::children(t1)
                    .iter()
                    .zip(L::children(t2))
                    .for_each(|(&a, &b)| {
                        eqs.push((a, b));
                    });
            }
        }
    }
    true
}

pub fn modus_ponens<L: Language>(p: &Normal<L>, f: &Normal<L>) -> Option<Normal<L>> {
    let Term::Term(t) = &f.0[0] else {
        return None;
    };
    if L::match_implication(t).is_none() {
        return None;
    }

    let mut arena = Vec::with_capacity(p.len() + f.len());

    let max_var = p.0.iter().fold(0, |acc, t| {
        if let &Term::Var(x) = t {
            std::cmp::max(acc, x)
        } else {
            acc
        }
    });

    let p = p.write_into(&mut arena, 0);
    let f = f.write_into(&mut arena, max_var + 1);

    let Term::Term(t) = &arena[f as usize] else {
        return None;
    };
    let Some(&[p1, q]) = L::match_implication(t) else {
        return None;
    };

    let mut arena = Arena(arena.into());

    if unify_many(&mut arena, vec![(p, p1)]) {
        Some(Normal::<L>::from_arena(&arena, q as usize))
    } else {
        None
    }
}

#[cfg(test)]
mod test {
    use crate::formula::{langs, language::Arena};

    use super::{Language, Normal};

    fn test_conversion<L: Language>(f: &Normal<L>) {
        let mut arena = Vec::new();
        let idx = f.write_into(&mut arena, 0);
        let arena = Arena(arena.into());
        let res = Normal::from_arena(&arena, idx as usize);
        assert_eq!(f, &res);
    }

    #[test]
    fn test1() {
        for f in langs::ImpNeg::frege() {
            test_conversion(&f);
        }
    }

    #[test]
    fn test2() {
        test_conversion(&langs::ImpNeg::lukasiewicz_tarski()[0]);
    }
}
