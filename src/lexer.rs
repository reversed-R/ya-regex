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
