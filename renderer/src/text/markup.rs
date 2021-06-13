//! Text markup parser.
//!
//! # Syntax
//! Formatting specifiers begin with '@' and are followed
//! by (optionally) an argument in braces and the text in br color: (), size: (), font: () aces. Examples:
//!
//! `@bold{Bold text} non bold text`
//!
//! `@font{Times New Roman}{Times New Roman text} default font text`
//!
//! `@italic{Italicized text @bold{Italicized and bolded text}} plain text`
//!
//! `@size{10}{10 px text}@size{50}{Very big text}`
//!
//! Icons can be embedded inside text using sprite names:
//!
//! `Icon: @icon{smily_face}`
//!
//! To avoid injection attacks, user-provided strings should be applied using variables:
//!
//! `%city_name` is replaced with the string provided as "city_name".

use ahash::AHashMap;
use anyhow::{anyhow, bail};
use logos::Logos;

use crate::{
    font::{Style, Weight},
    Text, TextSection, TextStyle,
};

use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Logos)]
enum Token {
    #[token("@")]
    At,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,

    #[regex("[^@{}]+")]
    Text,

    #[error]
    Error,
}

pub fn parse(
    markup: &str,
    default_style: TextStyle,
    resolve_variable: impl FnMut(&str) -> String,
) -> anyhow::Result<Text> {
    let lexer = Token::lexer(markup);
    let mut tokens: Vec<(Token, String)> = lexer
        .spanned()
        .map(|(token, span)| (token, markup[span].to_owned()))
        .collect();

    // Reverse the order so we can pop() when consuming tokens.
    tokens.reverse();

    apply_variables(&mut tokens, resolve_variable);

    let mut sections = Vec::new();
    parse_internal(&mut tokens, &mut sections, default_style)?;

    Ok(Text::from_sections(sections))
}

fn apply_variables(
    tokens: &mut Vec<(Token, String)>,
    mut resolve_variable: impl FnMut(&str) -> String,
) {
    for (token, text) in tokens {
        if *token != Token::Text {
            continue;
        }

        let mut cursor = 0;
        while let Some(var_start) = (&text[cursor..]).chars().position(|c| c == '%') {
            let var_end = (&text[var_start + 1..])
                .chars()
                .position(|c| !c.is_alphanumeric() && c != '_')
                .map(|pos| pos + var_start + 1)
                .unwrap_or_else(|| text.len());
            let var = &text[var_start + 1..var_end];

            let new_value = resolve_variable(var);

            *text = String::from(&text[..var_start]) + (&new_value) + &text[var_end..];

            cursor = var_start + new_value.len();
        }
    }
}

// Recursive descent parser.
fn parse_internal(
    tokens: &mut Vec<(Token, String)>,
    sections: &mut Vec<TextSection>,
    style: TextStyle,
) -> anyhow::Result<()> {
    match tokens.pop() {
        None => return Ok(()),
        // Text
        Some((Token::Text, text)) => {
            sections.push(TextSection::Text {
                text,
                style: style.clone(),
            });

            parse_internal(tokens, sections, style)?;
        }
        // Formatting specifier
        Some((Token::At, _)) => {
            let specifier = match tokens.pop() {
                Some((Token::Text, specifier)) => specifier,
                _ => bail!("expected formatting specifier after '@'"),
            };
            let specifier = specifier.trim();

            // Argument 1
            let arg1 = if specifier_has_first_argument(specifier) {
                if !matches!(tokens.pop(), Some((Token::LBrace, _))) {
                    bail!("expected at least one argument for formatting specifier");
                }

                let arg1 = match tokens.pop() {
                    Some((Token::Text, text)) => text,
                    _ => bail!("expected text inside argument"),
                };

                if !matches!(tokens.pop(), Some((Token::RBrace, _))) {
                    bail!("unterminated first argument to a formatting specifier");
                }

                arg1.trim().to_owned()
            } else {
                String::new()
            };

            // Parse a child using the new specifier.
            let mut child_style = style.clone();
            apply_specifier(specifier, &mut child_style, &arg1);

            // Argument 2 (recurse)
            if !matches!(tokens.pop(), Some((Token::LBrace, _))) {
                bail!("expected at least one argument for formatting specifier");
            }

            parse_internal(tokens, sections, child_style)?;

            parse_internal(tokens, sections, style)?;
        }
        Some((Token::RBrace, _)) => return Ok(()),
        Some((token, _)) => bail!("expected text or '@', found {:?}", token),
    }

    Ok(())
}

fn specifier_has_first_argument(specifier: &str) -> bool {
    match specifier {
        "bold" => false,
        "italic" => false,
        "font" => true,
        "size" => true,
        _ => false,
    }
}

fn apply_specifier(specifier: &str, style: &mut TextStyle, argument: &str) -> anyhow::Result<()> {
    match specifier {
        "bold" => style.font.weight = Weight::Bold,
        "italic" => style.font.style = Style::Italic,
        "font" => style.font.family = argument.to_owned(),
        "size" => style.size = f32::from_str(argument)?,
        _ => bail!("unknown formatting specifier '{}'", specifier),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::font::Query;

    use super::*;

    #[test]
    fn simple() {
        let text = parse(" basic text  ", TextStyle::default(), |_| String::new()).unwrap();
        assert_eq!(
            text,
            Text::from_sections(vec![TextSection::Text {
                text: " basic text  ".to_owned(),
                style: TextStyle::default()
            }])
        );
    }

    #[test]
    fn bold() {
        let text = parse("basic text @bold{bold text }", TextStyle::default(), |_| {
            String::new()
        })
        .unwrap();
        assert_eq!(
            text,
            Text::from_sections(vec![
                TextSection::Text {
                    text: "basic text ".to_owned(),
                    style: TextStyle::default()
                },
                TextSection::Text {
                    text: "bold text ".to_owned(),
                    style: TextStyle {
                        font: Query {
                            weight: Weight::Bold,
                            ..TextStyle::default().font
                        },
                        ..TextStyle::default()
                    }
                }
            ])
        );
    }

    #[test]
    fn nested() {
        let text = parse(
            "@size{5}{very small text }@size{50}{Big text @bold{Bold big text}} default text",
            TextStyle::default(),
            |_| String::new(),
        )
        .unwrap();
        assert_eq!(
            text,
            Text::from_sections(vec![
                TextSection::Text {
                    text: "very small text ".to_owned(),
                    style: TextStyle {
                        size: 5.0,
                        ..Default::default()
                    }
                },
                TextSection::Text {
                    text: "Big text ".to_owned(),
                    style: TextStyle {
                        size: 50.0,
                        ..Default::default()
                    },
                },
                TextSection::Text {
                    text: "Bold big text".to_owned(),
                    style: TextStyle {
                        font: Query {
                            weight: Weight::Bold,
                            ..TextStyle::default().font
                        },
                        size: 50.0,
                        ..Default::default()
                    }
                },
                TextSection::Text {
                    text: " default text".to_owned(),
                    style: TextStyle::default(),
                }
            ])
        );
    }

    #[test]
    fn variables() {
        let text = parse("My name is %name.", TextStyle::default(), |var| {
            if var == "name" {
                "Ozymandias".to_owned()
            } else {
                panic!("unknown variable {}", var);
            }
        })
        .unwrap();
        assert_eq!(
            text,
            Text::from_sections(vec![TextSection::Text {
                text: "My name is Ozymandias.".to_owned(),
                style: TextStyle::default()
            }])
        );
    }
}
