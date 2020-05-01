use crate::parser::Expr;
use crate::parser::Stmt;
use crate::scanner::Token;
use crate::scanner::TokenType;
use crate::value::Value;

#[derive(PartialEq, Debug)]
pub enum EvaluationError {
    InvalidUnaryOperand(Token, String),
    InvalidBinaryOperand(Token, String),
}

pub fn interpret(statements: &Vec<Stmt>) {
    for statement in statements {
        evaluate_statement(statement);
    }
}

fn evaluate_statement(statement: &Stmt) {
    match statement {
        Stmt::Print(expr) => {
            let result = evaluate_expression(expr);
            match result {
                Ok(value) => println!("{}", value.to_string()),
                Err(err) => println!("{:?}", err),
            }
        }
        Stmt::Expression(expr) => {
            let result = evaluate_expression(expr);
            match result {
                Ok(_) => {}
                Err(err) => println!("{:?}", err),
            }
        }
    }
}

fn evaluate_expression(expr: &Expr) -> Result<Value, EvaluationError> {
    match expr {
        Expr::Literal(value) => Ok(value.clone()),
        Expr::Grouping(expr) => evaluate_expression(expr),
        Expr::Unary(operator, expr) => evaluate_unary(operator, expr),
        Expr::Binary(left, operator, right) => evaluate_binary(left, operator, right),
    }
}

fn evaluate_unary(operator: &Token, expr: &Expr) -> Result<Value, EvaluationError> {
    let value = evaluate_expression(expr);

    match (&operator.type_, value) {
        (&TokenType::Minus, Ok(Value::Double(double))) => Ok(Value::Double(-double)),
        (&TokenType::Minus, Ok(_)) => Err(EvaluationError::InvalidUnaryOperand(
            operator.clone(),
            "Operand must be a number".to_string(),
        )),
        (&TokenType::Bang, Ok(value)) => Ok(Value::Bool(!is_truthy(&value))),
        (_, err @ Err(_)) => err,
        _ => Err(EvaluationError::InvalidUnaryOperand(
            operator.clone(),
            "Unrecognised unary operation".to_string(),
        )),
    }
}

fn evaluate_binary(left: &Expr, operator: &Token, right: &Expr) -> Result<Value, EvaluationError> {
    let left = evaluate_expression(left);
    let right = evaluate_expression(right);

    match (left, &operator.type_, right) {
        (Ok(Value::Double(left)), &TokenType::Minus, Ok(Value::Double(right))) => {
            Ok(Value::Double(left - right))
        }
        (Ok(_), &TokenType::Minus, Ok(_)) => Err(EvaluationError::InvalidBinaryOperand(
            operator.clone(),
            "Operands must be numbers".to_string(),
        )),
        (Ok(Value::Double(left)), &TokenType::Slash, Ok(Value::Double(right))) => {
            Ok(Value::Double(left / right))
        }
        (Ok(_), &TokenType::Slash, Ok(_)) => Err(EvaluationError::InvalidBinaryOperand(
            operator.clone(),
            "Operands must be numbers".to_string(),
        )),
        (Ok(Value::Double(left)), &TokenType::Star, Ok(Value::Double(right))) => {
            Ok(Value::Double(left * right))
        }
        (Ok(_), &TokenType::Star, Ok(_)) => Err(EvaluationError::InvalidBinaryOperand(
            operator.clone(),
            "Operands must be numbers".to_string(),
        )),
        (Ok(Value::Double(left)), &TokenType::Plus, Ok(Value::Double(right))) => {
            Ok(Value::Double(left + right))
        }
        (Ok(Value::String(left)), &TokenType::Plus, Ok(Value::String(right))) => {
            Ok(Value::String(format!("{}{}", left, right)))
        }
        (Ok(_), &TokenType::Plus, Ok(_)) => Err(EvaluationError::InvalidBinaryOperand(
            operator.clone(),
            "Operands must be two numbers or two strings".to_string(),
        )),
        // Greater
        (Ok(Value::Double(left)), &TokenType::Greater, Ok(Value::Double(right))) => {
            Ok(Value::Bool(left > right))
        }
        (Ok(_), &TokenType::Greater, Ok(_)) => Err(EvaluationError::InvalidBinaryOperand(
            operator.clone(),
            "Operands must be numbers".to_string(),
        )),
        // Greater Equal
        (Ok(Value::Double(left)), &TokenType::GreaterEqual, Ok(Value::Double(right))) => {
            Ok(Value::Bool(left >= right))
        }
        (Ok(_), &TokenType::GreaterEqual, Ok(_)) => Err(EvaluationError::InvalidBinaryOperand(
            operator.clone(),
            "Operands must be numbers".to_string(),
        )),
        // Less
        (Ok(Value::Double(left)), &TokenType::Less, Ok(Value::Double(right))) => {
            Ok(Value::Bool(left > right))
        }
        (Ok(_), &TokenType::Less, Ok(_)) => Err(EvaluationError::InvalidBinaryOperand(
            operator.clone(),
            "Operands must be numbers".to_string(),
        )),
        // Less Equal
        (Ok(Value::Double(left)), &TokenType::LessEqual, Ok(Value::Double(right))) => {
            Ok(Value::Bool(left >= right))
        }
        (Ok(_), &TokenType::LessEqual, Ok(_)) => Err(EvaluationError::InvalidBinaryOperand(
            operator.clone(),
            "Operands must be numbers".to_string(),
        )),
        (Ok(left), &TokenType::BangEqual, Ok(right)) => Ok(Value::Bool(left != right)),
        (Ok(left), &TokenType::EqualEqual, Ok(right)) => Ok(Value::Bool(left == right)),
        _ => Err(EvaluationError::InvalidBinaryOperand(
            operator.clone(),
            "Unrecognised binary operation".to_string(),
        )),
    }
}

fn is_truthy(value: &Value) -> bool {
    match value {
        Value::String(_) => true,
        Value::Double(_) => true,
        Value::Bool(boolean) => *boolean,
        Value::Nil => false,
    }
}
