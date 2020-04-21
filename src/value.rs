#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Double(f64),
    Bool(bool),
    Nil,
}

impl std::string::ToString for Value {
    fn to_string(&self) -> String {
        match self {
            Value::String(string) => format!("\"{}\"", string.clone()),
            Value::Double(double) => double.to_string(),
            Value::Bool(boolean) => boolean.to_string(),
            Value::Nil => "nil".to_string(),
        }
    }
}
