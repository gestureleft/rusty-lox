use std::rc::Rc;

use crate::{
    expression::{
        binary_expression, boolean_literal_expression, grouping_expression, nil_literal,
        number_literal_expression, string_literal_expression, unary_expression,
        AssignmentExpression, CallExpression, Expression, LogicalExpression, VariableExpression,
    },
    lexer::{self, Token, TokenType},
    span::Span,
    statement::{Declaration, Statement},
};

pub struct Parser {
    current_index: usize,
    errors: Vec<Error>,
}

pub struct ParserResult {
    pub errors: Vec<Error>,
    pub declarations: Vec<Declaration>,
}

impl Parser {
    pub fn parse(tokens: &[Token]) -> ParserResult {
        let mut parser = Parser {
            current_index: 0,
            errors: vec![],
        };
        let mut declarations = vec![];
        while let Some(token) = parser.current_token(tokens) && token.type_ != TokenType::Eof {
            let declaration = parser.parse_declaration(tokens);
            if declaration.is_none() {
                parser.synchronise(tokens);
                continue;
            }
            declarations.push(declaration.unwrap());
        }

        ParserResult {
            errors: parser.errors,
            declarations,
        }
    }

    fn parse_declaration(&mut self, tokens: &[Token]) -> Option<Declaration> {
        if self.consume_token_if_in_vec(tokens, &vec![TokenType::Fun]) {
            return self.parse_function_declaration(tokens);
        };
        if self.consume_token_if_in_vec(tokens, &vec![TokenType::Var]) {
            return self.parse_variable_declaration(tokens);
        };

        Some(Declaration::Statement(self.parse_statement(tokens)?))
    }

    fn parse_function_declaration(&mut self, tokens: &[Token]) -> Option<Declaration> {
        let name = self.consume_token_of_type(tokens, TokenType::Identifier)?;
        self.consume_token_of_type(tokens, TokenType::LeftParen)?;
        let mut parameters = Vec::new();
        if self.current_token(tokens)?.type_ != TokenType::RightParen {
            loop {
                // Make sure there's not too many arguments
                if parameters.len() >= 255 {
                    self.errors.push(Error::TwoManyArguments {
                        callee_span: name.span,
                    });
                    return None;
                };

                parameters.push(
                    self.consume_token_of_type(tokens, TokenType::Identifier)?
                        .clone(),
                );

                // If we don't find another comma, we're done parsing arguments
                if !self.consume_token_if_in_vec(tokens, &vec![TokenType::Comma]) {
                    break;
                }
            }
        }
        self.consume_token_of_type(tokens, TokenType::RightParen)?;

        self.consume_token_of_type(tokens, TokenType::LeftBrace)?;
        let body = self.parse_block(tokens)?;

        Some(Declaration::Function {
            name,
            parameters,
            body,
        })
    }

    fn parse_variable_declaration(&mut self, tokens: &[Token]) -> Option<Declaration> {
        let name = self.consume_token_of_type(tokens, TokenType::Identifier)?;
        let initialiser = if self.consume_token_if_in_vec(tokens, &vec![TokenType::Equal]) {
            self.parse_expression(tokens)
        } else {
            None
        };
        self.consume_token_of_type(tokens, TokenType::Semicolon)?;
        Some(Declaration::Variable { name, initialiser })
    }

    fn parse_statement(&mut self, tokens: &[Token]) -> Option<Statement> {
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

    fn parse_for_statement(&mut self, tokens: &[Token]) -> Option<Statement> {
        self.consume_token_of_type(tokens, TokenType::LeftParen)?;
        let initialiser = if self.consume_token_if_in_vec(tokens, &vec![TokenType::Semicolon]) {
            None
        } else if self.consume_token_if_in_vec(tokens, &vec![TokenType::Var]) {
            self.parse_variable_declaration(tokens)
        } else {
            Some(Declaration::Statement(
                self.parse_expression_statement(tokens)?,
            ))
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
                body = Statement::Block(Rc::new(vec![
                    Declaration::Statement(body),
                    Declaration::Statement(Statement::Expression(increment)),
                ]));
            }

            if let Some(condition) = condition {
                body = Statement::While {
                    condition,
                    body: Box::new(body),
                };
            }

            if let Some(initialiser) = initialiser {
                body = Statement::Block(Rc::new(vec![initialiser, Declaration::Statement(body)]));
            }
            body
        };

        Some(body)
    }

    fn parse_while_statement(&mut self, tokens: &[Token]) -> Option<Statement> {
        self.consume_token_of_type(tokens, TokenType::LeftParen)?;
        let condition = self.parse_expression(tokens)?;
        self.consume_token_of_type(tokens, TokenType::RightParen)?;

        let body = Box::new(self.parse_statement(tokens)?);

        Some(Statement::While { condition, body })
    }

    fn parse_if_statement(&mut self, tokens: &[Token]) -> Option<Statement> {
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

    fn parse_block(&mut self, tokens: &[Token]) -> Option<Rc<Vec<Declaration>>> {
        let mut statements = vec![];
        while self.current_token(tokens).map(|t| t.type_) != Some(TokenType::RightBrace)
            && self.current_token(tokens).is_some()
        {
            let declaration = self.parse_declaration(tokens)?;
            statements.push(declaration);
        }
        self.consume_token_of_type(tokens, TokenType::RightBrace)?;
        Some(Rc::new(statements))
    }

    fn parse_print_statement(&mut self, tokens: &[Token]) -> Option<Statement> {
        let expression = self.parse_expression(tokens)?;
        self.consume_token_of_type(tokens, TokenType::Semicolon)?;
        Some(Statement::Print(expression))
    }

    fn parse_expression_statement(&mut self, tokens: &[Token]) -> Option<Statement> {
        let expression = self.parse_expression(tokens)?;
        self.consume_token_of_type(tokens, TokenType::Semicolon)?;
        Some(Statement::Expression(expression))
    }

    fn parse_expression(&mut self, tokens: &[Token]) -> Option<Rc<Expression>> {
        self.parse_assignment(tokens)
    }

    fn parse_assignment(&mut self, tokens: &[Token]) -> Option<Rc<Expression>> {
        let expression = self.parse_or(tokens)?;

        if self.consume_token_if_in_vec(tokens, &vec![TokenType::Equal]) {
            let value = self.parse_assignment(tokens)?;

            if let Expression::Variable(variable_expression) = &*expression {
                return Some(Rc::new(Expression::Assignment(AssignmentExpression {
                    name: variable_expression.name.clone(),
                    value,
                })));
            };

            self.errors.push(Error::InvalidAssignmentTarget {
                target_span: expression.span(),
            });
        };

        Some(expression)
    }

    fn parse_or(&mut self, tokens: &[Token]) -> Option<Rc<Expression>> {
        let mut expression = self.parse_and(tokens)?;

        while self.consume_token_if_in_vec(tokens, &vec![TokenType::Or]) {
            let operator = tokens.get(self.current_index - 1).unwrap().clone();
            let right = self.parse_and(tokens)?;
            expression = Rc::new(Expression::Logical(LogicalExpression {
                left: expression,
                right,
                operator,
            }))
        }

        Some(expression)
    }

    fn parse_and(&mut self, tokens: &[Token]) -> Option<Rc<Expression>> {
        let mut expression = self.parse_equality(tokens)?;

        while self.consume_token_if_in_vec(tokens, &vec![TokenType::And]) {
            let operator = tokens.get(self.current_index - 1).unwrap().clone();
            let right = self.parse_equality(tokens)?;
            expression = Rc::new(Expression::Logical(LogicalExpression {
                left: expression,
                right,
                operator,
            }));
        }

        Some(expression)
    }

    fn parse_equality(&mut self, tokens: &[Token]) -> Option<Rc<Expression>> {
        let mut expression = self.parse_comparison(tokens)?;

        while self
            .consume_token_if_in_vec(tokens, &vec![TokenType::BangEqual, TokenType::EqualEqual])
        {
            let operator = tokens.get(self.current_index - 1).unwrap().clone();
            let right = self.parse_comparison(tokens)?;
            expression = binary_expression(expression, right, operator);
        }

        Some(expression)
    }

    fn parse_comparison(&mut self, tokens: &[Token]) -> Option<Rc<Expression>> {
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
            let operator = tokens.get(self.current_index - 1).unwrap().clone();
            let right = self.parse_term(tokens)?;
            expression = binary_expression(expression, right, operator);
        }

        Some(expression)
    }

    fn parse_term(&mut self, tokens: &[Token]) -> Option<Rc<Expression>> {
        let mut expression = self.parse_factor(tokens)?;

        while self.consume_token_if_in_vec(tokens, &vec![TokenType::Minus, TokenType::Plus]) {
            let operator = tokens.get(self.current_index - 1).unwrap().clone();
            let right = self.parse_factor(tokens)?;
            expression = binary_expression(expression, right, operator);
        }

        Some(expression)
    }

    fn parse_factor(&mut self, tokens: &[Token]) -> Option<Rc<Expression>> {
        let mut expression = self.parse_unary(tokens)?;

        while self.consume_token_if_in_vec(tokens, &vec![TokenType::Slash, TokenType::Star]) {
            let operator = tokens.get(self.current_index - 1).unwrap().clone();
            let right = self.parse_unary(tokens)?;
            expression = binary_expression(expression, right, operator);
        }

        Some(expression)
    }

    fn parse_unary(&mut self, tokens: &[Token]) -> Option<Rc<Expression>> {
        if self.consume_token_if_in_vec(tokens, &vec![TokenType::Bang, TokenType::Minus]) {
            let operator = tokens.get(self.current_index - 1).unwrap().clone();
            let right = self.parse_unary(tokens)?;
            return Some(unary_expression(operator, right));
        };

        self.parse_call(tokens)
    }

    fn parse_call(&mut self, tokens: &[Token]) -> Option<Rc<Expression>> {
        let mut expression = self.parse_primary(tokens)?;

        loop {
            if self.consume_token_if_in_vec(tokens, &vec![TokenType::LeftParen]) {
                expression = self.parse_call_arguments(tokens, expression)?;
            } else {
                break;
            }
        }

        Some(expression)
    }

    /// Given an expression being called, parse the arguments being passed to it
    /// (including the parens)
    fn parse_call_arguments(
        &mut self,
        tokens: &[Token],
        callee: Rc<Expression>,
    ) -> Option<Rc<Expression>> {
        let mut arguments = Vec::new();

        let current_token = self.current_token(tokens)?;
        if current_token.type_ != TokenType::RightParen {
            loop {
                arguments.push(self.parse_expression(tokens)?);
                if arguments.len() >= 255 {
                    self.errors.push(Error::TwoManyArguments {
                        callee_span: callee.span(),
                    })
                }
                if !self.consume_token_if_in_vec(tokens, &vec![TokenType::Comma]) {
                    break;
                }
            }
        }

        let closing_paren = self.consume_token_of_type(tokens, TokenType::RightParen)?;

        Some(Rc::new(Expression::Call(CallExpression {
            callee,
            closing_paren,
            arguments,
        })))
    }

    fn parse_primary(&mut self, tokens: &[Token]) -> Option<Rc<Expression>> {
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
                tokens.get(self.current_index - 1).unwrap().clone(),
            ));
        };
        if self.consume_token_if_in_vec(tokens, &vec![TokenType::String_]) {
            return Some(string_literal_expression(
                tokens.get(self.current_index - 1).unwrap().clone(),
            ));
        };
        if self.consume_token_if_in_vec(tokens, &vec![TokenType::Identifier]) {
            return Some(Rc::new(Expression::Variable(VariableExpression {
                name: tokens.get(self.current_index - 1).unwrap().clone(),
            })));
        }
        if self.consume_token_if_in_vec(tokens, &vec![TokenType::LeftParen]) {
            let expression = self.parse_expression(tokens)?;
            let current_token = self.current_token(tokens)?;

            if current_token.type_ != TokenType::RightParen {
                self.errors.push(Error::UnexpectedToken {
                    expected_token_type: Some(TokenType::RightParen),
                    unexpected_token_type: current_token.type_.clone(),
                    span: current_token.span,
                });
                return None;
            }

            self.current_index += 1;
            return Some(grouping_expression(expression));
        };

        self.errors.push(Error::UnexpectedToken {
            expected_token_type: None,
            unexpected_token_type: self.current_token(tokens).unwrap().type_,
            span: self.current_token(tokens).unwrap().span,
        });
        None
    }

    fn consume_token_of_type(&mut self, tokens: &[Token], token_type: TokenType) -> Option<Token> {
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

    fn current_token(&self, tokens: &[Token]) -> Option<Token> {
        tokens.get(self.current_index).cloned()
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
    TwoManyArguments {
        callee_span: Span,
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
            Error::TwoManyArguments { callee_span } => {
                lexer::Error::display_error(source, callee_span, "Too many arguments to call")
            }
        }
    }
}
