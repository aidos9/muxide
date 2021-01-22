use super::literal::Literal;
use super::token::Token;
use crate::config::Command;
use std::collections::HashMap;

#[derive(PartialEq, Clone, Debug)]
pub struct Environment {
    methods: HashMap<String, Vec<Command>>,
    variables: HashMap<String, Literal>,
}

impl Environment {
    pub fn new() -> Self {
        return Self {
            methods: HashMap::new(),
            variables: HashMap::new(),
        };
    }

    pub fn method_declared(&self, name: &str) -> bool {
        return self.methods.contains_key(name);
    }

    pub fn declare_method(&mut self, name: String, body: Vec<Command>) {
        self.methods.insert(name, body);
    }

    pub fn retrieve_method(&self, name: &str) -> Option<&Vec<Command>> {
        return self.methods.get(name);
    }

    pub fn variable_declared(&self, name: &str) -> bool {
        return self.variables.contains_key(name);
    }

    pub fn declare_variable(&mut self, name: String, value: Literal) {
        self.variables.insert(name, value);
    }

    pub fn retrieve_variable(&self, name: &str) -> Option<&Literal> {
        return self.variables.get(name);
    }
}
