use crate::{
    expression::{
        AssignmentExpression, BinaryExpression, Expression, GroupingExpression, LiteralExpression,
        LogicalExpression, UnaryExpression, VariableExpression,
    },
    lexer::{Token, TokenType},
    span::Span,
    statement::{Statement, VariableDeclaration},
};
use error::Error;
use value::Value;

use self::environment::Environment;

mod environment;
mod error;
mod value;

#[derive(Debug)]
pub struct Interpreter {
    environment_stack: Vec<Environment>,
}

impl Interpreter {
    fn as_number(&self, value: Value) -> Result<f64, Error> {
        if let Value::Number(_, value) = value {
            return Ok(value);
        }

        let span = value.span();
        Err(Error::type_error(
            "Number".into(),
            self.string_description(value),
            span,
        ))
    }

    fn string_description(&self, value: Value) -> String {
        match value {
            Value::String(_, _) => "String".into(),
            Value::Number(_, _) => "Number".into(),
            Value::Boolean(_, _) => "Boolean".into(),
            Value::Nil(_) => "Nil".into(),
        }
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment_stack: vec![Environment::new()],
        }
    }

    fn current_scope(&mut self) -> &mut Environment {
        self.environment_stack.last_mut().unwrap()
    }

    fn assign(&mut self, name: String, new_value: Value) -> Result<(), ()> {
        for environment in self.environment_stack.iter_mut().rev() {
            let result = environment.assign(&name, &new_value);

            if result.is_ok() {
                return result;
            };
        }
        Err(())
    }

    fn get(&mut self, source: &str, token: &Token) -> Option<Value> {
        for environment in self.environment_stack.iter_mut().rev() {
            let result = environment.get(source, token);
            if result.is_some() {
                return result;
            };
        }
        None
    }

    pub fn interpret(&mut self, source: &str, statements: Vec<Statement>) -> Result<(), Error> {
        self.evaluate_statements(source, &statements)
    }

    fn evaluate_statements(&mut self, source: &str, statements: &[Statement]) -> Result<(), Error> {
        statements
            .iter()
            .try_for_each(|statement| self.evaluate_statement(source, statement))
    }

    fn evaluate_statement(&mut self, source: &str, statement: &Statement) -> Result<(), Error> {
        match statement {
            Statement::Print(expression) => {
                let result = self.evaluate_expression(source, expression)?;
                result.pretty_print();
            }
            Statement::Expression(expression) => {
                self.evaluate_expression(source, expression)?;
            }
            Statement::VariableDeclaration(VariableDeclaration { name, initialiser }) => {
                let value = if let Some(initialiser) = initialiser {
                    self.evaluate_expression(source, initialiser)?
                } else {
                    Value::Nil(name.span)
                };
                self.current_scope()
                    .define(name.span.slice(source).to_string(), value);
            }
            Statement::Block(statements) => {
                self.environment_stack.push(Environment::new());
                self.evaluate_statements(source, statements)?;
                self.environment_stack.pop();
            }
            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition = self.evaluate_expression(source, condition)?;
                if self.is_truthy(&condition) {
                    self.evaluate_statement(source, then_branch)?;
                } else if let Some(else_branch) = else_branch {
                    self.evaluate_statement(source, else_branch)?;
                }
            }
            Statement::While { condition, body } => {
                let mut condition_value = self.evaluate_expression(source, condition)?;
                while self.is_truthy(&condition_value) {
                    self.evaluate_statement(source, body)?;
                    condition_value = self.evaluate_expression(source, condition)?;
                }
            }
        };
        Ok(())
    }

    fn evaluate_expression(
        &mut self,
        source: &str,
        expression: &Expression,
    ) -> Result<Value, Error> {
        match expression {
            Expression::Assignment(AssignmentExpression { name, value }) => {
                let value = self.evaluate_expression(source, value)?;
                let did_assign = self.assign(name.span.slice(source).to_string(), value.clone());
                if did_assign.is_ok() {
                    return Ok(value);
                };
                Err(Error::VariableDoesntExist((*name).clone()))
            }
            Expression::Binary(BinaryExpression {
                left,
                right,
                operator,
            }) => self.evaluate_binary_expression(source, left, right, operator),
            Expression::Call(_) => todo!(),
            Expression::Get(_) => todo!(),
            Expression::Grouping(GroupingExpression { expression }) => {
                self.evaluate_expression(source, expression)
            }
            Expression::Literal(literal) => self.evaluate_literal(source, literal),
            Expression::Logical(LogicalExpression {
                left,
                right,
                operator,
            }) => {
                let left = self.evaluate_expression(source, left)?;
                if operator.type_ == TokenType::Or && self.is_truthy(&left) {
                    return Ok(left);
                }
                if operator.type_ == TokenType::And && !self.is_truthy(&left) {
                    return Ok(left);
                }
                self.evaluate_expression(source, right)
            }
            Expression::Set(_) => todo!(),
            Expression::Super(_) => todo!(),
            Expression::This(_) => todo!(),
            Expression::Unary(UnaryExpression { operator, right }) => {
                self.evaluate_unary_expression(source, operator, right)
            }
            Expression::Variable(VariableExpression { name }) => {
                let token = *name;
                self.get(source, token)
                    .ok_or(Error::VariableDoesntExist(token.clone()))
            }
        }
    }

    fn evaluate_unary_expression(
        &mut self,
        source: &str,
        operator: &Token,
        right: &Expression,
    ) -> Result<Value, Error> {
        use TokenType::*;
        let right = self.evaluate_expression(source, right)?;
        match operator.type_ {
            LeftParen => todo!(),
            RightParen => todo!(),
            LeftBrace => todo!(),
            RightBrace => todo!(),
            Comma => todo!(),
            Dot => todo!(),
            Minus => Ok(Value::Number(
                operator.span.combine(right.span()),
                -self.as_number(right)?,
            )),
            Plus => todo!(),
            Semicolon => todo!(),
            Slash => todo!(),
            Star => todo!(),
            Bang => Ok(Value::Boolean(
                operator.span.combine(right.span()),
                !self.is_truthy(&right),
            )),
            BangEqual => todo!(),
            Equal => todo!(),
            EqualEqual => todo!(),
            Greater => todo!(),
            GreaterEqual => todo!(),
            Less => todo!(),
            LessEqual => todo!(),
            Identifier => todo!(),
            String_ => todo!(),
            Number => todo!(),
            And => todo!(),
            Class => todo!(),
            Else => todo!(),
            False => todo!(),
            Fun => todo!(),
            For => todo!(),
            If => todo!(),
            Nil => todo!(),
            Or => todo!(),
            Print => todo!(),
            Return => todo!(),
            Super => todo!(),
            This => todo!(),
            True => todo!(),
            Var => todo!(),
            While => todo!(),
            Eof => todo!(),
        }
    }

    fn evaluate_literal(&self, source: &str, literal: &LiteralExpression) -> Result<Value, Error> {
        Ok(match literal {
            LiteralExpression::String_(value) => Value::String(
                value.span,
                Span::new(value.span.start + 1, value.span.end - 1)
                    .slice(source)
                    .to_owned(),
            ),
            LiteralExpression::Number(value) => Value::Number(
                value.span,
                value.span.slice(source).parse().unwrap_or_else(|_| {
                    panic!("Couldn't parse number literal {}", value.span.slice(source))
                }),
            ),
            LiteralExpression::Boolean(span, value) => Value::Boolean(*span, *value),
            LiteralExpression::Nil(span) => Value::Nil(*span),
        })
    }

    fn is_truthy(&self, value: &Value) -> bool {
        match *value {
            Value::String(_, _) => true,
            Value::Number(_, _) => true,
            Value::Boolean(_, value) => value,
            Value::Nil(_) => false,
        }
    }

    fn evaluate_binary_expression(
        &mut self,
        source: &str,
        left: &Expression,
        right: &Expression,
        operator: &Token,
    ) -> Result<Value, Error> {
        use TokenType::*;
        let span = left.span().combine(operator.span).combine(right.span());
        Ok(match operator.type_ {
            LeftParen => todo!(),
            RightParen => todo!(),
            LeftBrace => todo!(),
            RightBrace => todo!(),
            Comma => todo!(),
            Dot => todo!(),
            Minus => {
                let left = self.evaluate_expression(source, left)?;
                let left = self.as_number(left)?;
                let right = self.evaluate_expression(source, right)?;
                let right = self.as_number(right)?;
                Value::Number(span, left - right)
            }
            Plus => {
                let left = self.evaluate_expression(source, left)?;
                let right = self.evaluate_expression(source, right)?;
                self.plus_or_concat(left, right)?
            }
            Semicolon => todo!(),
            Slash => {
                let left = self.evaluate_expression(source, left)?;
                let left = self.as_number(left)?;
                let right = self.evaluate_expression(source, right)?;
                let right = self.as_number(right)?;
                Value::Number(span, left / right)
            }
            Star => {
                let left = self.evaluate_expression(source, left)?;
                let left = self.as_number(left)?;
                let right = self.evaluate_expression(source, right)?;
                let right = self.as_number(right)?;
                Value::Number(span, left * right)
            }
            Bang => todo!(),
            BangEqual => {
                let left = self.evaluate_expression(source, left)?;
                let left = self.as_number(left)?;
                let right = self.evaluate_expression(source, right)?;
                let right = self.as_number(right)?;
                Value::Boolean(span, left != right)
            }
            Equal => todo!(),
            EqualEqual => {
                let left = self.evaluate_expression(source, left)?;
                let left = self.as_number(left)?;
                let right = self.evaluate_expression(source, right)?;
                let right = self.as_number(right)?;
                Value::Boolean(span, left == right)
            }
            Greater => {
                let left = self.evaluate_expression(source, left)?;
                let left = self.as_number(left)?;
                let right = self.evaluate_expression(source, right)?;
                let right = self.as_number(right)?;
                Value::Boolean(span, left > right)
            }
            GreaterEqual => {
                let left = self.evaluate_expression(source, left)?;
                let left = self.as_number(left)?;
                let right = self.evaluate_expression(source, right)?;
                let right = self.as_number(right)?;
                Value::Boolean(span, left >= right)
            }
            Less => {
                let left = self.evaluate_expression(source, left)?;
                let left = self.as_number(left)?;
                let right = self.evaluate_expression(source, right)?;
                let right = self.as_number(right)?;
                Value::Boolean(span, left < right)
            }
            LessEqual => {
                let left = self.evaluate_expression(source, left)?;
                let left = self.as_number(left)?;
                let right = self.evaluate_expression(source, right)?;
                let right = self.as_number(right)?;
                Value::Boolean(span, left < right)
            }
            Identifier => todo!(),
            String_ => todo!(),
            Number => todo!(),
            And => todo!(),
            Class => todo!(),
            Else => todo!(),
            False => todo!(),
            Fun => todo!(),
            For => todo!(),
            If => todo!(),
            Nil => todo!(),
            Or => todo!(),
            Print => todo!(),
            Return => todo!(),
            Super => todo!(),
            This => todo!(),
            True => todo!(),
            Var => todo!(),
            While => todo!(),
            Eof => todo!(),
        })
    }

    fn as_string(&self, value: Value) -> Result<String, Error> {
        match value {
            Value::String(_, string) => Ok(string),
            Value::Number(span, _) => Err(Error::type_error(
                "String".to_string(),
                "Number".to_string(),
                span,
            )),
            Value::Boolean(_, _) => todo!(),
            Value::Nil(_) => todo!(),
        }
    }

    fn plus_or_concat(&self, left: Value, right: Value) -> Result<Value, Error> {
        match left {
            Value::String(left_span, left) => Ok(Value::String(
                left_span.combine(right.span()),
                left + &self.as_string(right)?,
            )),
            Value::Number(left_span, left) => Ok(Value::Number(
                left_span.combine(right.span()),
                left + self.as_number(right)?,
            )),
            Value::Boolean(_, _) => todo!(),
            Value::Nil(_) => todo!(),
        }
    }
}
