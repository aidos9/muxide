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
}
