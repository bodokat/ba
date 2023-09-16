use super::{Formula, implication, not};

#[derive(Debug)]
pub enum FormulaParseError {
    UnexpectedEOF,
    UnexpectedChar(char),
}


pub fn parse(mut s: &str) -> Result<Formula,FormulaParseError> {
    let left: Formula = parse_one(&mut s)?;

    eat_space(&mut s);
    if s.is_empty() {
        return Ok(left);
    } else if s.starts_with("->") {
        let right = parse(&s[2..])?;
        return Ok(implication(left, right));
    } else {
        return Err(FormulaParseError::UnexpectedChar(s.chars().next().unwrap()));
    }
}

fn parse_to_bracket(s: &mut &str) -> Result<Formula, FormulaParseError> {
    let left: Formula = parse_one(s)?;

    eat_space(s);
    if s.is_empty() {
        return Err(FormulaParseError::UnexpectedEOF);
    } else if s.starts_with(')') {
        *s = &s[1..];
        return Ok(left);
    } else if s.starts_with("->") {
        *s = &s[2..];
        let right = parse_to_bracket(s)?;
        return Ok(implication(left, right));
    } else {
        return Err(FormulaParseError::UnexpectedChar(s.chars().next().unwrap()));
    }
}

fn parse_one(s: &mut &str) -> Result<Formula,FormulaParseError> {
    eat_space(s);
    let c = s.chars().next().ok_or(FormulaParseError::UnexpectedEOF)?;
    match c {
        '(' => {
            *s = &s[1..];
            parse_to_bracket(s)
        },
        '0'..='9' => parse_var(s),
        '-' => {
            *s = &s[1..];
            eat_space(s);
            let inner = parse_one(s)?;
            Ok(not(inner))
        }
        c => Err(FormulaParseError::UnexpectedChar(c))
    }
}

fn parse_var(s: &mut &str) -> Result<Formula, FormulaParseError> {
    let index = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
    let num_slice = &s[0..index];
    let num = num_slice.parse().map_err(|_| FormulaParseError::UnexpectedChar(num_slice.chars().next().unwrap()))?;
    *s = &s[index..];
    Ok(Formula::Var(num))
}

fn eat_space(s: &mut &str) {
    *s = s.trim_start();
}

#[cfg(test)]
mod test {
    use super::*;
    use super::Formula::*;
    use crate::AXIOMS;

    #[test]
    fn basic() {
        assert_eq!(parse("12 -> 13").unwrap(), implication(Var(12), Var(13)));
        assert_eq!(parse("12 -> 13 -> 14").unwrap(), implication(Var(12), implication(Var(13), Var(14))));
        assert_eq!(parse("-12 -> 13 -> -14").unwrap(), implication(not(Var(12)), implication(Var(13), not(Var(14)))));
    }

    #[test]
    fn brackets() {
        assert_eq!(parse("(1 -> 2) -> 3").unwrap(), implication(implication(Var(1), Var(2)), Var(3)));
        assert_eq!(parse("-(1 -> 2) -> 3").unwrap(), implication(not(implication(Var(1), Var(2))), Var(3)));
    }

    #[test]
    fn axioms() {
        assert_eq!(
            AXIOMS.iter().map(|s| parse(s).unwrap()).collect::<Vec<_>>(),
            vec![
                implication(Var(1), implication(Var(2), Var(1))),
                implication(implication(Var(1), implication(Var(2), Var(3))), implication(implication(Var(1), Var(2)), implication(Var(1), Var(3)))),
                implication(implication(Var(1), implication(Var(2), Var(3))), implication(Var(2), implication(Var(1), Var(3)))),
                implication(implication(Var(1), Var(2)), implication(not(Var(2)), not(Var(1)))),
                implication(not(not(Var(1))), Var(1)),
                implication(Var(1), not(not(Var(1)))),
            ]
        )
    }


}