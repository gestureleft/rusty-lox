use std::rc::Rc;

use crate::{lexer::Token, span::Span};

#[derive(Debug)]
pub enum Expression {
    Assignment(AssignmentExpression),
    Binary(BinaryExpression),
    Call(CallExpression),
    Get(GetExpression),
    Grouping(GroupingExpression),
    Literal(LiteralExpression),
    Logical(LogicalExpression),
    Set(SetExpression),
    Super(SuperExpression),
    This(ThisExpression),
    Unary(UnaryExpression),
    Variable(VariableExpression),
}

impl Expression {
    pub fn prettify(&self, source: &str) -> String {
        match self {
            Expression::Assignment(_) => todo!(),
            Expression::Binary(binary_expression) => format!(
                "({} {} {})",
                binary_expression.operator.span.slice(source),
                binary_expression.left.prettify(source),
                binary_expression.right.prettify(source)
            ),
            Expression::Call(_) => todo!(),
            Expression::Get(_) => todo!(),
            Expression::Grouping(group) => {
                format!("(group {})", group.expression.prettify(source))
            }
            Expression::Literal(literal) => literal.prettify(source),
            Expression::Logical(_) => todo!(),
            Expression::Set(_) => todo!(),
            Expression::Super(_) => todo!(),
            Expression::This(_) => todo!(),
            Expression::Unary(unary_expression) => format!(
                "({} {})",
                unary_expression.operator.span.slice(source),
                unary_expression.right.prettify(source)
            ),
            Expression::Variable(_) => todo!(),
        }
    }

    pub fn span(&self) -> Span {
        match self {
            Expression::Assignment(AssignmentExpression { name, value: _ }) => name.span,
            Expression::Binary(BinaryExpression {
                left,
                right,
                operator,
            }) => left.span().combine(operator.span).combine(right.span()),
            Expression::Call(CallExpression {
                callee,
                closing_paren,
                arguments: _,
            }) => callee.span().combine(closing_paren.span),
            Expression::Get(GetExpression { object, name }) => object.span().combine(name.span),
            Expression::Grouping(_) => todo!(),
            Expression::Literal(literal_expression) => literal_expression.span(),
            Expression::Logical(_) => todo!(),
            Expression::Set(_) => todo!(),
            Expression::Super(_) => todo!(),
            Expression::This(_) => todo!(),
            Expression::Unary(_) => todo!(),
            Expression::Variable(VariableExpression { name }) => name.span,
        }
    }
}

pub fn binary_expression(
    left: Rc<Expression>,
    right: Rc<Expression>,
    operator: Token,
) -> Rc<Expression> {
    Rc::new(Expression::Binary(BinaryExpression::new(
        left, right, operator,
    )))
}

pub fn unary_expression(operator: Token, right: Rc<Expression>) -> Rc<Expression> {
    Rc::new(Expression::Unary(UnaryExpression::new(operator, right)))
}

pub fn number_literal_expression(value: Token) -> Rc<Expression> {
    Rc::new(Expression::Literal(LiteralExpression::Number(value)))
}

pub fn string_literal_expression(value: Token) -> Rc<Expression> {
    Rc::new(Expression::Literal(LiteralExpression::String_(value)))
}

pub fn boolean_literal_expression(span: Span, value: bool) -> Rc<Expression> {
    Rc::new(Expression::Literal(LiteralExpression::Boolean(span, value)))
}

pub fn nil_literal(span: Span) -> Rc<Expression> {
    Rc::new(Expression::Literal(LiteralExpression::Nil(span)))
}

pub fn grouping_expression(expression: Rc<Expression>) -> Rc<Expression> {
    Rc::new(Expression::Grouping(GroupingExpression { expression }))
}

#[derive(Debug)]
pub struct AssignmentExpression {
    pub name: Token,
    pub value: Rc<Expression>,
}

#[derive(Debug)]
pub struct BinaryExpression {
    pub left: Rc<Expression>,
    pub right: Rc<Expression>,
    pub operator: Token,
}

impl BinaryExpression {
    pub fn new(left: Rc<Expression>, right: Rc<Expression>, operator: Token) -> Self {
        Self {
            left,
            right,
            operator,
        }
    }
}

#[derive(Debug)]
pub struct CallExpression {
    pub callee: Rc<Expression>,
    pub closing_paren: Token,
    pub arguments: Vec<Rc<Expression>>,
}

#[derive(Debug)]
pub struct GetExpression {
    object: Rc<Expression>,
    name: Token,
}

#[derive(Debug)]
pub struct GroupingExpression {
    pub expression: Rc<Expression>,
}

#[derive(Debug)]
pub enum LiteralExpression {
    String_(Token),
    Number(Token),
    Boolean(Span, bool),
    Nil(Span),
}

impl LiteralExpression {
    fn prettify(&self, source: &str) -> String {
        match self {
            LiteralExpression::String_(token) => token.span.slice(source).into(),
            LiteralExpression::Number(token) => token.span.slice(source).into(),
            LiteralExpression::Boolean(_, boolean) => {
                if *boolean {
                    "true".into()
                } else {
                    "false".into()
                }
            }
            LiteralExpression::Nil(_) => "nil".into(),
        }
    }

    pub(crate) fn span(&self) -> Span {
        match self {
            LiteralExpression::String_(token) => token.span,
            LiteralExpression::Number(token) => token.span,
            LiteralExpression::Boolean(span, _) => *span,
            LiteralExpression::Nil(span) => *span,
        }
    }
}

#[derive(Debug)]
pub struct LogicalExpression {
    pub left: Rc<Expression>,
    pub right: Rc<Expression>,
    pub operator: Token,
}

#[derive(Debug)]
pub struct SetExpression {
    object: Rc<Expression>,
    name: Token,
    value: Rc<Expression>,
}

#[derive(Debug)]
pub struct SuperExpression {
    keyword: Token,
    method: Token,
}

#[derive(Debug)]
pub struct ThisExpression {
    keyword: Token,
}

#[derive(Debug)]
pub struct UnaryExpression {
    pub operator: Token,
    pub right: Rc<Expression>,
}

impl UnaryExpression {
    pub fn new(operator: Token, right: Rc<Expression>) -> Self {
        Self { operator, right }
    }
}

#[derive(Debug)]
pub struct VariableExpression {
    pub name: Token,
}
