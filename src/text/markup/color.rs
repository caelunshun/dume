use palette::Srgba;

use std::str::FromStr;

#[derive(Debug, thiserror::Error)]
pub enum ColorParseError {
    #[error("expected parenthesis after color type")]
    MissingParenthesis,
    #[error("unknown color type - expected one of `rgb`, `rgba`")]
    UnknownType,
    #[error(transparent)]
    BadValue(std::num::ParseIntError),
    #[error("expected {expected} color components but found {actual}")]
    ComponentMismatch { expected: usize, actual: usize },
}

pub fn parse_color(s: &str) -> Result<Srgba<u8>, ColorParseError> {
    match *s.as_bytes() {
        [b'r', b'g', b'b', b'a', ..] => parse_rgba(&s[4..]),
        [b'r', b'g', b'b', ..] => parse_rgb(&s[3..]),
        _ => Err(ColorParseError::UnknownType),
    }
}

fn parse_rgb(s: &str) -> Result<Srgba<u8>, ColorParseError> {
    let components = parse_components(parenthesized(s)?)?;
    if let [r, g, b] = *components.as_slice() {
        Ok(Srgba::new(r, g, b, u8::MAX))
    } else {
        Err(ColorParseError::ComponentMismatch {
            expected: 3,
            actual: components.len(),
        })
    }
}

fn parse_rgba(s: &str) -> Result<Srgba<u8>, ColorParseError> {
    let components = parse_components(parenthesized(s)?)?;
    if let [r, g, b, a] = *components.as_slice() {
        Ok(Srgba::new(r, g, b, a))
    } else {
        Err(ColorParseError::ComponentMismatch {
            expected: 3,
            actual: components.len(),
        })
    }
}

fn parenthesized(s: &str) -> Result<&str, ColorParseError> {
    let s = s.trim();
    match (s.chars().next(), s.chars().last()) {
        (Some('('), Some(')')) => Ok(&s[1..s.len() - 1]),
        _ => Err(ColorParseError::MissingParenthesis),
    }
}

fn parse_components(s: &str) -> Result<Vec<u8>, ColorParseError> {
    let mut result = Vec::new();
    for part in s.split(',') {
        let part = part.trim();
        let component = u8::from_str(part).map_err(ColorParseError::BadValue)?;
        result.push(component);
    }
    Ok(result)
}
