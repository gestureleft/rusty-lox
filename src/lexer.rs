use std::collections::{HashMap, HashSet};

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
    errors: Vec<Error>,
    tokens: Vec<Token>,
}

#[derive(Debug, Clone)]
pub struct Token {
    span: Span,
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

#[derive(Debug)]
pub enum Error {
    UnterminatedStringLiteral { starting_at: usize },
    UnexpectedToken { at: usize },
}
