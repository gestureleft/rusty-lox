use crate::{
    expression::{
        binary_expression, boolean_literal_expression, grouping_expression, nil_literal,
        number_literal_expression, string_literal_expression, unary_expression, Expression,
    },
    lexer::{self, Token, TokenType},
    span::Span,
    statement::Statement,
};

pub struct Parser {
    current_index: usize,
    errors: Vec<Error>,
}

pub struct ParserResult<'a> {
    pub errors: Vec<Error>,
    pub statements: Vec<Statement<'a>>,
}

impl Parser {
    pub fn parse<'a>(source: &'a str, tokens: &'a [Token]) -> ParserResult<'a> {
        let mut parser = Parser {
            current_index: 0,
            errors: vec![],
        };
        let mut statements = vec![];
        while let Some(token) = parser.current_token(tokens) && token.type_ != TokenType::Eof {
            let expression = parser.parse_expression(tokens);
            if expression.is_none() {
                println!("Couldnt' parse expression");
                // TODO: Synchronise parser
                parser.synchronise(tokens);
                continue;
            }
            let expression = expression.unwrap();
            statements.push(Statement::Expression(expression));
        }

        ParserResult {
            errors: parser.errors,
            statements,
        }
    }

    fn parse_expression<'a>(&mut self, tokens: &'a [Token]) -> Option<Expression<'a>> {
        self.parse_equality(tokens)
    }

    fn parse_equality<'a>(&mut self, tokens: &'a [Token]) -> Option<Expression<'a>> {
        let mut expression = self.parse_comparison(tokens)?;

        while self
            .consume_token_if_in_vec(tokens, &vec![TokenType::BangEqual, TokenType::EqualEqual])
        {
            let operator = tokens.get(self.current_index - 1).unwrap();
            let right = self.parse_comparison(tokens)?;
            expression = binary_expression(expression, right, operator);
        }

        Some(expression)
    }

    fn parse_comparison<'a>(&mut self, tokens: &'a [Token]) -> Option<Expression<'a>> {
        let mut expression = self.parse_term(tokens)?;

        while self.consume_token_if_in_vec(
            tokens,
            &vec![
                TokenType::Greater,
                TokenType::GreaterEqual,
                TokenType::Less,
                TokenType::LessEqual,
            ],
        ) {
            let operator = tokens.get(self.current_index - 1).unwrap();
            let right = self.parse_term(tokens)?;
            expression = binary_expression(expression, right, operator);
        }

        Some(expression)
    }

    fn parse_term<'a>(&mut self, tokens: &'a [Token]) -> Option<Expression<'a>> {
        let mut expression = self.parse_factor(tokens)?;

        while self.consume_token_if_in_vec(tokens, &vec![TokenType::Minus, TokenType::Plus]) {
            let operator = tokens.get(self.current_index - 1).unwrap();
            let right = self.parse_factor(tokens)?;
            expression = binary_expression(expression, right, operator);
        }

        Some(expression)
    }

    fn parse_factor<'a>(&mut self, tokens: &'a [Token]) -> Option<Expression<'a>> {
        let mut expression = self.parse_unary(tokens)?;

        while self.consume_token_if_in_vec(tokens, &vec![TokenType::Slash, TokenType::Star]) {
            let operator = tokens.get(self.current_index - 1).unwrap();
            let right = self.parse_unary(tokens)?;
            expression = binary_expression(expression, right, operator);
        }

        Some(expression)
    }

    fn parse_unary<'a>(&mut self, tokens: &'a [Token]) -> Option<Expression<'a>> {
        if self.consume_token_if_in_vec(tokens, &vec![TokenType::Bang, TokenType::Minus]) {
            let operator = tokens.get(self.current_index - 1).unwrap();
            let right = self.parse_unary(tokens)?;
            return Some(unary_expression(operator, right));
        };

        self.parse_primary(tokens)
    }

    fn parse_primary<'a>(&mut self, tokens: &'a [Token]) -> Option<Expression<'a>> {
        if self.consume_token_if_in_vec(tokens, &vec![TokenType::False]) {
            return Some(boolean_literal_expression(false));
        };
        if self.consume_token_if_in_vec(tokens, &vec![TokenType::True]) {
            return Some(boolean_literal_expression(true));
        };
        if self.consume_token_if_in_vec(tokens, &vec![TokenType::Nil]) {
            return Some(nil_literal());
        };
        if self.consume_token_if_in_vec(tokens, &vec![TokenType::Number]) {
            return Some(number_literal_expression(
                tokens.get(self.current_index - 1).unwrap(),
            ));
        };
        if self.consume_token_if_in_vec(tokens, &vec![TokenType::String_]) {
            return Some(string_literal_expression(
                tokens.get(self.current_index - 1).unwrap(),
            ));
        };
        if self.consume_token_if_in_vec(tokens, &vec![TokenType::LeftParen]) {
            let expression = self.parse_expression(tokens)?;
            let current_token = self.current_token(tokens);

            if current_token.expect("Unexpectedly ran out of tokens").type_ != TokenType::RightParen
            {
                self.errors.push(Error::UnexpectedToken {
                    expected_token_type: Some(TokenType::RightParen),
                    unexpected_token_type: current_token.unwrap().type_.clone(),
                    span: current_token.unwrap().span.clone(),
                });
                panic!()
            }

            self.current_index += 1;
            return Some(grouping_expression(expression));
        };

        self.errors.push(Error::UnexpectedToken {
            expected_token_type: None,
            unexpected_token_type: self.current_token(tokens).unwrap().type_.clone(),
            span: self.current_token(tokens).unwrap().span.clone(),
        });
        None
    }

    /// If the current token's type is in the given list, consume it and return true.
    /// Else, do nothing and return false
    fn consume_token_if_in_vec(&mut self, tokens: &[Token], token_types: &Vec<TokenType>) -> bool {
        let current_token = self.current_token(tokens);
        if current_token.is_none() {
            return false;
        };
        let current_token = current_token.unwrap();
        for token_type in token_types {
            if token_type == &current_token.type_ {
                self.current_index += 1;
                return true;
            };
        }

        false
    }

    fn current_token<'a>(&self, tokens: &'a [Token]) -> Option<&'a Token> {
        tokens.get(self.current_index)
    }

    pub(crate) fn synchronise(&mut self, tokens: &[Token]) {
        self.current_index += 1;

        while let Some(token) = self.current_token(tokens) {
            let previous = tokens.get(self.current_index - 1).unwrap();
            if previous.type_ == TokenType::Semicolon {
                return;
            };

            use TokenType::*;
            match token.type_ {
                Class | Fun | Var | For | If | While | Print | Return => return,
                _ => {}
            };

            self.current_index += 1;
        }
    }
}

pub enum Error {
    UnexpectedToken {
        expected_token_type: Option<TokenType>,
        unexpected_token_type: TokenType,
        span: Span,
    },
}

impl Error {
    pub fn display(&self, source: &str) {
        match self {
            Error::UnexpectedToken {
                expected_token_type: _,
                unexpected_token_type,
                span,
            } => lexer::Error::display_error(
                source,
                span,
                &format!("Unexpected token {:?}", unexpected_token_type),
            ),
        }
    }
}
