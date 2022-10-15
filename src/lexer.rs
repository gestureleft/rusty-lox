use std::collections::HashMap;

use crate::span::Span;

pub struct Lexer<'a> {
    source: &'a str,
    current_position: usize,
    errors: Vec<Error>,
    tokens: Vec<Token>,
}

impl<'a> Lexer<'a> {
    pub fn lex(input: &'a str) -> Result {
        let mut lexer = Lexer {
            source: input,
            current_position: 0,
            errors: vec![],
            tokens: vec![],
        };

        loop {
            let next_result = lexer.next();
            if next_result == NextResult::Done {
                break;
            };
        }

        Result {
            tokens: lexer.tokens,
            errors: lexer.errors,
        }
    }

    fn current_character(&self) -> Option<char> {
        self.source.chars().nth(self.current_position)
    }

    fn absorb_single_character_token(&mut self, token_type: TokenType) {
        self.tokens.push(Token::new(
            self.current_position,
            self.current_position + 1,
            token_type,
        ));
        self.current_position += 1;
    }

    /// Given a character, if the current character matches it, absorb
    /// the current character and return true. Else, return false without
    /// absorbing.
    fn absorb_if_match(&mut self, character_to_match: char) -> bool {
        match self.current_character() {
            Some(current_character) if current_character == character_to_match => {
                self.current_position += 1;
                true
            }
            _ => false,
        }
    }

    fn next(&mut self) -> NextResult {
        use TokenType::*;
        loop {
            match self.current_character() {
                Some('\n') => {
                    self.current_position += 1;
                }
                Some('!') => {
                    self.current_position += 1;
                    let is_bang_equal = self.absorb_if_match('=');
                    self.tokens.push(if is_bang_equal {
                        Token::new(self.current_position - 2, self.current_position, BangEqual)
                    } else {
                        Token::new(self.current_position - 1, self.current_position, Bang)
                    });
                    return NextResult::NotDone;
                }
                Some('=') => {
                    self.current_position += 1;
                    let is_equal_equal = self.absorb_if_match('=');
                    self.tokens.push(if is_equal_equal {
                        Token::new(self.current_position - 2, self.current_position, EqualEqual)
                    } else {
                        Token::new(self.current_position - 1, self.current_position, Equal)
                    });
                    return NextResult::NotDone;
                }
                Some('>') => {
                    self.current_position += 1;
                    let is_greater_equal = self.absorb_if_match('=');
                    self.tokens.push(if is_greater_equal {
                        Token::new(
                            self.current_position - 2,
                            self.current_position,
                            GreaterEqual,
                        )
                    } else {
                        Token::new(self.current_position - 1, self.current_position, Greater)
                    });
                    return NextResult::NotDone;
                }
                Some('<') => {
                    self.current_position += 1;
                    let is_less_equal = self.absorb_if_match('=');
                    self.tokens.push(if is_less_equal {
                        Token::new(self.current_position - 2, self.current_position, LessEqual)
                    } else {
                        Token::new(self.current_position - 1, self.current_position, Less)
                    });
                    return NextResult::NotDone;
                }
                Some('/') => {
                    self.current_position += 1;
                    let is_comment = self.absorb_if_match('/');
                    if is_comment {
                        self.absorb_until_newline();
                        return NextResult::NotDone;
                    };
                    self.tokens.push(Token::new(
                        self.current_position - 1,
                        self.current_position,
                        Slash,
                    ));
                    return NextResult::NotDone;
                }
                Some(character) => {
                    let token_type = TokenType::from_character(character);
                    if let Some(token_type) = token_type {
                        self.absorb_single_character_token(token_type);
                        return NextResult::NotDone;
                    };

                    if character == ' '
                        || character == '\n'
                        || character == '\r'
                        || character == '\t'
                    {
                        self.current_position += 1;
                        return NextResult::NotDone;
                    }

                    if character == '"' {
                        self.lex_string();
                        return NextResult::NotDone;
                    }

                    if is_digit(character) {
                        self.lex_number();
                        return NextResult::NotDone;
                    }

                    if character.is_ascii() && character.is_alphabetic() {
                        self.lex_identifier_or_keyword();
                        return NextResult::NotDone;
                    }

                    self.errors.push(Error::UnexpectedToken {
                        at: self.current_position,
                    });
                    self.current_position += 1;
                }
                None => {
                    self.tokens.push(Token::new(
                        self.current_position,
                        self.current_position + 1,
                        TokenType::Eof,
                    ));
                    return NextResult::Done;
                }
            };
        }
    }

    fn lex_identifier_or_keyword(&mut self) {
        let identifier_start = self.current_position;
        let mut current_character = self.current_character();

        let keywords: HashMap<&'static str, TokenType> = [
            ("and", TokenType::And),
            ("class", TokenType::Class),
            ("else", TokenType::Else),
            ("false", TokenType::False),
            ("for", TokenType::For),
            ("fun", TokenType::Fun),
            ("if", TokenType::If),
            ("nil", TokenType::Nil),
            ("or", TokenType::Or),
            ("print", TokenType::Print),
            ("return", TokenType::Return),
            ("super", TokenType::Super),
            ("this", TokenType::This),
            ("true", TokenType::True),
            ("var", TokenType::Var),
            ("while", TokenType::While),
        ]
        .into();

        while current_character.is_some()
            && current_character.unwrap().is_ascii()
            && (current_character.unwrap().is_alphabetic()
                || current_character.unwrap().is_numeric())
        {
            self.current_position += 1;
            current_character = self.current_character();
        }

        let identifier_or_keyword =
            Span::new(identifier_start, self.current_position).slice(self.source);

        if keywords.contains_key(identifier_or_keyword) {
            self.tokens.push(Token::new(
                identifier_start,
                self.current_position,
                keywords.get(identifier_or_keyword).unwrap().clone(),
            ));
            return;
        }

        self.tokens.push(Token::new(
            identifier_start,
            self.current_position,
            TokenType::Identifier,
        ));
    }

    fn lex_number(&mut self) {
        let number_start = self.current_position;
        let mut current_character = self.current_character();

        while current_character.is_some() && is_digit(current_character.unwrap()) {
            self.current_position += 1;
            current_character = self.current_character();
        }

        let next_character = self.source.chars().nth(self.current_position + 1);
        if current_character == Some('.')
            && next_character.is_some()
            && is_digit(next_character.unwrap())
        {
            self.current_position += 1;
            let mut current_character = self.current_character();
            while current_character.is_some() && is_digit(current_character.unwrap()) {
                self.current_position += 1;
                current_character = self.current_character();
            }
        }

        self.tokens.push(Token::new(
            number_start,
            self.current_position,
            TokenType::Number,
        ));
    }

    fn lex_string(&mut self) {
        let string_start = self.current_position;
        assert!(self.absorb_if_match('"'));
        let mut current_character = self.current_character();

        while current_character.is_some() && current_character != Some('"') {
            self.current_position += 1;
            current_character = self.current_character();
        }

        if current_character.is_none() {
            self.errors.push(Error::UnterminatedStringLiteral {
                starting_at: string_start,
            });
            return;
        }

        assert!(self.absorb_if_match('"'));
        self.tokens.push(Token::new(
            string_start,
            self.current_position,
            TokenType::String_,
        ));
    }

    /// Ignore the rest of the line
    fn absorb_until_newline(&mut self) {
        let mut current_character = self.current_character();
        while current_character.is_some() && current_character != Some('\n') {
            self.current_position += 1;
            current_character = self.current_character();
        }
    }
}

fn is_digit(character: char) -> bool {
    matches!(
        character,
        '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9'
    )
}

#[derive(PartialEq)]
enum NextResult {
    Done,
    NotDone,
}

#[derive(Debug)]
pub struct Result {
    pub errors: Vec<Error>,
    pub tokens: Vec<Token>,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub span: Span,
    pub type_: TokenType,
}

impl Token {
    pub fn new(span_start: usize, span_end: usize, type_: TokenType) -> Self {
        Self {
            type_,
            span: Span::new(span_start, span_end),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String_,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

impl TokenType {
    pub fn from_character(character: char) -> Option<Self> {
        use TokenType::*;
        match character {
            '(' => Some(LeftParen),
            ')' => Some(RightParen),
            '{' => Some(LeftBrace),
            '}' => Some(RightBrace),
            ',' => Some(Comma),
            '.' => Some(Dot),
            '-' => Some(Minus),
            '+' => Some(Plus),
            ';' => Some(Semicolon),
            '*' => Some(Star),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Error {
    UnterminatedStringLiteral { starting_at: usize },
    UnexpectedToken { at: usize },
}

struct LinesForErrorDisplay {
    pub line_before: Option<Span>,
    pub line: Span,
    // line_after: Span,
    pub line_number_of_error: usize,
}

impl Error {
    fn lines_for_error_display(source: &str, error_starts_at: usize) -> LinesForErrorDisplay {
        // Index into the source the start of the current line
        let mut previous_line_start = 0;
        let mut current_line_start = 0;
        let mut line_number = 1;
        for (index, value) in source.chars().enumerate() {
            if index == error_starts_at {
                break;
            };
            if value == '\n' && index < source.len() - 1 {
                previous_line_start = current_line_start;
                current_line_start = index + 1;
                line_number += 1;
            };
        }
        let next_new_line = {
            let mut i = error_starts_at;
            let mut current_char = source.chars().nth(i);
            while current_char.is_some() && current_char != Some('\n') {
                i += 1;
                current_char = source.chars().nth(i);
            }
            i
        };

        let line_before = {
            if line_number == 1 {
                None
            } else {
                Some(Span::new(previous_line_start, current_line_start))
            }
        };

        LinesForErrorDisplay {
            line_before,
            line: Span::new(current_line_start, next_new_line),
            line_number_of_error: line_number,
        }
    }

    pub fn display(&self, source: &str) {
        match self {
            Error::UnterminatedStringLiteral { starting_at } => Self::display_error(
                source,
                &Span::new(
                    *starting_at,
                    Self::index_of_first_new_line_after(source, *starting_at),
                ),
                "Unterminated String Literal",
            ),
            Error::UnexpectedToken { at } => {
                Self::display_error(source, &Span::new(*at, *at + 1), "Unexpected token")
            }
        }
    }

    /// Given some source and an index, return the index of the next newline after the given index in the source
    fn index_of_first_new_line_after(source: &str, index: usize) -> usize {
        let mut i = index;
        let mut current_char = source.chars().nth(i);
        while current_char.is_some() && current_char != Some('\n') {
            i += 1;
            current_char = source.chars().nth(i);
        }
        i
    }

    pub(crate) fn display_error<'a>(source: &'a str, span: &Span, error: &'a str) {
        let lines = Error::lines_for_error_display(source, span.start);

        println!("\n  \x1b[31mError:\x1b[0m {}\n", error);
        if let Some(line_before) = lines.line_before {
            // FIXME: We may need padding here if the number of digits in `line_number - 1` is
            // less than `line_number`
            print!(
                " \x1b[34m{}\x1b[0m |  {}",
                lines.line_number_of_error - 1,
                line_before.slice(source)
            )
        }

        println!(
            " \x1b[34m{}\x1b[0m |  {}",
            lines.line_number_of_error,
            lines.line.slice(source)
        );

        // FIXME: The amount of padding here should be dependent on the width of `line_number`
        println!(
            "      \x1b[31m{}{}=== {}\x1b[0m",
            (0..span.start - lines.line.start)
                .map(|_| ' ')
                .collect::<String>(),
            (0..span.end - span.start).map(|_| '^').collect::<String>(),
            error
        );
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn number() {
        let lex_result = Lexer::lex("2.34");
        assert_eq!(lex_result.errors.len(), 0);
        assert_eq!(lex_result.tokens.len(), 2);
        assert_eq!(lex_result.tokens[0].type_, TokenType::Number);
        assert_eq!(lex_result.tokens[0].span.start, 0);
        assert_eq!(lex_result.tokens[0].span.end, 4);
        assert_eq!(lex_result.tokens[1].type_, TokenType::Eof);
        assert_eq!(lex_result.tokens[1].span.start, 4);
        assert_eq!(lex_result.tokens[1].span.end, 5);
    }

    #[test]
    fn bad_number() {
        let lex_result = Lexer::lex("2.");
        assert_eq!(lex_result.errors.len(), 0);
        assert_eq!(lex_result.tokens.len(), 3);
        assert_eq!(lex_result.tokens[0].type_, TokenType::Number);
        assert_eq!(lex_result.tokens[1].type_, TokenType::Dot);
        assert_eq!(lex_result.tokens[2].type_, TokenType::Eof);

        let lex_result = Lexer::lex(".2");
        assert_eq!(lex_result.errors.len(), 0);
        assert_eq!(lex_result.tokens.len(), 3);
        assert_eq!(lex_result.tokens[0].type_, TokenType::Dot);
        assert_eq!(lex_result.tokens[1].type_, TokenType::Number);
        assert_eq!(lex_result.tokens[2].type_, TokenType::Eof);
    }

    #[test]
    fn string() {
        let lex_result = Lexer::lex(r#" "hello" "#);
        assert_eq!(lex_result.errors.len(), 0);
        assert_eq!(lex_result.tokens.len(), 2);
        assert_eq!(lex_result.tokens[0].type_, TokenType::String_);
        assert_eq!(lex_result.tokens[1].type_, TokenType::Eof);
        assert_eq!(lex_result.tokens[0].span.start, 1);
        assert_eq!(lex_result.tokens[0].span.end, 8);
    }
}
