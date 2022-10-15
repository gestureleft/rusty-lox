use crate::{
    expression::{
        binary_expression, boolean_literal_expression, grouping_expression, nil_literal,
        number_literal_expression, string_literal_expression, unary_expression,
        AssignmentExpression, Expression, VariableExpression,
    },
    lexer::{self, Token, TokenType},
    span::Span,
    statement::{Statement, VariableDeclaration},
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
    pub fn parse<'a>(tokens: &'a [Token]) -> ParserResult<'a> {
        let mut parser = Parser {
            current_index: 0,
            errors: vec![],
        };
        let mut statements = vec![];
        while let Some(token) = parser.current_token(tokens) && token.type_ != TokenType::Eof {
            let statement = parser.parse_declaration(tokens);
            if statement.is_none() {
                parser.synchronise(tokens);
                continue;
            }
            statements.push(statement.unwrap());
        }

        ParserResult {
            errors: parser.errors,
            statements,
        }
    }

    fn parse_declaration<'a>(&mut self, tokens: &'a [Token]) -> Option<Statement<'a>> {
        if self.consume_token_if_in_vec(tokens, &vec![TokenType::Var]) {
            return self.parse_variable_declaration(tokens);
        };

        self.parse_statement(tokens)
    }

    fn parse_variable_declaration<'a>(&mut self, tokens: &'a [Token]) -> Option<Statement<'a>> {
        let name = self
            .consume_token_of_type(tokens, TokenType::Identifier)?
            .clone();
        let initialiser = if self.consume_token_if_in_vec(tokens, &vec![TokenType::Equal]) {
            self.parse_expression(tokens)
        } else {
            None
        };
        self.consume_token_of_type(tokens, TokenType::Semicolon)?;
        Some(Statement::VariableDeclaration(VariableDeclaration {
            name,
            initialiser,
        }))
    }

    fn parse_statement<'a>(&mut self, tokens: &'a [Token]) -> Option<Statement<'a>> {
        if self.consume_token_if_in_vec(tokens, &vec![TokenType::Print]) {
            self.print_statement(tokens)
        } else if self.consume_token_if_in_vec(tokens, &vec![TokenType::LeftBrace]) {
            Some(Statement::Block(self.parse_block(tokens)?))
        } else {
            self.expression_statement(tokens)
        }
    }

    fn parse_block<'a>(&mut self, tokens: &'a [Token]) -> Option<Vec<Statement<'a>>> {
        let mut statements = vec![];
        while self.current_token(tokens).map(|t| &t.type_) != Some(&TokenType::RightBrace)
            && self.current_token(tokens).is_some()
        {
            statements.push(self.parse_declaration(tokens)?);
        }
        self.consume_token_of_type(tokens, TokenType::RightBrace)?;
        Some(statements)
    }

    fn print_statement<'a>(&mut self, tokens: &'a [Token]) -> Option<Statement<'a>> {
        let expression = self.parse_expression(tokens)?;
        self.consume_token_of_type(tokens, TokenType::Semicolon)?;
        Some(Statement::Print(expression))
    }

    fn expression_statement<'a>(&mut self, tokens: &'a [Token]) -> Option<Statement<'a>> {
        let expression = self.parse_expression(tokens)?;
        self.consume_token_of_type(tokens, TokenType::Semicolon)?;
        Some(Statement::Expression(expression))
    }

    fn parse_expression<'a>(&mut self, tokens: &'a [Token]) -> Option<Expression<'a>> {
        self.parse_assignment(tokens)
    }

    fn parse_assignment<'a>(&mut self, tokens: &'a [Token]) -> Option<Expression<'a>> {
        let expression = self.parse_equality(tokens)?;

        if self.consume_token_if_in_vec(tokens, &vec![TokenType::Equal]) {
            let value = self.parse_assignment(tokens)?;

            if let Expression::Variable(variable_expression) = expression {
                return Some(Expression::Assignment(AssignmentExpression {
                    name: variable_expression.name,
                    value: Box::new(value),
                }));
            };

            self.errors.push(Error::InvalidAssignmentTarget {
                target_span: expression.span(),
            });
        };

        Some(expression)
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
            let span = tokens.get(self.current_index - 1).unwrap().span;
            return Some(boolean_literal_expression(span, false));
        };
        if self.consume_token_if_in_vec(tokens, &vec![TokenType::True]) {
            let span = tokens.get(self.current_index - 1).unwrap().span;
            return Some(boolean_literal_expression(span, true));
        };
        if self.consume_token_if_in_vec(tokens, &vec![TokenType::Nil]) {
            let span = tokens.get(self.current_index - 1).unwrap().span;
            return Some(nil_literal(span));
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
        if self.consume_token_if_in_vec(tokens, &vec![TokenType::Identifier]) {
            return Some(Expression::Variable(VariableExpression {
                name: tokens.get(self.current_index - 1).unwrap(),
            }));
        }
        if self.consume_token_if_in_vec(tokens, &vec![TokenType::LeftParen]) {
            let expression = self.parse_expression(tokens)?;
            let current_token = self.current_token(tokens);

            if current_token.expect("Unexpectedly ran out of tokens").type_ != TokenType::RightParen
            {
                self.errors.push(Error::UnexpectedToken {
                    expected_token_type: Some(TokenType::RightParen),
                    unexpected_token_type: current_token.unwrap().type_.clone(),
                    span: current_token.unwrap().span,
                });
                return None;
            }

            self.current_index += 1;
            return Some(grouping_expression(expression));
        };

        self.errors.push(Error::UnexpectedToken {
            expected_token_type: None,
            unexpected_token_type: self.current_token(tokens).unwrap().type_.clone(),
            span: self.current_token(tokens).unwrap().span,
        });
        None
    }

    fn consume_token_of_type<'a>(
        &'a mut self,
        tokens: &'a [Token],
        token_type: TokenType,
    ) -> Option<&'a Token> {
        let current_token = self.current_token(tokens);
        if current_token.is_none() {
            self.errors.push(Error::UnexpectedEof);
            return None;
        };
        let current_token = current_token.unwrap();
        if current_token.type_ != token_type {
            println!(
                "Tryed to consume token of type {:?}, but got {:?} instead",
                token_type, current_token
            );
            self.errors.push(Error::UnexpectedToken {
                expected_token_type: Some(token_type),
                unexpected_token_type: current_token.type_.clone(),
                span: current_token.span,
            });
            return None;
        };
        self.current_index += 1;
        Some(current_token)
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
    UnexpectedEof,
    InvalidAssignmentTarget {
        target_span: Span,
    },
}

impl Error {
    pub fn display(&self, source: &str) {
        match self {
            Error::UnexpectedToken {
                expected_token_type,
                unexpected_token_type,
                span,
            } => {
                lexer::Error::display_error(
                    source,
                    span,
                    &format!("Unexpected token {:?}", unexpected_token_type),
                );
                if let Some(expected_token_type) = expected_token_type {
                    println!(
                        "  Info: \x1b[34mExpected token of type: {:?}\x1b[0m",
                        expected_token_type
                    );
                }
            }
            Error::UnexpectedEof => todo!(),
            Error::InvalidAssignmentTarget { target_span } => {
                lexer::Error::display_error(source, target_span, "Invalid assignment target")
            }
        }
    }
}
