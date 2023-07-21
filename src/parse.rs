use crate::{Formula, FormulaParseError, implication, not};




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
        return Err(FormulaParseError::MismatchedParen);
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
    use crate::{Formula::*, implication};

    #[test]
    fn basic() {
        assert_eq!(parse("12 -> 13").unwrap(), implication(Var(12), Var(13)));
        assert_eq!(parse("12 -> 13 -> 14").unwrap(), implication(Var(12), implication(Var(13), Var(14))));
        assert_eq!(parse("-12 -> 13 -> -14").unwrap(), implication(not(Var(12)), implication(Var(13), not(Var(14)))));
    }

    #[test]
    fn brackets() {
        assert_eq!(parse("(1 -> 2) -> 3").unwrap(), implication(implication(Var(1), Var(2)), Var(3)))
    }

    
}