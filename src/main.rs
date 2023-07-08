use std::str::FromStr;

mod parse;

fn main() {
    println!("Hello, world!");
}


fn try_apply_mp(f: &Formula, a: &Formula, b: &Formula) -> Option<Formula> {
    let Some(subst) = try_unify(f,a)
    else {return None};

    Some(b.subst(&subst))
}

fn occurs(x: usize, t: &Formula) -> bool {
    match t {
        Formula::Var(y) => x == *y,
        Formula::Implication(a, b) => occurs(x, a) || occurs(x, b),
        Formula::Not(a) => occurs(x, a),
    }
}



fn try_unify(a: &Formula,b: &Formula) -> Option<Vec<(usize,Formula)>> {
    try_unify_many(vec![(a.clone(),b.clone())])
}

fn try_unify_many(mut eqs: Vec<(Formula, Formula)>) -> Option<Vec<(usize, Formula)>> {
    let mut solution = Vec::new();
    while let Some(eq) = eqs.pop() {
        match eq {
            (Formula::Var(x), t) | (t, Formula::Var(x)) => {
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

#[derive(Clone, Debug)]
pub enum Formula {
    Var(usize),
    Implication(Box<Formula>, Box<Formula>),
    Not(Box<Formula>),
}

impl FromStr for Formula {
    type Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}

impl Formula {
    fn subst(&self, s: &Vec<(usize,Formula)>) -> Formula {
        match self {
            Formula::Var(v) => {
                if let Some((_,sub)) = s.iter().find(|(x,_)| x == v) {
                    sub.clone()
                } else {
                    self.clone()
                }
            },
            Formula::Implication(a, b) => Formula::Implication(Box::new(a.subst(s)), Box::new(b.subst(s))),
            Formula::Not(a) => Formula::Not(Box::new(a.subst(s))),
        }
    }

    
}