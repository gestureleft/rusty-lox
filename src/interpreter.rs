use crate::{
    expression::{
        BinaryExpression, Expression, GroupingExpression, LiteralExpression, UnaryExpression,
    },
    lexer::{Token, TokenType},
    span::Span,
    statement::Statement,
};
use error::Error;
use value::Value;

mod error;
mod value;

#[derive(Debug)]
pub struct Interpreter<'a> {
    source: &'a str,
}

impl<'a> Interpreter<'a> {
    fn as_number(&self, value: Value) -> Result<f64, Error> {
        if let Value::Number(value) = value {
            return Ok(value);
        }

        Err(Error::type_error(
            "Number".into(),
            self.string_description(value),
        ))
    }

    fn string_description(&self, value: Value) -> String {
        match value {
            Value::String(_) => "String".into(),
            Value::Number(_) => "Number".into(),
            Value::Boolean(_) => "Boolean".into(),
            Value::Nil => "Nil".into(),
        }
    }
}

impl<'a> Interpreter<'a> {
    pub fn interpret(source: &'a str, statements: &'a [Statement<'a>]) {
        let interpreter = Self { source };
        for statement in statements {
            if let Statement::Expression(expression) = statement {
                interpreter.evaluate_expression(expression).unwrap();
            };
            if let Statement::Print(expression) = statement {
                interpreter.evaluate_expression(expression).unwrap();
            };
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
            Expression::Variable(_) => todo!(),
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
            Minus => Ok(Value::Number(-self.as_number(right)?)),
            Plus => todo!(),
            Semicolon => todo!(),
            Slash => todo!(),
            Star => todo!(),
            Bang => Ok(Value::Boolean(!self.is_truthy(right))),
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
                Span::new(value.span.start + 1, value.span.end - 1)
                    .slice(self.source)
                    .to_owned(),
            ),
            LiteralExpression::Number(value) => {
                Value::Number(value.span.slice(self.source).parse().unwrap_or_else(|_| {
                    panic!(
                        "Couldn't parse number literal {}",
                        value.span.slice(self.source)
                    )
                }))
            }
            LiteralExpression::Boolean(value) => Value::Boolean(*value),
            LiteralExpression::Nil => Value::Nil,
        })
    }

    fn is_truthy(&self, value: Value) -> bool {
        match value {
            Value::String(_) => todo!(),
            Value::Number(_) => todo!(),
            Value::Boolean(value) => value,
            Value::Nil => false,
        }
    }

    fn evaluate_binary_expression(
        &'a self,
        left: &'a Expression,
        right: &'a Expression,
        operator: &'a Token,
    ) -> Result<Value, Error> {
        use TokenType::*;
        Ok(match operator.type_ {
            LeftParen => todo!(),
            RightParen => todo!(),
            LeftBrace => todo!(),
            RightBrace => todo!(),
            Comma => todo!(),
            Dot => todo!(),
            Minus => Value::Number(
                self.as_number(self.evaluate_expression(left)?)?
                    - self.as_number(self.evaluate_expression(right)?)?,
            ),
            Plus => self.plus_or_concat(
                self.evaluate_expression(left)?,
                self.evaluate_expression(right)?,
            )?,
            Semicolon => todo!(),
            Slash => Value::Number(
                self.as_number(self.evaluate_expression(left)?)?
                    / self.as_number(self.evaluate_expression(right)?)?,
            ),
            Star => Value::Number(
                self.as_number(self.evaluate_expression(left)?)?
                    * self.as_number(self.evaluate_expression(right)?)?,
            ),
            Bang => todo!(),
            BangEqual => Value::Boolean(
                self.as_number(self.evaluate_expression(left)?)?
                    != self.as_number(self.evaluate_expression(right)?)?,
            ),
            Equal => todo!(),
            EqualEqual => Value::Boolean(
                self.as_number(self.evaluate_expression(left)?)?
                    == self.as_number(self.evaluate_expression(right)?)?,
            ),
            Greater => Value::Boolean(
                self.as_number(self.evaluate_expression(left)?)?
                    > self.as_number(self.evaluate_expression(right)?)?,
            ),
            GreaterEqual => Value::Boolean(
                self.as_number(self.evaluate_expression(left)?)?
                    >= self.as_number(self.evaluate_expression(right)?)?,
            ),
            Less => Value::Boolean(
                self.as_number(self.evaluate_expression(left)?)?
                    < self.as_number(self.evaluate_expression(right)?)?,
            ),
            LessEqual => Value::Boolean(
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
            Value::String(string) => Ok(string),
            Value::Number(_) => todo!(),
            Value::Boolean(_) => todo!(),
            Value::Nil => todo!(),
        }
    }

    fn plus_or_concat(&self, left: Value, right: Value) -> Result<Value, Error> {
        match left {
            Value::String(left) => Ok(Value::String(left + &self.as_string(right)?)),
            Value::Number(left) => Ok(Value::Number(left + self.as_number(right)?)),
            Value::Boolean(_) => todo!(),
            Value::Nil => todo!(),
        }
    }
}
