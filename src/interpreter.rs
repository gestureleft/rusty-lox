use std::rc::Rc;

use crate::{
    expression::{
        AssignmentExpression, BinaryExpression, CallExpression, Expression, GroupingExpression,
        LiteralExpression, LogicalExpression, UnaryExpression, VariableExpression,
    },
    lexer::{Token, TokenType},
    span::Span,
    statement::{Declaration, Statement},
};
use error::Error;
use value::Value;

use self::{environment::Environment, value::Callable};

mod environment;
mod error;
mod value;

#[derive(Debug)]
pub struct Interpreter {
    environment_stack: Vec<Environment>,
}

#[derive(Debug)]
enum ErrorOrReturn {
    Err(Error),
    Return(Rc<Value>),
}

impl Interpreter {
    fn as_number(&self, value: Rc<Value>) -> Result<f64, Error> {
        if let Value::Number(_, value) = *value {
            return Ok(value);
        }

        let span = value.span();
        Err(Error::type_error(
            "Number".into(),
            self.string_description(value),
            span,
        ))
    }

    fn string_description(&self, value: Rc<Value>) -> String {
        match &*value {
            Value::String(_, _) => "String".into(),
            Value::Number(_, _) => "Number".into(),
            Value::Boolean(_, _) => "Boolean".into(),
            Value::Nil(_) => "Nil".into(),
            Value::Callable { .. } => todo!(),
        }
    }

    pub fn new() -> Self {
        Self {
            environment_stack: vec![Environment::new()],
        }
    }

    fn current_scope(&mut self) -> &mut Environment {
        self.environment_stack.last_mut().unwrap()
    }

    fn assign(&mut self, name: String, new_value: Rc<Value>) -> Result<(), ()> {
        for environment in self.environment_stack.iter_mut().rev() {
            let result = environment.assign(&name, &new_value);

            if result.is_ok() {
                return result;
            };
        }
        Err(())
    }

    fn get(&mut self, source: &str, token: Token) -> Option<Rc<Value>> {
        for environment in self.environment_stack.iter_mut().rev() {
            let result = environment.get(source, &token);
            if result.is_some() {
                return result;
            };
        }
        None
    }

    pub fn interpret(&mut self, source: &str, declarations: Vec<Declaration>) -> Result<(), Error> {
        let result = self.evaluate_declarations(source, &declarations);
        match result {
            Ok(_) => Ok(()),
            Err(ErrorOrReturn::Return(_)) => Ok(()),
            Err(ErrorOrReturn::Err(error)) => Err(error),
        }
    }

    fn evaluate_declarations(
        &mut self,
        source: &str,
        declarations: &[Declaration],
    ) -> Result<(), ErrorOrReturn> {
        declarations
            .iter()
            .try_for_each(|declaration| self.evaluate_declaration(source, declaration))?;
        Ok(())
    }

    fn evaluate_declaration(
        &mut self,
        source: &str,
        declaration: &Declaration,
    ) -> Result<(), ErrorOrReturn> {
        match declaration {
            Declaration::Function {
                name,
                parameters,
                body,
            } => self.current_scope().define(
                name.span.slice(source).to_string(),
                Rc::new(Value::Callable(Callable {
                    name_span: name.span,
                    parameters: parameters
                        .iter()
                        .map(|token| token.span.slice(source).to_string())
                        .collect(),
                    body: body.clone(),
                })),
            ),
            Declaration::Variable { name, initialiser } => {
                let value = if let Some(initialiser) = initialiser {
                    self.evaluate_expression(source, initialiser.clone())
                        .map_err(ErrorOrReturn::Err)?
                } else {
                    Rc::new(Value::Nil(name.span))
                };
                self.current_scope()
                    .define(name.span.slice(source).to_string(), value);
            }
            Declaration::Statement(statement) => self.evaluate_statement(source, statement)?,
        };
        Ok(())
    }

    fn evaluate_statement(
        &mut self,
        source: &str,
        statement: &Statement,
    ) -> Result<(), ErrorOrReturn> {
        match statement {
            Statement::Print(expression) => {
                let result = self
                    .evaluate_expression(source, expression.clone())
                    .map_err(ErrorOrReturn::Err)?;
                result.pretty_print();
            }
            Statement::Expression(expression) => {
                self.evaluate_expression(source, expression.clone())
                    .map_err(ErrorOrReturn::Err)?;
            }
            Statement::Block(declarations) => {
                self.environment_stack.push(Environment::new());
                let result = self.evaluate_declarations(source, declarations);
                self.environment_stack.pop();
                result?;
            }
            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition = self
                    .evaluate_expression(source, condition.clone())
                    .map_err(ErrorOrReturn::Err)?;
                if self.is_truthy(condition) {
                    self.evaluate_statement(source, then_branch)?;
                } else if let Some(else_branch) = else_branch {
                    self.evaluate_statement(source, else_branch)?;
                }
            }
            Statement::While { condition, body } => {
                let mut condition_value = self
                    .evaluate_expression(source, condition.clone())
                    .map_err(ErrorOrReturn::Err)?;
                while self.is_truthy(condition_value) {
                    self.evaluate_statement(source, body)?;
                    condition_value = self
                        .evaluate_expression(source, condition.clone())
                        .map_err(ErrorOrReturn::Err)?
                }
            }
            Statement::Return { value, .. } => {
                let result = self
                    .evaluate_expression(source, value.clone())
                    .map_err(ErrorOrReturn::Err)?;
                return Err(ErrorOrReturn::Return(result));
            }
        };
        Ok(())
    }

    fn evaluate_expression(
        &mut self,
        source: &str,
        expression: Rc<Expression>,
    ) -> Result<Rc<Value>, Error> {
        match &*expression {
            Expression::Assignment(AssignmentExpression { name, value }) => {
                let value = self.evaluate_expression(source, value.clone())?;
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
            }) => self.evaluate_binary_expression(source, left.clone(), right.clone(), operator),
            Expression::Call(CallExpression {
                callee,
                arguments,
                closing_paren,
            }) => self.evaluate_call(source, callee.clone(), closing_paren.clone(), arguments),
            Expression::Get(_) => todo!(),
            Expression::Grouping(GroupingExpression { expression }) => {
                self.evaluate_expression(source, expression.clone())
            }
            Expression::Literal(literal) => self.evaluate_literal(source, literal),
            Expression::Logical(LogicalExpression {
                left,
                right,
                operator,
            }) => {
                let left = self.evaluate_expression(source, left.clone())?;
                if operator.type_ == TokenType::Or && self.is_truthy(left.clone()) {
                    return Ok(left);
                }
                if operator.type_ == TokenType::And && !self.is_truthy(left.clone()) {
                    return Ok(left);
                }
                self.evaluate_expression(source, right.clone())
            }
            Expression::Set(_) => todo!(),
            Expression::Super(_) => todo!(),
            Expression::This(_) => todo!(),
            Expression::Unary(UnaryExpression { operator, right }) => {
                self.evaluate_unary_expression(source, operator.clone(), right.clone())
            }
            Expression::Variable(VariableExpression { name }) => {
                let token = name;
                self.get(source, token.clone())
                    .ok_or(Error::VariableDoesntExist(token.clone()))
            }
        }
    }

    fn value_as_callable(&self, value: Rc<Value>, name_span: Span) -> Result<Callable, Error> {
        if let Value::Callable(callable) = &*value {
            Ok(callable.clone())
        } else {
            Err(Error::NotCallable(name_span))
        }
    }

    fn evaluate_call(
        &mut self,
        source: &str,
        callee: Rc<Expression>,
        closing_paren: Token,
        arguments: &[Rc<Expression>],
    ) -> Result<Rc<Value>, Error> {
        let callee_span = callee.span();
        let callee = self.evaluate_expression(source, callee)?;
        let callee = self.value_as_callable(callee, callee_span)?;

        if callee.parameters.len() != arguments.len() {
            return Err(Error::Arity {
                expected: callee.parameters.len(),
                got: arguments.len(),
                call_span: callee_span.combine(closing_paren.span),
            });
        };

        let mut argument_values = Vec::new();

        for argument in arguments {
            let argument_value = self.evaluate_expression(source, argument.clone())?;
            argument_values.push(argument_value);
        }

        self.environment_stack.push(Environment::new());
        for (paramater_name, argument) in callee.parameters.iter().zip(argument_values.iter()) {
            self.current_scope()
                .define(paramater_name.to_owned(), argument.clone())
        }
        let result = self.evaluate_declarations(source, &callee.body);
        self.environment_stack.pop();

        match result {
            Ok(_) => Ok(Rc::new(Value::Nil(Span::new(0, 0)))),
            Err(ErrorOrReturn::Return(value)) => Ok(value),
            Err(ErrorOrReturn::Err(error)) => Err(error),
        }
    }

    fn evaluate_unary_expression(
        &mut self,
        source: &str,
        operator: Token,
        right: Rc<Expression>,
    ) -> Result<Rc<Value>, Error> {
        use TokenType::*;
        let right = self.evaluate_expression(source, right)?;
        Ok(Rc::new(match operator.type_ {
            LeftParen => todo!(),
            RightParen => todo!(),
            LeftBrace => todo!(),
            RightBrace => todo!(),
            Comma => todo!(),
            Dot => todo!(),
            Minus => Value::Number(operator.span.combine(right.span()), -self.as_number(right)?),
            Plus => todo!(),
            Semicolon => todo!(),
            Slash => todo!(),
            Star => todo!(),
            Bang => Value::Boolean(operator.span.combine(right.span()), !self.is_truthy(right)),
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
        }))
    }

    fn evaluate_literal(
        &self,
        source: &str,
        literal: &LiteralExpression,
    ) -> Result<Rc<Value>, Error> {
        Ok(Rc::new(match literal {
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
        }))
    }

    fn is_truthy(&self, value: Rc<Value>) -> bool {
        match &*value {
            Value::String(_, _) => true,
            Value::Number(_, _) => true,
            Value::Boolean(_, value) => *value,
            Value::Nil(_) => false,
            Value::Callable { .. } => true,
        }
    }

    fn evaluate_binary_expression(
        &mut self,
        source: &str,
        left: Rc<Expression>,
        right: Rc<Expression>,
        operator: &Token,
    ) -> Result<Rc<Value>, Error> {
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
                Rc::new(Value::Number(span, left - right))
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
                Rc::new(Value::Number(span, left / right))
            }
            Star => {
                let left = self.evaluate_expression(source, left)?;
                let left = self.as_number(left)?;
                let right = self.evaluate_expression(source, right)?;
                let right = self.as_number(right)?;
                Rc::new(Value::Number(span, left * right))
            }
            Bang => todo!(),
            BangEqual => {
                let left = self.evaluate_expression(source, left)?;
                let left = self.as_number(left)?;
                let right = self.evaluate_expression(source, right)?;
                let right = self.as_number(right)?;
                Rc::new(Value::Boolean(span, left != right))
            }
            Equal => todo!(),
            EqualEqual => {
                let left = self.evaluate_expression(source, left)?;
                let left = self.as_number(left)?;
                let right = self.evaluate_expression(source, right)?;
                let right = self.as_number(right)?;
                Rc::new(Value::Boolean(span, left == right))
            }
            Greater => {
                let left = self.evaluate_expression(source, left)?;
                let left = self.as_number(left)?;
                let right = self.evaluate_expression(source, right)?;
                let right = self.as_number(right)?;
                Rc::new(Value::Boolean(span, left > right))
            }
            GreaterEqual => {
                let left = self.evaluate_expression(source, left)?;
                let left = self.as_number(left)?;
                let right = self.evaluate_expression(source, right)?;
                let right = self.as_number(right)?;
                Rc::new(Value::Boolean(span, left >= right))
            }
            Less => {
                let left = self.evaluate_expression(source, left)?;
                let left = self.as_number(left)?;
                let right = self.evaluate_expression(source, right)?;
                let right = self.as_number(right)?;
                Rc::new(Value::Boolean(span, left < right))
            }
            LessEqual => {
                let left = self.evaluate_expression(source, left)?;
                let left = self.as_number(left)?;
                let right = self.evaluate_expression(source, right)?;
                let right = self.as_number(right)?;
                Rc::new(Value::Boolean(span, left <= right))
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

    fn as_string(&self, value: Rc<Value>) -> Result<String, Error> {
        match &*value {
            Value::String(_, string) => Ok(string.to_owned()),
            Value::Number(span, _) => Err(Error::type_error(
                "String".to_string(),
                "Number".to_string(),
                *span,
            )),
            Value::Boolean(_, _) => todo!(),
            Value::Nil(_) => todo!(),
            Value::Callable { .. } => todo!(),
        }
    }

    fn plus_or_concat(&self, left: Rc<Value>, right: Rc<Value>) -> Result<Rc<Value>, Error> {
        Ok(Rc::new(match &*left {
            Value::String(left_span, left) => Value::String(
                left_span.combine(right.span()),
                left.to_owned() + &self.as_string(right)?,
            ),
            Value::Number(left_span, left) => Value::Number(
                left_span.combine(right.span()),
                left + self.as_number(right)?,
            ),
            Value::Boolean(_, _) => todo!(),
            Value::Nil(_) => todo!(),
            Value::Callable { .. } => todo!(),
        }))
    }
}
