use std::cmp::Ordering;
use std::io::Read;

mod error;
mod parser;
mod scanner;

/*
let ast = Expr::Binary(
    Box::new(Expr::Unary(
        Token::new(TokenType::Minus, "-".to_string(), None, 1),
        Box::new(Expr::Literal(TokenValue::Double(123.0))),
    )),
    Token::new(TokenType::Star, "*".to_string(), None, 1),
    Box::new(Expr::Grouping(Box::new(Expr::Literal(TokenValue::Double(
        45.67,
    ))))),
);

println!("{}", ast.to_string())
*/

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let count = args.len();
    match count.cmp(&2) {
        Ordering::Greater => {
            println!("Usage: rox [script]");
            std::process::exit(64);
        }
        Ordering::Equal => {
            print!("{:?}", args);
            run_file(&args[1]);
        }
        Ordering::Less => {
            run_prompt();
        }
    }
}

fn run_file(file: &str) {
    let mut file = std::fs::File::open(file).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    run(contents);

    // if (hadError) std::process::exit(65);
}

fn run_prompt() {
    print!("> ");
    loop {
        let mut input = String::new();
        match std::io::stdin().read_line(&mut input) {
            Ok(_) => run(input),
            Err(_error) => {
                std::process::exit(64);
            }
        }
        // hadError = false;
    }
}

fn run(source: String) {
    let mut scanner = scanner::Scanner::new(source);
    let tokens = scanner.scan_tokens();
    let mut parser = parser::Parser::new(tokens);

    match parser.parse() {
        Ok(expr) => println!("{}", expr.to_string()),
        Err(err) => println!("{:?}", err),
    }
}
