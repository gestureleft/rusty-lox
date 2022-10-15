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
pub struct Interpreter<'a> {
    source: &'a str,
    environment: Environment<'a>,
}

impl<'a> Interpreter<'a> {
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

impl<'a> Interpreter<'a> {
    pub fn interpret(
        source: &'a str,
        statements: &'a [Statement<'a>],
    ) -> Result<Vec<Value>, Error> {
        Self {
            source,
            environment: Environment::new(),
        }
        .evaluate_statements(source, statements)
    }

    fn evaluate_statements(
        &mut self,
        source: &'a str,
        statements: &'a [Statement<'a>],
    ) -> Result<Vec<Value>, Error> {
        statements
            .iter()
            .map(|statement| self.evaluate_statement(source, statement))
            .collect::<Result<_, _>>()
    }

    fn evaluate_statement(
        &mut self,
        source: &'a str,
        statement: &'a Statement,
    ) -> Result<Value, Error> {
        match statement {
            Statement::Print(expression) => {
                let result = self.evaluate_expression(expression)?;
                result.pretty_print();
                Ok(result)
            }
            Statement::Expression(expression) => self.evaluate_expression(expression),
            Statement::VariableDeclaration(VariableDeclaration { name, initialiser }) => {
                let value = if let Some(initialiser) = initialiser {
                    self.evaluate_expression(initialiser)?
                } else {
                    Value::Nil(name.span)
                };
                self.environment
                    .define(name.span.slice(source), value.clone());
                Ok(value)
            }
        }
    }

    fn evaluate_expression(&self, expression: &'a Expression<'a>) -> Result<Value, Error> {
        match expression {
            Expression::Assignment(_) => todo!(),
            Expression::Binary(BinaryExpression {
                left,
                right,
                operator,
            }) => self.evaluate_binary_expression(left, right, operator),
            Expression::Call(_) => todo!(),
            Expression::Get(_) => todo!(),
            Expression::Grouping(GroupingExpression { expression }) => {
                self.evaluate_expression(expression)
            }
            Expression::Literal(literal) => self.evaluate_literal(literal),
            Expression::Logical(_) => todo!(),
            Expression::Set(_) => todo!(),
            Expression::Super(_) => todo!(),
            Expression::This(_) => todo!(),
            Expression::Unary(UnaryExpression { operator, right }) => {
                self.evaluate_unary_expression(operator, right)
            }
            Expression::Variable(VariableExpression { name }) => {
                let token = *name;
                self.environment
                    .get(self.source, token)
                    .ok_or(Error::VariableDoesntExist(token.clone()))
            }
        }
    }

    fn evaluate_unary_expression(
        &self,
        operator: &'a Token,
        right: &'a Expression<'a>,
    ) -> Result<Value, Error> {
        use TokenType::*;
        let right = self.evaluate_expression(right)?;
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

    fn evaluate_literal(&self, literal: &'a LiteralExpression) -> Result<Value, Error> {
        Ok(match literal {
            LiteralExpression::String_(value) => Value::String(
                value.span,
                Span::new(value.span.start + 1, value.span.end - 1)
                    .slice(self.source)
                    .to_owned(),
            ),
            LiteralExpression::Number(value) => Value::Number(
                value.span,
                value.span.slice(self.source).parse().unwrap_or_else(|_| {
                    panic!(
                        "Couldn't parse number literal {}",
                        value.span.slice(self.source)
                    )
                }),
            ),
            LiteralExpression::Boolean(span, value) => Value::Boolean(span.clone(), *value),
            LiteralExpression::Nil(span) => Value::Nil(span.clone()),
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
        &'a self,
        left: &'a Expression,
        right: &'a Expression,
        operator: &'a Token,
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
                self.as_number(self.evaluate_expression(left)?)?
                    - self.as_number(self.evaluate_expression(right)?)?,
            ),
            Plus => self.plus_or_concat(
                self.evaluate_expression(left)?,
                self.evaluate_expression(right)?,
            )?,
            Semicolon => todo!(),
            Slash => Value::Number(
                span,
                self.as_number(self.evaluate_expression(left)?)?
                    / self.as_number(self.evaluate_expression(right)?)?,
            ),
            Star => Value::Number(
                span,
                self.as_number(self.evaluate_expression(left)?)?
                    * self.as_number(self.evaluate_expression(right)?)?,
            ),
            Bang => todo!(),
            BangEqual => Value::Boolean(
                span,
                self.as_number(self.evaluate_expression(left)?)?
                    != self.as_number(self.evaluate_expression(right)?)?,
            ),
            Equal => todo!(),
            EqualEqual => Value::Boolean(
                span,
                self.as_number(self.evaluate_expression(left)?)?
                    == self.as_number(self.evaluate_expression(right)?)?,
            ),
            Greater => Value::Boolean(
                span,
                self.as_number(self.evaluate_expression(left)?)?
                    > self.as_number(self.evaluate_expression(right)?)?,
            ),
            GreaterEqual => Value::Boolean(
                span,
                self.as_number(self.evaluate_expression(left)?)?
                    >= self.as_number(self.evaluate_expression(right)?)?,
            ),
            Less => Value::Boolean(
                span,
                self.as_number(self.evaluate_expression(left)?)?
                    < self.as_number(self.evaluate_expression(right)?)?,
            ),
            LessEqual => Value::Boolean(
                span,
                self.as_number(self.evaluate_expression(left)?)?
                    < self.as_number(self.evaluate_expression(right)?)?,
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
