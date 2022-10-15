use crate::{
    expression::{
        binary_expression, boolean_literal_expression, grouping_expression, nil_literal,
        number_literal_expression, string_literal_expression, unary_expression,
        AssignmentExpression, Expression, LogicalExpression, VariableExpression,
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
        // If statement
        if self.consume_token_if_in_vec(tokens, &vec![TokenType::If]) {
            return self.parse_if_statement(tokens);
        };
        // While statement
        if self.consume_token_if_in_vec(tokens, &vec![TokenType::While]) {
            return self.parse_while_statement(tokens);
        };
        // For statement
        if self.consume_token_if_in_vec(tokens, &vec![TokenType::For]) {
            return self.parse_for_statement(tokens);
        };
        // Print statement
        if self.consume_token_if_in_vec(tokens, &vec![TokenType::Print]) {
            return self.parse_print_statement(tokens);
        };
        // Block statement
        if self.consume_token_if_in_vec(tokens, &vec![TokenType::LeftBrace]) {
            return Some(Statement::Block(self.parse_block(tokens)?));
        };

        // Expression statement
        self.parse_expression_statement(tokens)
    }

    fn parse_for_statement<'a>(&mut self, tokens: &'a [Token]) -> Option<Statement<'a>> {
        self.consume_token_of_type(tokens, TokenType::LeftParen)?;
        let initialiser = if self.consume_token_if_in_vec(tokens, &vec![TokenType::Semicolon]) {
            None
        } else if self.consume_token_if_in_vec(tokens, &vec![TokenType::Var]) {
            self.parse_variable_declaration(tokens)
        } else {
            self.parse_expression_statement(tokens)
        };

        let current_token = self.current_token(tokens)?;
        let condition = if current_token.type_ != TokenType::Semicolon {
            self.parse_expression(tokens)
        } else {
            None
        };

        self.consume_token_of_type(tokens, TokenType::Semicolon);

        let current_token = self.current_token(tokens)?;
        let increment = if current_token.type_ != TokenType::RightParen {
            self.parse_expression(tokens)
        } else {
            None
        };

        self.consume_token_of_type(tokens, TokenType::RightParen);

        let body = {
            let mut body = self.parse_statement(tokens)?;

            if let Some(increment) = increment {
                body = Statement::Block(vec![body, Statement::Expression(increment)]);
            }

            if let Some(condition) = condition {
                body = Statement::While {
                    condition,
                    body: Box::new(body),
                };
            }

            if let Some(initialiser) = initialiser {
                body = Statement::Block(vec![initialiser, body]);
            }
            body
        };

        Some(body)
    }

    fn parse_while_statement<'a>(&mut self, tokens: &'a [Token]) -> Option<Statement<'a>> {
        self.consume_token_of_type(tokens, TokenType::LeftParen)?;
        let condition = self.parse_expression(tokens)?;
        self.consume_token_of_type(tokens, TokenType::RightParen)?;

        let body = Box::new(self.parse_statement(tokens)?);

        Some(Statement::While { condition, body })
    }

    fn parse_if_statement<'a>(&mut self, tokens: &'a [Token]) -> Option<Statement<'a>> {
        self.consume_token_of_type(tokens, TokenType::LeftParen)?;
        let condition = self.parse_expression(tokens)?;
        self.consume_token_of_type(tokens, TokenType::RightParen)?;

        let then_branch = Box::new(self.parse_statement(tokens)?);
        let else_branch = if self.consume_token_if_in_vec(tokens, &vec![TokenType::Else]) {
            self.parse_statement(tokens).map(Box::new)
        } else {
            None
        };

        Some(Statement::If {
            condition,
            then_branch,
            else_branch,
        })
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

    fn parse_print_statement<'a>(&mut self, tokens: &'a [Token]) -> Option<Statement<'a>> {
        let expression = self.parse_expression(tokens)?;
        self.consume_token_of_type(tokens, TokenType::Semicolon)?;
        Some(Statement::Print(expression))
    }

    fn parse_expression_statement<'a>(&mut self, tokens: &'a [Token]) -> Option<Statement<'a>> {
        let expression = self.parse_expression(tokens)?;
        self.consume_token_of_type(tokens, TokenType::Semicolon)?;
        Some(Statement::Expression(expression))
    }

    fn parse_expression<'a>(&mut self, tokens: &'a [Token]) -> Option<Expression<'a>> {
        self.parse_assignment(tokens)
    }

    fn parse_assignment<'a>(&mut self, tokens: &'a [Token]) -> Option<Expression<'a>> {
        let expression = self.parse_or(tokens)?;

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

    fn parse_or<'a>(&mut self, tokens: &'a [Token]) -> Option<Expression<'a>> {
        let mut expression = self.parse_and(tokens)?;

        while self.consume_token_if_in_vec(tokens, &vec![TokenType::Or]) {
            let operator = tokens.get(self.current_index - 1).unwrap();
            let right = Box::new(self.parse_and(tokens)?);
            expression = Expression::Logical(LogicalExpression {
                left: Box::new(expression),
                right,
                operator,
            })
        }

        Some(expression)
    }

    fn parse_and<'a>(&mut self, tokens: &'a [Token]) -> Option<Expression<'a>> {
        let mut expression = self.parse_equality(tokens)?;

        while self.consume_token_if_in_vec(tokens, &vec![TokenType::And]) {
            let operator = tokens.get(self.current_index - 1).unwrap();
            let right = Box::new(self.parse_equality(tokens)?);
            expression = Expression::Logical(LogicalExpression {
                left: Box::new(expression),
                right,
                operator,
            });
        }

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
