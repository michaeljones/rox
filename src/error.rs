pub fn error(line: usize, message: String) {
    report(line, String::new(), message);
}

fn report(line: usize, where_: String, message: String) {
    println!("[line {}] Error {}: {}", line, where_, message);

    // hadError = true;
}
