

enum Expr {
    Binary(Box<Expr>, Token, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(TokenValue),
    Unary(Token, Box<Expr>),
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
                TokenValue::String(string) => string.clone(),
                TokenValue::Double(double) => double.to_string(),
            },
            Expr::Unary(operator, inner_expr) => format!(
                "({} {})",
                operator.type_.to_string(),
                inner_expr.to_string()
            ),
        }
    }
}
