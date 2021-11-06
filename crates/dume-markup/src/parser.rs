use anyhow::{bail, Context};
use logos::Logos;

use std::str::FromStr;

use crate::{
    lexer::{ColorToken, Token},
    output::{Text, TextChunk, TextSection, TextStyle},
};

#[derive(Debug, Copy, Clone)]
struct ParseToken<'a> {
    tok: Token,
    text: &'a str,
}

impl Default for ParseToken<'_> {
    fn default() -> Self {
        Self {
            tok: Token::Error,
            text: "",
        }
    }
}

struct Parser<'a> {
    tokens: Vec<ParseToken<'a>>,
    cursor: usize,
    output: Text,
    next_fmt_index: usize,
}

impl<'a> Parser<'a> {
    pub fn new(markup: &'a str) -> Self {
        let mut tokens = Vec::new();
        let mut lexer = Token::lexer(markup);
        while let Some(tok) = lexer.next() {
            tokens.push(ParseToken {
                tok,
                text: lexer.slice(),
            })
        }

        Self {
            tokens,
            cursor: 0,
            output: Text::default(),
            next_fmt_index: 0,
        }
    }

    pub fn peek(&mut self) -> ParseToken<'a> {
        self.consume_ws();
        self.current_token()
    }

    pub fn expect(&mut self, tok: Token) -> anyhow::Result<&'a str> {
        if self.peek().tok != tok {
            bail!("expected {:?}, found {:?}", tok, self.peek().tok);
        } else {
            let text = self.peek().text;
            self.cursor += 1;
            Ok(text)
        }
    }

    pub fn consume(&mut self) {
        self.cursor += 1;
    }

    fn current_token(&self) -> ParseToken<'a> {
        self.tokens.get(self.cursor).copied().unwrap_or_default()
    }

    fn consume_ws(&mut self) {
        while self.current_token().tok == Token::Whitespace {
            self.cursor += 1;
        }
    }

    pub fn section(&mut self, section: TextSection) {
        self.output.sections.push(section);
    }

    pub fn next_fmt_index(&mut self) -> usize {
        self.next_fmt_index += 1;
        self.next_fmt_index - 1
    }

    pub fn is_done(&self) -> bool {
        self.cursor >= self.tokens.len()
    }

    pub fn finish(self) -> Text {
        assert!(self.is_done());
        self.output
    }
}

pub fn parse(markup: &str) -> anyhow::Result<Text> {
    let mut parser = Parser::new(markup);
    let base_style = TextStyle::default();

    parse_section(&mut parser, base_style)?;

    Ok(parser.finish())
}

fn parse_section(p: &mut Parser, current_style: TextStyle) -> anyhow::Result<()> {
    while !p.is_done() {
        let token = p.current_token();
        match token.tok {
            Token::At => {
                p.consume();
                let command = p.expect(Token::Text)?;
                let command = Command::from_str(command)?;

                let mut nested_style = current_style.clone();
                command.parse_and_apply(p, &mut nested_style)?;

                if p.peek().tok == Token::LBracket {
                    // Recurse with the nested style.
                    p.consume();
                    parse_section(p, nested_style)?;
                    p.expect(Token::RBracket)?;
                }
                continue;
            }
            Token::Text | Token::Whitespace => {
                p.section(TextSection::Text {
                    chunk: TextChunk::Literal(token.text.to_owned()),
                    style: current_style.clone(),
                });
            }
            Token::Display => {
                let fmt_index = p.next_fmt_index();
                p.section(TextSection::Text {
                    chunk: TextChunk::FormatDisplay { fmt_index },
                    style: current_style.clone(),
                })
            }
            Token::Debug => {
                let fmt_index = p.next_fmt_index();
                p.section(TextSection::Text {
                    chunk: TextChunk::FormatDebug { fmt_index },
                    style: current_style.clone(),
                })
            }
            Token::RBracket => break,
            t => bail!("expected text or @, found {:?}", t),
        }
        p.consume();
    }

    Ok(())
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Command {
    Bold,
    Light,
    Italic,
    Size,
    Color,
    Font,
    Icon,
}

impl Command {
    fn from_str(s: &str) -> anyhow::Result<Self> {
        Ok(match s.trim() {
            "bold" => Command::Bold,
            "light" => Command::Light,
            "italic" => Command::Italic,
            "size" => Command::Size,
            "color" => Command::Color,
            "icon" => Command::Icon,
            "font" => Command::Font,
            s => bail!("'{}' is not a recognized command", s),
        })
    }

    fn parse_and_apply(self, p: &mut Parser, style: &mut TextStyle) -> anyhow::Result<()> {
        match self {
            Command::Bold => style.bold = true,
            Command::Light => {
                style.bold = false;
                style.light = true;
            }
            Command::Italic => style.italic = true,
            Command::Size => {
                p.expect(Token::LBracket)?;
                let text = p.expect(Token::Text)?;
                let size = f32::from_str(text.trim())?;
                style.size = Some(size);
                p.expect(Token::RBracket)?;
            }
            Command::Font => {
                p.expect(Token::LBracket)?;
                let text = p.expect(Token::Text)?;
                style.font = Some(text.trim().to_owned());
                p.expect(Token::RBracket)?;
            }
            Command::Color => {
                p.expect(Token::LBracket)?;

                let text = p.expect(Token::Text)?;
                let color = parse_color(text).context("failed to parse color")?;
                style.color = Some(color);

                p.expect(Token::RBracket)?;
            }
            Command::Icon => {
                p.expect(Token::LBracket)?;

                let icon = p.expect(Token::Text)?;
                p.section(TextSection::Icon {
                    texture: icon.trim().to_owned(),
                    size: style.size.unwrap_or(12.),
                });

                p.expect(Token::RBracket)?;
            }
        }

        Ok(())
    }
}

fn parse_color(input: &str) -> anyhow::Result<[u8; 4]> {
    let mut lexer = ColorToken::lexer(input);

    let mut color = [u8::MAX; 4];
    let mut component = 0;
    let mut waiting_for_comma = false;

    while let Some(token) = lexer.next() {
        match token {
            ColorToken::Comma => {
                if !waiting_for_comma {
                    bail!("expected number")
                } else {
                    waiting_for_comma = false;
                }
            }
            ColorToken::Number => {
                if component > 3 {
                    bail!("too many color components");
                }

                let number = u8::from_str(lexer.slice())?;
                color[component] = number;
                component += 1;
                waiting_for_comma = true;
            }
            ColorToken::Whitespace => continue,
            ColorToken::Error => bail!("encountered error while parsing color"),
        }
    }

    Ok(color)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_color_rgb() {
        assert_eq!(parse_color("5, 10 ,235 ").unwrap(), [5, 10, 235, 255]);
    }

    #[test]
    fn test_parse_color_rgba() {
        assert_eq!(parse_color("235, 10,5,100").unwrap(), [235, 10, 5, 100]);
    }

    #[test]
    fn test_parse_color_too_many_components() {
        assert!(parse_color("235,100,20,40,20").is_err());
    }

    #[test]
    fn test_parse_complex_text() {
        let markup = "Ozymandias, {} of Kings @bold[requests that you@color[5,5,5][ go away]]. @icon[lol] @size[80][Do it.]";
        println!("{:#?}", parse(markup).unwrap());
    }
}
