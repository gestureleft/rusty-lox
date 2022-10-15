use crate::{expression::Expression, lexer::Token};

#[derive(Debug)]
pub enum Statement<'a> {
    Print(Expression<'a>),
    Expression(Expression<'a>),
    VariableDeclaration(VariableDeclaration<'a>),
    Block(Vec<Statement<'a>>),
}

#[derive(Debug)]
pub struct VariableDeclaration<'a> {
    pub name: Token,
    pub initialiser: Option<Expression<'a>>,
}
