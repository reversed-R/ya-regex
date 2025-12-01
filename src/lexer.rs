use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Char(char), // single character
    LPare,      // (
    RPare,      // )
    Bar,        // |
    Star,       // *
}

impl From<char> for TokenKind {
    fn from(value: char) -> Self {
        match value {
            '(' => Self::LPare,
            ')' => Self::RPare,
            '|' => Self::Bar,
            '*' => Self::Star,
            _ => Self::Char(value),
        }
    }
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Char(c) => write!(f, "char `{c}`"),
            Self::LPare => write!(f, "`(`"),
            Self::RPare => write!(f, "`)`"),
            Self::Bar => write!(f, "`|`"),
            Self::Star => write!(f, "`*`"),
        }
    }
}

pub fn tokenize(src: &str) -> Vec<TokenKind> {
    src.chars().map(TokenKind::from).collect()
}

#[cfg(test)]
mod tests {
    use crate::lexer::{self, TokenKind};

    #[test]
    fn tokenize_raw_chars() {
        let raw = "a(b|c)*";

        let expected = vec![
            TokenKind::Char('a'),
            TokenKind::LPare,
            TokenKind::Char('b'),
            TokenKind::Bar,
            TokenKind::Char('c'),
            TokenKind::RPare,
            TokenKind::Star,
        ];

        let result = lexer::tokenize(raw);

        assert_eq!(result, expected);
    }
}
