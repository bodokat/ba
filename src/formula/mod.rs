mod mp;
pub mod parse;
pub use mp::modus_ponens;

use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Entry {
    Var(usize),
    Implication(Idx, Idx),
    Not(Idx),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Idx(usize);

#[derive(Debug, Clone)]
pub struct Arena(Vec<Entry>);

impl Arena {
    fn substitute(&mut self, var: usize, by: &Entry) {
        self.0.iter_mut().for_each(|e| {
            if *e == Entry::Var(var) {
                *e = by.clone();
            }
        })
    }
}

impl std::ops::Index<Idx> for Arena {
    type Output = Entry;

    fn index(&self, index: Idx) -> &Self::Output {
        &self.0[index.0]
    }
}

impl std::ops::IndexMut<Idx> for Arena {
    fn index_mut(&mut self, index: Idx) -> &mut Self::Output {
        &mut self.0[index.0]
    }
}

#[derive(Debug, Clone)]
pub struct Formula {
    arena: Arena,
}

impl PartialEq for Formula {
    fn eq(&self, other: &Self) -> bool {
        fn eq_at(arena1: &Arena, i1: Idx, arena2: &Arena, i2: Idx) -> bool {
            match (&arena1[i1], &arena2[i2]) {
                (Entry::Var(v1), Entry::Var(v2)) => v1 == v2,
                (Entry::Implication(a1, b1), Entry::Implication(a2, b2)) => {
                    eq_at(arena1, *a1, arena2, *a2) && eq_at(arena1, *b1, arena2, *b2)
                }

                (Entry::Not(a1), Entry::Not(a2)) => eq_at(arena1, *a1, arena2, *a2),
                _ => false,
            }
        }
        eq_at(
            &self.arena,
            self.toplevel_idx(),
            &other.arena,
            other.toplevel_idx(),
        )
    }
}

impl Display for Formula {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn format_at(arena: &Arena, i: Idx, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match arena[i] {
                Entry::Var(v) => {
                    write!(f, "{v}")?;
                }
                Entry::Implication(a, b) => {
                    if matches!(arena[a], Entry::Implication(_, _)) {
                        write!(f, "(")?;
                        format_at(arena, a, f)?;
                        write!(f, ") -> ")?;
                        format_at(arena, b, f)?;
                    } else {
                        format_at(arena, a, f)?;
                        write!(f, " -> ")?;
                        format_at(arena, b, f)?;
                    }
                }
                Entry::Not(a) => {
                    write!(f, "-")?;
                    if matches!(arena[a], Entry::Implication(_, _)) {
                        write!(f, "(")?;
                        format_at(arena, a, f)?;
                        write!(f, ")")?;
                    } else {
                        format_at(arena, a, f)?;
                    }
                }
            }
            Ok(())
        }
        format_at(&self.arena, self.toplevel_idx(), f)
    }
}

impl Formula {
    fn toplevel_idx(&self) -> Idx {
        Idx(0)
    }

    fn each_index<F: FnMut(&mut Idx)>(&mut self, mut f: F) {
        for elem in self.arena.0.iter_mut() {
            match elem {
                Entry::Var(_) => {}
                Entry::Implication(a, b) => {
                    f(a);
                    f(b);
                }
                Entry::Not(a) => {
                    f(a);
                }
            }
        }
    }

    pub fn var(x: usize) -> Self {
        Self {
            arena: Arena(vec![Entry::Var(x)]),
        }
    }
    pub fn implication(mut a: Self, mut b: Self) -> Self {
        let mut elems = Vec::with_capacity(1 + a.arena.0.len() + b.arena.0.len());
        elems.push(Entry::Implication(Idx(1), Idx(a.arena.0.len() + 1)));
        a.each_index(|Idx(x)| {
            *x += 1;
        });
        b.each_index(|Idx(x)| {
            *x += a.arena.0.len() + 1;
        });
        elems.append(&mut a.arena.0);
        elems.append(&mut b.arena.0);
        Self {
            arena: Arena(elems),
        }
    }

    pub fn not(mut a: Self) -> Self {
        let mut elems = Vec::with_capacity(1 + a.arena.0.len());
        a.each_index(|Idx(x)| {
            *x += 1;
        });
        elems.push(Entry::Not(Idx(1)));
        elems.append(&mut a.arena.0);
        Self {
            arena: Arena(elems),
        }
    }
}

// impl Into<Formula> for super::Formula {
//     fn into(self) -> Formula {
//         match self {
//             super::Formula::Var(x) => Formula::var(x),
//             super::Formula::Not(a) => Formula::not((*a).into()),
//             super::Formula::Implication(a, b) => Formula::implication((*a).into(), (*b).into()),
//         }
//     }
// }

impl FromStr for Formula {
    type Err = parse::FormulaParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse::parse(s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Normal {
    elems: Vec<NormalEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum NormalEntry {
    Implication,
    Not,
    Var(usize),
}

impl Normal {
    fn from_arena(arena: &Arena, i: Idx) -> Self {
        let mut elems = Vec::with_capacity(arena.0.len());
        fn inner(elems: &mut Vec<NormalEntry>, arena: &Arena, i: Idx) {
            match arena[i] {
                Entry::Var(x) => elems.push(NormalEntry::Var(x)),
                Entry::Implication(a, b) => {
                    elems.push(NormalEntry::Implication);
                    inner(elems, arena, a);
                    inner(elems, arena, b);
                }
                Entry::Not(a) => {
                    elems.push(NormalEntry::Not);
                    inner(elems, arena, a)
                }
            }
        }
        inner(&mut elems, &arena, i);
        let mut result = Self { elems };
        result.normalize();
        result
    }

    fn from_formula(val: &Formula) -> Self {
        Self::from_arena(&val.arena, val.toplevel_idx())
    }

    pub fn write_into(&self, arena: &mut Arena) {
        fn write_into_inner(val: &[NormalEntry], arena: &mut Arena) -> usize {
            match val[0] {
                NormalEntry::Var(x) => {
                    arena.0.push(Entry::Var(x));
                    1
                }
                NormalEntry::Implication => {
                    let s_index = Idx(arena.0.len());
                    arena.0.push(Entry::Var(0)); // Sentinel value
                    let a_index = Idx(arena.0.len());
                    let rest1 = write_into_inner(&val[1..], arena);
                    let b_index = Idx(arena.0.len());
                    let rest2 = write_into_inner(&val[(rest1 + 1)..], arena);
                    arena[s_index] = Entry::Implication(a_index, b_index);
                    rest1 + rest2 + 1
                }
                NormalEntry::Not => {
                    arena.0.push(Entry::Not(Idx(arena.0.len() + 1)));
                    write_into_inner(&val[1..], arena) + 1
                }
            }
        }
        let rest = write_into_inner(&self.elems, arena);
        assert_eq!(rest, self.elems.len());
    }

    fn normalize(&mut self) {
        let mut current_var = 1;
        for i in 0..self.elems.len() {
            if let &NormalEntry::Var(new_var) = &self.elems[i] {
                if current_var < new_var {
                    for elem in self.elems[i..].iter_mut() {
                        if let NormalEntry::Var(x) = elem {
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
}

impl Display for Normal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn inner(
            val: &[NormalEntry],
            f: &mut std::fmt::Formatter<'_>,
        ) -> Result<usize, std::fmt::Error> {
            match val[0] {
                NormalEntry::Implication => {
                    if val[1] == NormalEntry::Implication {
                        write!(f, "(")?;
                        let i = inner(&val[1..], f)?;
                        write!(f, ") -> ")?;
                        Ok(inner(&val[(i + 1)..], f)? + i + 1)
                    } else {
                        let i = inner(&val[1..], f)?;
                        write!(f, " -> ")?;
                        Ok(inner(&val[(i + 1)..], f)? + i + 1)
                    }
                }
                NormalEntry::Not => {
                    write!(f, "-")?;
                    if val[1] == NormalEntry::Implication {
                        write!(f, "(")?;
                        let i = inner(&val[1..], f)?;
                        write!(f, ")")?;
                        Ok(i + 1)
                    } else {
                        Ok(inner(&val[1..], f)? + 1)
                    }
                }
                NormalEntry::Var(x) => {
                    write!(f, "{x}")?;
                    Ok(1)
                }
            }
        }
        inner(&self.elems, f).map(|_| ())
    }
}

impl From<Formula> for Normal {
    fn from(value: Formula) -> Self {
        Normal::from_formula(&value)
    }
}

#[cfg(test)]
mod test {
    use super::{parse::parse, Normal};

    #[test]
    fn normalize() {
        let test_cases = &[
            "3 -> 4 -> -3 -> 5 -> --4",
            "1 -> 2 -> -1 -> 3 -> --2",
            "5 -> 1 -> -5 -> 3 -> --1",
            "2 -> 1 -> -3 -> 5 -> --1",
        ]
        .map(|x| Normal::from(parse(x).unwrap()));
        let expected = Normal::from(parse("1 -> 2 -> -1 -> 3 -> --2").unwrap());
        for case in test_cases.iter() {
            assert_eq!(*case, expected);
        }
    }
}
