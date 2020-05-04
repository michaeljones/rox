use crate::scanner::Token;
use crate::value::Value;
use std::collections::HashMap;

pub struct Environment<'a, 'b> {
    enclosing: Option<&'a mut Environment<'a, 'b>>,
    values: HashMap<String, Option<Value>>,
}

pub enum Error {
    NameDoesNotExist,
}

impl<'a, 'b> Environment<'a, 'b> {
    pub fn new() -> Environment<'a, 'b> {
        Environment {
            enclosing: None,
            values: HashMap::new(),
        }
    }

    pub fn enclosing<'c, 'd>(enclosed: &'c mut Environment<'c, 'd>) -> Environment<'c, 'd> {
        Environment {
            enclosing: Some(enclosed),
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: Option<Value>) {
        self.values.insert(name, value);
    }

    pub fn assign(&mut self, name: &Token, value: Value) -> bool {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.clone(), Some(value));
            true
        } else {
            match &mut self.enclosing {
                Some(environment) => environment.assign(name, value),
                None => false,
            }
        }
    }

    pub fn get(&self, name: &Token) -> Result<Option<Value>, Error> {
        let option = self.values.get(&name.lexeme);

        match option {
            Some(inner) => Ok(inner.clone()),
            None => match &self.enclosing {
                Some(environment) => environment.get(name),
                None => Err(Error::NameDoesNotExist),
            },
        }
    }
}
