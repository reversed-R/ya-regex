use crate::lexer::TokenKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Node {
    Char(char),
    Concat(Box<Node>, Box<Node>),
    Or(Box<Node>, Box<Node>),
    Repeat(Box<Node>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ParseError {
    UnexpectedEOF,
    UnexpectedToken,
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
        let left = Self::parse_atomic(tokens)?;

        if let Some(t) = tokens.peek() {
            match t {
                TokenKind::Bar => {
                    tokens.next();

                    let right = Self::parse_atomic(tokens)?;

                    Ok(Self::Or(Box::new(left), Box::new(right)))
                }
                TokenKind::Star => {
                    tokens.next();

                    Ok(Self::Repeat(Box::new(left)))
                }
                TokenKind::LPare => Ok(left),
                TokenKind::RPare => Ok(left),
                TokenKind::Char(_) => Ok(left),
            }
        } else {
            Ok(left)
        }
    }

    fn parse_atomic(
        tokens: &mut std::iter::Peekable<std::slice::Iter<'_, TokenKind>>,
    ) -> Result<Self, ParseError> {
        match tokens.next().ok_or(ParseError::UnexpectedEOF)? {
            TokenKind::Char(c) => Ok(Self::Char(*c)),
            TokenKind::LPare => {
                let expr = Self::parse_expression(tokens)?;

                Self::consume_token(tokens, TokenKind::RPare)?;

                Ok(expr)
            }
            _ => Err(ParseError::UnexpectedToken),
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
                Err(ParseError::UnexpectedToken)
            }
        } else {
            Err(ParseError::UnexpectedEOF)
        }
    }
}
