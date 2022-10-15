use crate::{lexer::Token, span::Span};

#[derive(Debug)]
pub enum Expression<'a> {
    Assignment(AssignmentExpression<'a>),
    Binary(BinaryExpression<'a>),
    Call(CallExpression<'a>),
    Get(GetExpression<'a>),
    Grouping(GroupingExpression<'a>),
    Literal(LiteralExpression<'a>),
    Logical(LogicalExpression<'a>),
    Set(SetExpression<'a>),
    Super(SuperExpression<'a>),
    This(ThisExpression<'a>),
    Unary(UnaryExpression<'a>),
    Variable(VariableExpression<'a>),
}

impl<'a> Expression<'a> {
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
                paren,
                arguments,
            }) => callee.span().combine(paren.span).combine(
                arguments
                    .iter()
                    .fold(paren.span, |acc, el| acc.combine(el.span())),
            ),
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

pub fn binary_expression<'a>(
    left: Expression<'a>,
    right: Expression<'a>,
    operator: &'a Token,
) -> Expression<'a> {
    Expression::Binary(BinaryExpression::new(left, right, operator))
}

pub fn unary_expression<'a>(operator: &'a Token, right: Expression<'a>) -> Expression<'a> {
    Expression::Unary(UnaryExpression::new(operator, right))
}

pub fn number_literal_expression<'a>(value: &'a Token) -> Expression<'a> {
    Expression::Literal(LiteralExpression::Number(value))
}

pub fn string_literal_expression<'a>(value: &'a Token) -> Expression<'a> {
    Expression::Literal(LiteralExpression::String_(value))
}

pub fn boolean_literal_expression<'a>(span: Span, value: bool) -> Expression<'a> {
    Expression::Literal(LiteralExpression::Boolean(span, value))
}

pub fn nil_literal<'a>(span: Span) -> Expression<'a> {
    Expression::Literal(LiteralExpression::Nil(span))
}

pub fn grouping_expression<'a>(expression: Expression<'a>) -> Expression<'a> {
    Expression::Grouping(GroupingExpression {
        expression: Box::new(expression),
    })
}

#[derive(Debug)]
pub struct AssignmentExpression<'a> {
    pub name: &'a Token,
    pub value: Box<Expression<'a>>,
}

#[derive(Debug)]
pub struct BinaryExpression<'a> {
    pub left: Box<Expression<'a>>,
    pub right: Box<Expression<'a>>,
    pub operator: &'a Token,
}

impl<'a> BinaryExpression<'a> {
    pub fn new(left: Expression<'a>, right: Expression<'a>, operator: &'a Token) -> Self {
        Self {
            left: Box::new(left),
            right: Box::new(right),
            operator,
        }
    }
}

#[derive(Debug)]
pub struct CallExpression<'a> {
    callee: Box<Expression<'a>>,
    paren: &'a Token,
    arguments: Vec<Expression<'a>>,
}

#[derive(Debug)]
pub struct GetExpression<'a> {
    object: Box<Expression<'a>>,
    name: &'a Token,
}

#[derive(Debug)]
pub struct GroupingExpression<'a> {
    pub expression: Box<Expression<'a>>,
}

#[derive(Debug)]
pub enum LiteralExpression<'a> {
    String_(&'a Token),
    Number(&'a Token),
    Boolean(Span, bool),
    Nil(Span),
}

impl<'a> LiteralExpression<'a> {
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
pub struct LogicalExpression<'a> {
    pub left: Box<Expression<'a>>,
    pub right: Box<Expression<'a>>,
    pub operator: &'a Token,
}

#[derive(Debug)]
pub struct SetExpression<'a> {
    object: Box<Expression<'a>>,
    name: &'a Token,
    value: Box<Expression<'a>>,
}

#[derive(Debug)]
pub struct SuperExpression<'a> {
    keyword: &'a Token,
    method: &'a Token,
}

#[derive(Debug)]
pub struct ThisExpression<'a> {
    keyword: &'a Token,
}

#[derive(Debug)]
pub struct UnaryExpression<'a> {
    pub operator: &'a Token,
    pub right: Box<Expression<'a>>,
}

impl<'a> UnaryExpression<'a> {
    pub fn new(operator: &'a Token, right: Expression<'a>) -> Self {
        Self {
            operator,
            right: Box::new(right),
        }
    }
}

#[derive(Debug)]
pub struct VariableExpression<'a> {
    pub name: &'a Token,
}
