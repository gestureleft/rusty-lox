use crate::{
    expression::{
        BinaryExpression, Expression, GroupingExpression, LiteralExpression, UnaryExpression,
        VariableExpression,
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
    environment: Environment,
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
            environment: Environment::new(),
        }
    }

    pub fn interpret(
        &mut self,
        source: &str,
        statements: Vec<Statement>,
    ) -> Result<Vec<Value>, Error> {
        self.evaluate_statements(source, &statements)
    }

    fn evaluate_statements(
        &mut self,
        source: &str,
        statements: &[Statement],
    ) -> Result<Vec<Value>, Error> {
        statements
            .iter()
            .map(|statement| self.evaluate_statement(source, statement))
            .collect::<Result<_, _>>()
    }

    fn evaluate_statement(&mut self, source: &str, statement: &Statement) -> Result<Value, Error> {
        match statement {
            Statement::Print(expression) => {
                let result = self.evaluate_expression(source, expression)?;
                result.pretty_print();
                Ok(result)
            }
            Statement::Expression(expression) => self.evaluate_expression(source, expression),
            Statement::VariableDeclaration(VariableDeclaration { name, initialiser }) => {
                let value = if let Some(initialiser) = initialiser {
                    self.evaluate_expression(source, initialiser)?
                } else {
                    Value::Nil(name.span)
                };
                self.environment
                    .define(name.span.slice(source).to_string(), value.clone());
                Ok(value)
            }
        }
    }

    fn evaluate_expression(&self, source: &str, expression: &Expression) -> Result<Value, Error> {
        match expression {
            Expression::Assignment(_) => todo!(),
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
            Expression::Logical(_) => todo!(),
            Expression::Set(_) => todo!(),
            Expression::Super(_) => todo!(),
            Expression::This(_) => todo!(),
            Expression::Unary(UnaryExpression { operator, right }) => {
                self.evaluate_unary_expression(source, operator, right)
            }
            Expression::Variable(VariableExpression { name }) => {
                let token = *name;
                self.environment
                    .get(source, token)
                    .ok_or(Error::VariableDoesntExist(token.clone()))
            }
        }
    }

    fn evaluate_unary_expression(
        &self,
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
                !self.is_truthy(right),
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

    fn is_truthy(&self, value: Value) -> bool {
        match value {
            Value::String(_, _) => todo!(),
            Value::Number(_, _) => todo!(),
            Value::Boolean(_, value) => value,
            Value::Nil(_) => false,
        }
    }

    fn evaluate_binary_expression(
        &self,
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
            Minus => Value::Number(
                span,
                self.as_number(self.evaluate_expression(source, left)?)?
                    - self.as_number(self.evaluate_expression(source, right)?)?,
            ),
            Plus => self.plus_or_concat(
                self.evaluate_expression(source, left)?,
                self.evaluate_expression(source, right)?,
            )?,
            Semicolon => todo!(),
            Slash => Value::Number(
                span,
                self.as_number(self.evaluate_expression(source, left)?)?
                    / self.as_number(self.evaluate_expression(source, right)?)?,
            ),
            Star => Value::Number(
                span,
                self.as_number(self.evaluate_expression(source, left)?)?
                    * self.as_number(self.evaluate_expression(source, right)?)?,
            ),
            Bang => todo!(),
            BangEqual => Value::Boolean(
                span,
                self.as_number(self.evaluate_expression(source, left)?)?
                    != self.as_number(self.evaluate_expression(source, right)?)?,
            ),
            Equal => todo!(),
            EqualEqual => Value::Boolean(
                span,
                self.as_number(self.evaluate_expression(source, left)?)?
                    == self.as_number(self.evaluate_expression(source, right)?)?,
            ),
            Greater => Value::Boolean(
                span,
                self.as_number(self.evaluate_expression(source, left)?)?
                    > self.as_number(self.evaluate_expression(source, right)?)?,
            ),
            GreaterEqual => Value::Boolean(
                span,
                self.as_number(self.evaluate_expression(source, left)?)?
                    >= self.as_number(self.evaluate_expression(source, right)?)?,
            ),
            Less => Value::Boolean(
                span,
                self.as_number(self.evaluate_expression(source, left)?)?
                    < self.as_number(self.evaluate_expression(source, right)?)?,
            ),
            LessEqual => Value::Boolean(
                span,
                self.as_number(self.evaluate_expression(source, left)?)?
                    < self.as_number(self.evaluate_expression(source, right)?)?,
            ),
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
