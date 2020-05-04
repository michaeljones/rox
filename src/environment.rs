use crate::scanner::Token;
use crate::value::Value;
use std::collections::HashMap;

pub struct Environment {
    values: HashMap<String, Option<Value>>,
}

pub enum Error {
    NameDoesNotExist,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: Option<Value>) {
        self.values.insert(name, value);
    }

    pub fn assign(&mut self, name: &Token, value: &Value) -> bool {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.clone(), Some(value.clone()));
            true
        } else {
            false
        }
    }

    pub fn get(&self, name: &Token) -> Result<Option<Value>, Error> {
        let option = self.values.get(&name.lexeme);

        match option {
            Some(inner) => Ok(inner.clone()),
            None => Err(Error::NameDoesNotExist),
        }
    }
}
