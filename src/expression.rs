use crate::lexer::Token;

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
            Expression::Grouping(group) => format!("(group {})", group.expression.prettify(source)),
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
}

pub fn binary_expression(left: Expression, right: Expression, operator: Token) -> Expression {
    Expression::Binary(BinaryExpression::new(left, right, operator))
}

pub fn unary_expression(operator: Token, right: Expression) -> Expression {
    Expression::Unary(UnaryExpression::new(operator, right))
}

pub fn number_literal_expression(value: Token) -> Expression {
    Expression::Literal(LiteralExpression::Number(value))
}

pub fn grouping_expression(expression: Expression) -> Expression {
    Expression::Grouping(GroupingExpression {
        expression: Box::new(expression),
    })
}

#[derive(Debug)]
pub struct AssignmentExpression {
    name: Token,
    value: Box<Expression>,
}

#[derive(Debug)]
pub struct BinaryExpression {
    left: Box<Expression>,
    right: Box<Expression>,
    operator: Token,
}

impl BinaryExpression {
    pub fn new(left: Expression, right: Expression, operator: Token) -> Self {
        Self {
            left: Box::new(left),
            right: Box::new(right),
            operator,
        }
    }
}

#[derive(Debug)]
pub struct CallExpression {
    callee: Box<Expression>,
    paren: Token,
    arguments: Vec<Expression>,
}

#[derive(Debug)]
pub struct GetExpression {
    object: Box<Expression>,
    name: Token,
}

#[derive(Debug)]
pub struct GroupingExpression {
    expression: Box<Expression>,
}

#[derive(Debug)]
pub enum LiteralExpression {
    String_(Token),
    Number(Token),
}

impl LiteralExpression {
    fn prettify(&self, source: &str) -> String {
        match self {
            LiteralExpression::String_(token) => token.span.slice(source).into(),
            LiteralExpression::Number(token) => token.span.slice(source).into(),
        }
    }
}

#[derive(Debug)]
pub struct LogicalExpression {
    left: Box<Expression>,
    right: Box<Expression>,
    operator: Token,
}

#[derive(Debug)]
pub struct SetExpression {
    object: Box<Expression>,
    name: Token,
    value: Box<Expression>,
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
    operator: Token,
    right: Box<Expression>,
}

impl UnaryExpression {
    pub fn new(operator: Token, right: Expression) -> Self {
        Self {
            operator,
            right: Box::new(right),
        }
    }
}

#[derive(Debug)]
pub struct VariableExpression {
    name: Token,
}
