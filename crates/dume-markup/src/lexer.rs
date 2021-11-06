use logos::Logos;

#[derive(Copy, Clone, Debug, Logos, PartialEq, Eq)]
pub enum Token {
    #[token("{}")]
    Display,
    #[token("{:?}")]
    Debug,

    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,

    #[token("@")]
    At,
    
    #[regex("[ \n\t]+", priority = 2)]
    Whitespace,

    #[regex("[^@{}\\[\\]]+")]
    Text,

    #[error]
    Error,
}

#[derive(Copy, Clone, Debug, Logos, PartialEq, Eq)]
pub enum ColorToken {
    #[token(",")]
    Comma,

    #[regex("[0-9]+")]
    Number,

    #[regex("[ \n\t]+")]
    Whitespace,

    #[error]
    Error,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lex_string() {
        let markup = "@color[5, 10, 100][@bold[Ozymandias {} King of {:?} Kings]]";
        let mut lexer = Token::lexer(markup);

        while let Some(tok) = lexer.next() {
            println!("{:?} '{}'", tok, lexer.slice());
        }
    }

    #[test]
    fn lex_color() {
        let color = "5, 10, 100 ";
        let mut lexer = ColorToken::lexer(color);

        while let Some(tok) = lexer.next() {
            println!("{:?} '{}'", tok, lexer.slice());
        }
    }
}
