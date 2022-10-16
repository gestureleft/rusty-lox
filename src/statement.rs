use std::rc::Rc;

use crate::{expression::Expression, lexer::Token};

#[derive(Debug)]
pub enum Declaration {
    Function {
        name: Token,
        parameters: Vec<Token>,
        body: Rc<Vec<Declaration>>,
    },
    Variable {
        name: Token,
        initialiser: Option<Rc<Expression>>,
    },
    Statement(Statement),
}

#[derive(Debug)]
pub enum Statement {
    Print(Rc<Expression>),
    Expression(Rc<Expression>),
    Block(Rc<Vec<Declaration>>),
    If {
        condition: Rc<Expression>,
        then_branch: Box<Statement>,
        else_branch: Option<Box<Statement>>,
    },
    While {
        condition: Rc<Expression>,
        body: Box<Statement>,
    },
}
