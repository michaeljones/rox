use crate::error;
use crate::scanner;
use crate::scanner::Token;
use crate::scanner::TokenType;
use crate::value::Value;

pub fn token_error(token: &Token, message: &String) {
    if token.type_ == TokenType::Eof {
        error::report(token.line, " at end", message);
    } else {
        let string = format!(" at '{}'", token.lexeme);
        error::report(token.line, &string, message);
    }
}

pub enum Stmt {
    Block(Vec<Stmt>),
    Expression(Expr),
    Print(Expr),
    Var(Token, Option<Expr>),
}

pub enum Expr {
    Binary(Box<Expr>, Token, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(Value),
    Unary(Token, Box<Expr>),
    Variable(Token),
    Assign(Token, Box<Expr>),
}

// Printer

impl std::string::ToString for Expr {
    fn to_string(&self) -> String {
        match self {
            Expr::Binary(left, operator, right) => format!(
                "({} {} {})",
                operator.type_.to_string(),
                left.to_string(),
                right.to_string()
            ),
            Expr::Grouping(inner_expr) => format!("(group {})", inner_expr.to_string()),
            Expr::Literal(value) => match value {
                Value::String(string) => format!("\"{}\"", string.clone()),
                Value::Double(double) => double.to_string(),
                Value::Bool(boolean) => boolean.to_string(),
                Value::Nil => "nil".to_string(),
            },
            Expr::Unary(operator, inner_expr) => format!(
                "({} {})",
                operator.type_.to_string(),
                inner_expr.to_string()
            ),
            Expr::Variable(name_token) => format!("{}", name_token.lexeme),
            Expr::Assign(name_token, expr) => {
                format!("{} = {}", name_token.lexeme, expr.to_string())
            }
        }
    }
}

#[derive(Debug)]
pub enum ParserError {
    UnmatchedPrimary,
    UnexpectedTokenError,
    InvalidAssignment,
}

pub struct Parser {
    tokens: scanner::TokenVec,
    current: usize,
}

impl Parser {
    pub fn new(tokens: scanner::TokenVec) -> Parser {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, ParserError> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            match self.declaration() {
                Ok(statement) => statements.push(statement),
                Err(err) => return Err(err),
            }
        }

        Ok(statements)
    }

    fn declaration(&mut self) -> Result<Stmt, ParserError> {
        let result = if self.match_(&vec![TokenType::Var]) {
            self.var_declaration()
        } else {
            self.statement()
        };

        match result {
            Ok(_) => result,
            Err(_) => {
                self.synchronize();
                result
            }
        }
    }

    fn var_declaration(&mut self) -> Result<Stmt, ParserError> {
        let name = self.consume(&TokenType::Identifier, "Expect variable name".to_string());

        let mut initializer = Ok(None);
        if self.match_(&vec![TokenType::Equal]) {
            initializer = self.expression().map(Some);
        }

        let consume_result = self.consume(
            &TokenType::Semicolon,
            "Expect ';' after variable declaration".to_string(),
        );

        result_map3(name, initializer, consume_result, |n, i, _| Stmt::Var(n, i))
    }

    fn statement(&mut self) -> Result<Stmt, ParserError> {
        if self.match_(&vec![TokenType::Print]) {
            self.print_statement()
        } else if self.match_(&vec![TokenType::LeftBrace]) {
            self.block().map(|statements| Stmt::Block(statements))
        } else {
            self.expression_statement()
        }
    }

    fn block(&mut self) -> Result<Vec<Stmt>, ParserError> {
        let mut statements = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            match self.declaration() {
                Ok(statement) => statements.push(statement),
                Err(err) => return Err(err),
            }
        }

        let result = self.consume(&TokenType::RightBrace, "Expect '}' after block".to_string());

        result.map(|_| statements)
    }

    fn print_statement(&mut self) -> Result<Stmt, ParserError> {
        let value = self.expression();
        let result = self.consume(&TokenType::Semicolon, "Expect ';' after value".to_string());

        result_map2(value, result, |value, _| Stmt::Print(value))
    }

    fn expression_statement(&mut self) -> Result<Stmt, ParserError> {
        let value = self.expression();
        let result = self.consume(&TokenType::Semicolon, "Expect ';' after value".to_string());

        result_map2(value, result, |value, _| Stmt::Expression(value))
    }

    fn expression(&mut self) -> Result<Expr, ParserError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, ParserError> {
        let expr = self.equality();

        if self.match_(&vec![TokenType::Equal]) {
            let _equals = self.previous();
            let value = self.assignment();

            return match expr {
                Ok(Expr::Variable(name_token)) => {
                    value.map(|value| Expr::Assign(name_token, Box::new(value)))
                }
                Ok(_) => Err(ParserError::InvalidAssignment),
                err @ Err(_) => err,
            };
        }

        expr
    }

    fn equality(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.comparison();

        while self.match_(&vec![TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous();
            let right = self.comparison();
            expr = result_map2(expr, right, |l, r| {
                Expr::Binary(Box::new(l), operator, Box::new(r))
            });
        }

        expr
    }

    fn comparison(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.addition();

        let tokens = vec![
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ];
        while self.match_(&tokens) {
            let operator = self.previous();
            let right = self.addition();
            expr = result_map2(expr, right, |l, r| {
                Expr::Binary(Box::new(l), operator, Box::new(r))
            });
        }

        expr
    }

    fn addition(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.multiplication();

        let tokens = vec![TokenType::Minus, TokenType::Plus];
        while self.match_(&tokens) {
            let operator = self.previous();
            let right = self.multiplication();
            expr = result_map2(expr, right, |l, r| {
                Expr::Binary(Box::new(l), operator, Box::new(r))
            });
        }

        expr
    }

    fn multiplication(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.unary();

        let tokens = vec![TokenType::Slash, TokenType::Star];
        while self.match_(&tokens) {
            let operator = self.previous();
            let right = self.unary();
            expr = result_map2(expr, right, |l, r| {
                Expr::Binary(Box::new(l), operator, Box::new(r))
            });
        }

        expr
    }

    fn unary(&mut self) -> Result<Expr, ParserError> {
        let tokens = vec![TokenType::Bang, TokenType::Minus];
        while self.match_(&tokens) {
            let operator = self.previous();
            let right = self.unary();
            return right.map(|r| Expr::Unary(operator, Box::new(r)));
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, ParserError> {
        if self.match_(&vec![TokenType::False]) {
            return Ok(Expr::Literal(Value::Bool(false)));
        }
        if self.match_(&vec![TokenType::True]) {
            return Ok(Expr::Literal(Value::Bool(true)));
        }
        if self.match_(&vec![TokenType::Nil]) {
            return Ok(Expr::Literal(Value::Nil));
        }
        if self.match_(&vec![TokenType::Number, TokenType::String]) {
            return Ok(Expr::Literal(self.previous().literal.unwrap()));
        }
        if self.match_(&vec![TokenType::Identifier]) {
            return Ok(Expr::Variable(self.previous()));
        }
        if self.match_(&vec![TokenType::LeftParen]) {
            let expr = self.expression();
            let result = self.consume(
                &TokenType::RightParen,
                "Expect ')' after expression".to_string(),
            );
            return result_map2(expr, result, |expr, _| Expr::Grouping(Box::new(expr)));
        }

        Err(ParserError::UnmatchedPrimary)
    }

    fn consume(&mut self, type_: &TokenType, message: String) -> Result<Token, ParserError> {
        if self.check(type_) {
            return Ok(self.advance());
        }

        self.error(&self.peek(), &message);
        Err(ParserError::UnexpectedTokenError)
    }

    fn error(&self, token: &Token, message: &String) {
        token_error(token, message);
    }

    fn match_(&mut self, token_types: &Vec<TokenType>) -> bool {
        for type_ in token_types {
            if self.check(type_) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&mut self, type_: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().type_ == *type_
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().type_ == TokenType::Eof
    }

    fn peek(&self) -> Token {
        self.tokens.iter().nth(self.current).unwrap().clone()
    }

    fn previous(&self) -> Token {
        self.tokens.iter().nth(self.current - 1).unwrap().clone()
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().type_ == TokenType::Semicolon {
                return;
            }

            match self.peek().type_ {
                TokenType::Class => return,
                TokenType::Fun => return,
                TokenType::Var => return,
                TokenType::For => return,
                TokenType::If => return,
                TokenType::While => return,
                TokenType::Print => return,
                TokenType::Return => return,
                _ => {}
            }

            self.advance();
        }
    }
}

fn result_map2<T, S, O, E, F: FnOnce(T, S) -> O>(
    a: Result<T, E>,
    b: Result<S, E>,
    op: F,
) -> Result<O, E> {
    a.and_then(|a| b.map(|b| op(a, b)))
}

fn result_map3<T, U, V, O, E, F: FnOnce(T, U, V) -> O>(
    a: Result<T, E>,
    b: Result<U, E>,
    c: Result<V, E>,
    op: F,
) -> Result<O, E> {
    a.and_then(|a| b.and_then(|b| c.map(|c| op(a, b, c))))
}

/*
fn result_map4<T, U, V, W, O, E, F: FnOnce(T, U, V, W) -> O>(
    a: Result<T, E>,
    b: Result<U, E>,
    c: Result<V, E>,
    d: Result<W, E>,
    op: F,
) -> Result<O, E> {
    a.and_then(|a| b.and_then(|b| c.and_then(|c| d.map(|d| op(a, b, c, d)))))
}
*/
