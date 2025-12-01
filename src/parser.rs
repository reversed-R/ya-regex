use crate::lexer::TokenKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Node {
    Char(char),
    Concat(Box<Node>, Box<Node>),
    Or(Box<Node>, Box<Node>),
    Repeat(Box<Node>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ParseError {
    UnexpectedEOF,
    UnexpectedToken(TokenKind, Vec<TokenKind>),
    ExpectedEOF(TokenKind),
}

impl Node {
    pub(crate) fn parse(tokens: &[TokenKind]) -> Result<Self, ParseError> {
        let mut tokens = tokens.iter().peekable();

        let seq = Self::parse_sequence(&mut tokens)?;

        if let Some(t) = tokens.next() {
            Err(ParseError::ExpectedEOF(*t))
        } else {
            Ok(seq)
        }
    }

    pub(crate) fn parse_sequence(
        tokens: &mut std::iter::Peekable<std::slice::Iter<'_, TokenKind>>,
    ) -> Result<Self, ParseError> {
        let mut left = Self::parse_binary(tokens)?;

        while let Some(t) = tokens.peek() {
            if matches!(t, TokenKind::Char(_) | TokenKind::LPare) {
                let right = Self::parse_binary(tokens)?;

                left = Self::Concat(Box::new(left), Box::new(right));
            } else {
                return Ok(left);
            }
        }

        Ok(left)
    }

    fn parse_binary(
        tokens: &mut std::iter::Peekable<std::slice::Iter<'_, TokenKind>>,
    ) -> Result<Self, ParseError> {
        let left = Self::parse_unary(tokens)?;

        if let Some(TokenKind::Bar) = tokens.peek() {
            tokens.next();

            let right = Self::parse_unary(tokens)?;

            Ok(Self::Or(Box::new(left), Box::new(right)))
        } else {
            Ok(left)
        }
    }

    fn parse_unary(
        tokens: &mut std::iter::Peekable<std::slice::Iter<'_, TokenKind>>,
    ) -> Result<Self, ParseError> {
        let left = Self::parse_atomic(tokens)?;

        if let Some(TokenKind::Star) = tokens.peek() {
            tokens.next();

            Ok(Self::Repeat(Box::new(left)))
        } else {
            Ok(left)
        }
    }

    fn parse_atomic(
        tokens: &mut std::iter::Peekable<std::slice::Iter<'_, TokenKind>>,
    ) -> Result<Self, ParseError> {
        let t = tokens.next().ok_or(ParseError::UnexpectedEOF)?;

        match t {
            TokenKind::Char(c) => Ok(Self::Char(*c)),
            TokenKind::LPare => {
                let seq = Self::parse_sequence(tokens)?;

                Self::consume_token(tokens, TokenKind::RPare)?;

                Ok(seq)
            }
            _ => Err(ParseError::UnexpectedToken(
                *t,
                vec![TokenKind::Char('c'), TokenKind::LPare],
            )),
        }
    }

    fn consume_token(
        tokens: &mut std::iter::Peekable<std::slice::Iter<'_, TokenKind>>,
        t: TokenKind,
    ) -> Result<(), ParseError> {
        if let Some(next) = tokens.next() {
            if &t == next {
                Ok(())
            } else {
                Err(ParseError::UnexpectedToken(*next, vec![t]))
            }
        } else {
            Err(ParseError::UnexpectedEOF)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{lexer::TokenKind, parser::Node};

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
