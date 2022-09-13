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

/// Expressions reference tokens that are owned by the parser, they do this by indexing into the
/// parser's array of tokens
#[derive(Debug)]
pub struct TokenIndex(pub usize);

impl Expression {
    pub fn prettify(&self, source: &str, tokens: &Vec<Token>) -> String {
        match self {
            Expression::Assignment(_) => todo!(),
            Expression::Binary(binary_expression) => format!(
                "({} {} {})",
                tokens[binary_expression.operator.0].span.slice(source),
                binary_expression.left.prettify(source, tokens),
                binary_expression.right.prettify(source, tokens)
            ),
            Expression::Call(_) => todo!(),
            Expression::Get(_) => todo!(),
            Expression::Grouping(group) => {
                format!("(group {})", group.expression.prettify(source, tokens))
            }
            Expression::Literal(literal) => literal.prettify(source, tokens),
            Expression::Logical(_) => todo!(),
            Expression::Set(_) => todo!(),
            Expression::Super(_) => todo!(),
            Expression::This(_) => todo!(),
            Expression::Unary(unary_expression) => format!(
                "({} {})",
                tokens[unary_expression.operator.0].span.slice(source),
                unary_expression.right.prettify(source, tokens)
            ),
            Expression::Variable(_) => todo!(),
        }
    }
}

pub fn binary_expression(left: Expression, right: Expression, operator: TokenIndex) -> Expression {
    Expression::Binary(BinaryExpression::new(left, right, operator))
}

pub fn unary_expression(operator: TokenIndex, right: Expression) -> Expression {
    Expression::Unary(UnaryExpression::new(operator, right))
}

pub fn number_literal_expression(value: TokenIndex) -> Expression {
    Expression::Literal(LiteralExpression::Number(value))
}

pub fn string_literal_expression(value: TokenIndex) -> Expression {
    Expression::Literal(LiteralExpression::String_(value))
}

pub fn boolean_literal_expression(value: bool) -> Expression {
    Expression::Literal(LiteralExpression::Boolean(value))
}

pub fn nil_literal() -> Expression {
    Expression::Literal(LiteralExpression::Nil)
}

pub fn grouping_expression(expression: Expression) -> Expression {
    Expression::Grouping(GroupingExpression {
        expression: Box::new(expression),
    })
}

#[derive(Debug)]
pub struct AssignmentExpression {
    name: TokenIndex,
    value: Box<Expression>,
}

#[derive(Debug)]
pub struct BinaryExpression {
    left: Box<Expression>,
    right: Box<Expression>,
    operator: TokenIndex,
}

impl BinaryExpression {
    pub fn new(left: Expression, right: Expression, operator: TokenIndex) -> Self {
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
    paren: TokenIndex,
    arguments: Vec<Expression>,
}

#[derive(Debug)]
pub struct GetExpression {
    object: Box<Expression>,
    name: TokenIndex,
}

#[derive(Debug)]
pub struct GroupingExpression {
    expression: Box<Expression>,
}

#[derive(Debug)]
pub enum LiteralExpression {
    String_(TokenIndex),
    Number(TokenIndex),
    Boolean(bool),
    Nil,
}

impl LiteralExpression {
    fn prettify(&self, source: &str, tokens: &Vec<Token>) -> String {
        match self {
            LiteralExpression::String_(token) => tokens[token.0].span.slice(source).into(),
            LiteralExpression::Number(token) => tokens[token.0].span.slice(source).into(),
            LiteralExpression::Boolean(boolean) => {
                if *boolean {
                    "true".into()
                } else {
                    "false".into()
                }
            }
            LiteralExpression::Nil => "nil".into(),
        }
    }
}

#[derive(Debug)]
pub struct LogicalExpression {
    left: Box<Expression>,
    right: Box<Expression>,
    operator: TokenIndex,
}

#[derive(Debug)]
pub struct SetExpression {
    object: Box<Expression>,
    name: TokenIndex,
    value: Box<Expression>,
}

#[derive(Debug)]
pub struct SuperExpression {
    keyword: TokenIndex,
    method: TokenIndex,
}

#[derive(Debug)]
pub struct ThisExpression {
    keyword: TokenIndex,
}

#[derive(Debug)]
pub struct UnaryExpression {
    operator: TokenIndex,
    right: Box<Expression>,
}

impl UnaryExpression {
    pub fn new(operator: TokenIndex, right: Expression) -> Self {
        Self {
            operator,
            right: Box::new(right),
        }
    }
}

#[derive(Debug)]
pub struct VariableExpression {
    name: TokenIndex,
}
