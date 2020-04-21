use std::collections::HashMap;

use crate::error;
use crate::value::Value;

pub fn line_error(line: usize, message: &str) {
    error::report(line, "", message);
}

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,

    start: usize,
    current: usize,
    line: usize,

    keywords: HashMap<String, TokenType>,
}

impl Scanner {
    pub fn new(source: String) -> Scanner {
        let mut keywords = HashMap::new();
        keywords.insert("and".to_string(), TokenType::And);
        keywords.insert("class".to_string(), TokenType::Class);
        keywords.insert("else".to_string(), TokenType::Else);
        keywords.insert("false".to_string(), TokenType::False);
        keywords.insert("for".to_string(), TokenType::For);
        keywords.insert("fun".to_string(), TokenType::Fun);
        keywords.insert("if".to_string(), TokenType::If);
        keywords.insert("nil".to_string(), TokenType::Nil);
        keywords.insert("or".to_string(), TokenType::Or);
        keywords.insert("print".to_string(), TokenType::Print);
        keywords.insert("return".to_string(), TokenType::Return);
        keywords.insert("super".to_string(), TokenType::Super);
        keywords.insert("this".to_string(), TokenType::This);
        keywords.insert("true".to_string(), TokenType::True);
        keywords.insert("var".to_string(), TokenType::Var);
        keywords.insert("while".to_string(), TokenType::While);

        Scanner {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 0,
            keywords,
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens
            .push(Token::new(TokenType::Eof, String::new(), None, self.line));

        self.tokens.clone()
    }

    pub fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::Semicolon),
            '*' => self.add_token(TokenType::Star),
            '!' => {
                let token = if self.match_('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };
                self.add_token(token)
            }
            '=' => {
                let token = if self.match_('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };
                self.add_token(token)
            }
            '<' => {
                let token = if self.match_('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };
                self.add_token(token)
            }
            '>' => {
                let token = if self.match_('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };
                self.add_token(token)
            }
            '/' => {
                if self.match_('/') {
                    // A comment goes to the end of the line
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash)
                }
            }
            '"' => self.string(),
            ' ' => {}
            '\r' => {}
            '\t' => {}
            '\n' => self.line += 1,

            _ => {
                if Scanner::is_digit(c) {
                    self.number()
                } else if Scanner::is_alpha(c) {
                    self.identifier()
                } else {
                    line_error(self.line, "Unexpected character")
                }
            }
        }
    }

    fn match_(&mut self, c: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source.chars().nth(self.current).unwrap_or('\0') != c {
            return false;
        }

        self.current += 1;
        true
    }

    fn peek(&mut self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source.chars().nth(self.current).unwrap_or('\0')
        }
    }

    fn peek_next(&mut self) -> char {
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            self.source.chars().nth(self.current + 1).unwrap_or('\0')
        }
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.source.chars().nth(self.current - 1).unwrap_or(' ')
    }

    fn is_at_end(&self) -> bool {
        self.current == self.source.len()
    }

    fn add_token(&mut self, type_: TokenType) {
        self.add_token_value(type_, None);
    }

    fn add_token_value(&mut self, type_: TokenType, value: Option<Value>) {
        let len = self.current - self.start;
        let text = self.source.chars().skip(self.start).take(len).collect();
        self.tokens.push(Token::new(type_, text, value, self.line));
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1
            }
            self.advance();
        }

        // Check for unterminated string
        if self.is_at_end() {
            line_error(self.line, "Unterminated string.");
            return;
        }

        // The closing "
        self.advance();

        let len = self.current - self.start;
        let value = self
            .source
            .chars()
            .skip(self.start + 1)
            .take(len - 2)
            .collect();
        self.add_token_value(TokenType::String, Some(Value::String(value)));
    }

    fn number(&mut self) {
        while Scanner::is_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == '.' && Scanner::is_digit(self.peek_next()) {
            self.advance();

            while Scanner::is_digit(self.peek()) {
                self.advance();
            }
        }

        let len = self.current - self.start;
        let text: String = self.source.chars().skip(self.start).take(len).collect();
        self.add_token_value(
            TokenType::Number,
            Some(Value::Double(text.parse::<f64>().unwrap())),
        )
    }

    fn identifier(&mut self) {
        while Scanner::is_alpha_numeric(self.peek()) {
            self.advance();
        }

        let len = self.current - self.start;
        let text: String = self.source.chars().skip(self.start).take(len).collect();

        let token = match self.keywords.get(&text) {
            Some(token) => token.clone(),
            None => TokenType::Identifier,
        };

        self.add_token(token)
    }

    // Stand alone

    fn is_digit(c: char) -> bool {
        c >= '0' && c <= '9'
    }

    fn is_alpha(c: char) -> bool {
        (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_'
    }

    fn is_alpha_numeric(c: char) -> bool {
        Scanner::is_alpha(c) || Scanner::is_digit(c)
    }
}

#[derive(Debug, Clone, PartialEq)]
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

    // one or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // literals.
    Identifier,
    String,
    Number,

    // // keywords.
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

impl std::string::ToString for TokenType {
    fn to_string(&self) -> String {
        match self {
            // Single-character tokens.
            TokenType::LeftParen => "(".to_string(),
            TokenType::RightParen => ")".to_string(),
            TokenType::LeftBrace => "{".to_string(),
            TokenType::RightBrace => "}".to_string(),
            TokenType::Comma => ",".to_string(),
            TokenType::Dot => ".".to_string(),
            TokenType::Minus => "-".to_string(),
            TokenType::Plus => "+".to_string(),
            TokenType::Semicolon => ";".to_string(),
            TokenType::Slash => "/".to_string(),
            TokenType::Star => "*".to_string(),

            // one or two character tokens.
            TokenType::Bang => "!".to_string(),
            TokenType::BangEqual => "!=".to_string(),
            TokenType::Equal => "=".to_string(),
            TokenType::EqualEqual => "==".to_string(),
            TokenType::Greater => ">".to_string(),
            TokenType::GreaterEqual => ">=".to_string(),
            TokenType::Less => "<".to_string(),
            TokenType::LessEqual => "<=".to_string(),

            // literals.
            TokenType::Identifier => "".to_string(),
            TokenType::String => "".to_string(),
            TokenType::Number => "".to_string(),

            // keywords.
            TokenType::And => "".to_string(),
            TokenType::Class => "".to_string(),
            TokenType::Else => "".to_string(),
            TokenType::False => "".to_string(),
            TokenType::Fun => "".to_string(),
            TokenType::For => "".to_string(),
            TokenType::If => "".to_string(),
            TokenType::Nil => "".to_string(),
            TokenType::Or => "".to_string(),
            TokenType::Print => "".to_string(),
            TokenType::Return => "".to_string(),
            TokenType::Super => "".to_string(),
            TokenType::This => "".to_string(),
            TokenType::True => "".to_string(),
            TokenType::Var => "".to_string(),
            TokenType::While => "".to_string(),
            TokenType::Eof => "".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub type_: TokenType,
    pub lexeme: String,
    pub literal: Option<Value>,
    pub line: usize,
}

pub type TokenVec = Vec<Token>;

impl Token {
    pub fn new(type_: TokenType, lexeme: String, literal: Option<Value>, line: usize) -> Token {
        Token {
            type_,
            lexeme,
            literal,
            line,
        }
    }
}
