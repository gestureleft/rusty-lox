use crate::{
    expression::{
        binary_expression, boolean_literal_expression, grouping_expression, nil_literal,
        number_literal_expression, string_literal_expression, unary_expression, Expression,
        TokenIndex,
    },
    lexer::{self, Lexer, Token, TokenType},
};

pub struct Parser {
    current_index: usize,
    tokens: Vec<Token>,
    errors: Vec<Error>,
}

impl Parser {
    pub fn parse(source: &str) -> Vec<Error> {
        let lex_result = Lexer::lex(source);
        let mut parser = Parser {
            current_index: 0,
            tokens: lex_result.tokens,
            errors: lex_result
                .errors
                .iter()
                .map(|e| Error::Lexer(e.clone()))
                .collect(),
        };
        let expression = parser.parse_expression();
        if let Some(expression) = expression {
            println!(
                "parse() got expression: {:?}",
                expression.prettify(source, &parser.tokens)
            );
        }

        parser.errors
    }

    fn parse_expression(&mut self) -> Option<Expression> {
        self.parse_equality()
    }

    fn parse_equality(&mut self) -> Option<Expression> {
        let mut expression = self.parse_comparison()?;

        while self.consume_token_if_in_vec(&vec![TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = TokenIndex(self.current_index - 1);
            let right = self.parse_comparison()?;
            expression = binary_expression(expression, right, operator);
        }

        Some(expression)
    }
    fn parse_comparison(&mut self) -> Option<Expression> {
        let mut expression = self.parse_term()?;

        while self.consume_token_if_in_vec(&vec![
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = TokenIndex(self.current_index - 1);
            let right = self.parse_term()?;
            expression = binary_expression(expression, right, operator);
        }

        Some(expression)
    }

    fn parse_term(&mut self) -> Option<Expression> {
        let mut expression = self.parse_factor()?;

        while self.consume_token_if_in_vec(&vec![TokenType::Minus, TokenType::Plus]) {
            let operator = TokenIndex(self.current_index - 1);
            let right = self.parse_factor()?;
            expression = binary_expression(expression, right, operator);
        }

        Some(expression)
    }

    fn parse_factor(&mut self) -> Option<Expression> {
        let mut expression = self.parse_unary()?;

        while self.consume_token_if_in_vec(&vec![TokenType::Slash, TokenType::Star]) {
            let operator = TokenIndex(self.current_index - 1);
            let right = self.parse_unary()?;
            expression = binary_expression(expression, right, operator);
        }

        Some(expression)
    }

    fn parse_unary(&mut self) -> Option<Expression> {
        if self.consume_token_if_in_vec(&vec![TokenType::Bang, TokenType::Minus]) {
            let operator = TokenIndex(self.current_index - 1);
            let right = self.parse_unary()?;
            return Some(unary_expression(operator, right));
        };

        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Option<Expression> {
        if self.consume_token_if_in_vec(&vec![TokenType::False]) {
            return Some(boolean_literal_expression(false));
        };
        if self.consume_token_if_in_vec(&vec![TokenType::True]) {
            return Some(boolean_literal_expression(true));
        };
        if self.consume_token_if_in_vec(&vec![TokenType::Nil]) {
            return Some(nil_literal());
        };
        if self.consume_token_if_in_vec(&vec![TokenType::Number]) {
            return Some(number_literal_expression(TokenIndex(
                self.current_index - 1,
            )));
        };
        if self.consume_token_if_in_vec(&vec![TokenType::String_]) {
            return Some(string_literal_expression(TokenIndex(
                self.current_index - 1,
            )));
        };
        if self.consume_token_if_in_vec(&vec![TokenType::LeftParen, TokenType::RightParen]) {
            let expression = self.parse_expression()?;
            let current_token = self.current_token();

            if current_token.expect("Unexpectedly ran out of tokens").type_ != TokenType::RightParen
            {
                self.errors.push(Error::UnexpectedToken {
                    expected_token_type: Some(TokenType::RightParen),
                });
                panic!()
            }

            self.current_index += 1;
            return Some(grouping_expression(expression));
        };

        self.errors.push(Error::UnexpectedToken {
            expected_token_type: None,
        });
        None
    }

    /// If the current token's type is in the given list, consume it and return true.
    /// Else, do nothing and return false
    fn consume_token_if_in_vec(&mut self, token_types: &Vec<TokenType>) -> bool {
        let current_token = self.current_token();
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

    fn current_token(&self) -> Option<&Token> {
        self.tokens.get(self.current_index)
    }
}

pub enum Error {
    UnexpectedToken {
        expected_token_type: Option<TokenType>,
    },
    Lexer(lexer::Error),
}

impl Error {
    pub fn display(&self, source: &str) {
        match self {
            Error::UnexpectedToken {
                expected_token_type: _,
            } => todo!(),
            Error::Lexer(lexer_error) => lexer_error.display(source),
        }
    }
}
