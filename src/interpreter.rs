use crate::environment::Environment;
use crate::parser::Expr;
use crate::parser::Stmt;
use crate::scanner::Token;
use crate::scanner::TokenType;
use crate::value::Value;

#[derive(PartialEq, Debug, Clone)]
pub enum EvaluationError {
    InvalidUnaryOperand(Token, String),
    InvalidBinaryOperand(Token, String),
    VariableDoesNotExist,
    InvalidAssignment,
}

pub struct Interpreter {}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {}
    }

    pub fn interpret(&mut self, statements: &Vec<Stmt>) {
        let mut environments = vec![Environment::new()];
        for statement in statements {
            self.execute_statement(statement, &mut environments);
        }
    }

    fn execute_statement(&mut self, statement: &Stmt, environments: &mut Vec<Environment>) {
        match statement {
            Stmt::Block(statements) => {
                self.execute_block(statements, environments);
            }
            Stmt::Print(expr) => {
                let result = self.evaluate_expression(expr, environments);
                match result {
                    Ok(value) => println!("{}", value.to_string()),
                    Err(err) => println!("{:?}", err),
                }
            }
            Stmt::Expression(expr) => {
                let result = self.evaluate_expression(expr, environments);
                match result {
                    Ok(_) => {}
                    Err(err) => println!("{:?}", err),
                }
            }
            Stmt::Var(name, initialiser) => {
                let value = initialiser
                    .as_ref()
                    .map(|expr| self.evaluate_expression(expr, environments))
                    .transpose();
                match value {
                    Ok(value) => Interpreter::define(environments, name.lexeme.clone(), value),
                    Err(err) => println!("{:?}", err),
                }
            }
        }
    }

    fn execute_block(&mut self, statements: &Vec<Stmt>, environments: &mut Vec<Environment>) {
        environments.push(Environment::new());
        for statement in statements {
            self.execute_statement(statement, environments);
        }
        environments.pop();
    }

    fn evaluate_expression(
        &mut self,
        expr: &Expr,
        environments: &mut Vec<Environment>,
    ) -> Result<Value, EvaluationError> {
        match expr {
            Expr::Literal(value) => Ok(value.clone()),
            Expr::Grouping(expr) => self.evaluate_expression(expr, environments),
            Expr::Unary(operator, expr) => self.evaluate_unary(operator, expr, environments),
            Expr::Binary(left, operator, right) => {
                self.evaluate_binary(left, operator, right, environments)
            }
            Expr::Variable(name_token) => Interpreter::get(environments, name_token)
                .map(|value_option| value_option.unwrap_or(Value::Nil))
                .map_err(|_| EvaluationError::VariableDoesNotExist),
            Expr::Assign(name_token, expr) => {
                let result = self.evaluate_expression(expr, environments);
                result.and_then(|value| {
                    if Interpreter::assign(environments, name_token, &value) {
                        Ok(value)
                    } else {
                        Err(EvaluationError::InvalidAssignment)
                    }
                })
            }
        }
    }

    fn evaluate_unary(
        &mut self,
        operator: &Token,
        expr: &Expr,
        environments: &mut Vec<Environment>,
    ) -> Result<Value, EvaluationError> {
        let value = self.evaluate_expression(expr, environments);

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

    fn evaluate_binary(
        &mut self,
        left: &Expr,
        operator: &Token,
        right: &Expr,
        environments: &mut Vec<Environment>,
    ) -> Result<Value, EvaluationError> {
        let left = self.evaluate_expression(left, environments);
        let right = self.evaluate_expression(right, environments);

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

    fn define(environments: &mut Vec<Environment>, name: String, value: Option<Value>) {
        if let Some(last) = environments.last_mut() {
            last.define(name, value);
        }
    }

    fn assign(environments: &mut Vec<Environment>, name: &Token, value: &Value) -> bool {
        for environment in environments.iter_mut().rev() {
            if environment.assign(name, value) {
                return true;
            }
        }
        false
    }

    fn get(
        environments: &mut Vec<Environment>,
        name: &Token,
    ) -> Result<Option<Value>, EvaluationError> {
        let mut result = Err(EvaluationError::VariableDoesNotExist);
        for environment in environments.iter().rev() {
            result = environment
                .get(name)
                .map_err(|_| EvaluationError::VariableDoesNotExist);
            if let Ok(_) = result {
                return result;
            }
        }
        result
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
