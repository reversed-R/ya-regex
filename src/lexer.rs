#[derive(Debug, Clone, Copy, PartialEq)]
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
