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
}

impl Node {
    pub(crate) fn parse(tokens: &[TokenKind]) -> Result<Self, ParseError> {
        let mut tokens = tokens.iter().peekable();

        let mut left = Self::parse_expression(&mut tokens)?;

        while tokens.peek().is_some() {
            let right = Self::parse_expression(&mut tokens)?;

            left = Self::Concat(Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    fn parse_expression(
        tokens: &mut std::iter::Peekable<std::slice::Iter<'_, TokenKind>>,
    ) -> Result<Self, ParseError> {
        let left = Self::parse_repeat(tokens)?;

        if let Some(TokenKind::Bar) = tokens.peek() {
            tokens.next();

            let right = Self::parse_repeat(tokens)?;

            Ok(Self::Or(Box::new(left), Box::new(right)))
        } else {
            Ok(left)
        }
    }

    fn parse_repeat(
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
            TokenKind::Char(c) => {
                let mut left = Self::Char(*c);

                while let Some(TokenKind::Char(c)) = tokens.peek() {
                    tokens.next();

                    left = Self::Concat(Box::new(left), Box::new(Self::Char(*c)));
                }

                Ok(left)
            }
            TokenKind::LPare => {
                let expr = Self::parse_expression(tokens)?;

                Self::consume_token(tokens, TokenKind::RPare)?;

                Ok(expr)
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
