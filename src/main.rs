use std::str::FromStr;

use formula::Formula;
use mp::try_apply_mp;

mod formula;
mod mp;

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
    dbg!(&context.formulas);
    dbg!(&new);
    println!("Axioms:");
    for formula in context.formulas.iter() {
        println!("{formula}");
    }
    println!("New:");
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
