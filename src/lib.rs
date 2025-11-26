pub(crate) mod lexer;
pub(crate) mod nfa;
pub(crate) mod parser;

#[cfg(test)]
mod tests {
    use crate::{
        lexer::{self, TokenKind},
        parser::Node,
    };

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

    #[test]
    fn parse_tokens() {
        let tokens = vec![
            TokenKind::Char('a'),
            TokenKind::LPare,
            TokenKind::Char('b'),
            TokenKind::Bar,
            TokenKind::Char('c'),
            TokenKind::RPare,
            TokenKind::Star,
        ];

        let expected = Node::Concat(
            Box::new(Node::Char('a')),
            Box::new(Node::Repeat(Box::new(Node::Or(
                Box::new(Node::Char('b')),
                Box::new(Node::Char('c')),
            )))),
        );

        let result = Node::parse(&tokens);

        assert_eq!(result, Ok(expected));
    }
}
