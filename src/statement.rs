use crate::{expression::Expression, lexer::Token};

#[derive(Debug)]
pub enum Statement<'a> {
    Print(Expression<'a>),
    Expression(Expression<'a>),
    VariableDeclaration(VariableDeclaration<'a>),
    Block(Vec<Statement<'a>>),
    If {
        condition: Expression<'a>,
        then_branch: Box<Statement<'a>>,
        else_branch: Option<Box<Statement<'a>>>,
    },
}

#[derive(Debug)]
pub struct VariableDeclaration<'a> {
    pub name: Token,
    pub initialiser: Option<Expression<'a>>,
}
