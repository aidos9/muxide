use super::environment::Environment;
use super::token::Token;
use crate::config::Command;
use crate::error::{Error, ErrorType};

pub fn process_tokens(tokens: Vec<Token>, env: &mut Environment) -> Result<Vec<Command>, Error> {
    return Processor::new(tokens, env).run();
}

struct Processor<'a> {
    tokens: Vec<Token>,
    environment: &'a mut Environment,
    current_index: usize,
    commands: Vec<Command>,
}

impl<'a> Processor<'a> {
    pub fn new(tokens: Vec<Token>, environment: &'a mut Environment) -> Self {
        return Self {
            tokens,
            environment,
            current_index: 0,
            commands: Vec::new(),
        };
    }

    pub fn run(mut self) -> Result<Vec<Command>, Error> {
        while let Some(current_token) = self.current_token() {
            if current_token.is_identifier() {
                self.consume_method_call()?;
            } else {
                return Err(ErrorType::ScriptError {
                    description: format!("Unexpected identifier: {}", current_token.get_lexeme()),
                }
                .into_error());
            }
        }

        return Ok(self.commands);
    }

    fn consume_method_call(&mut self) -> Result<(), Error> {
        let method_token = self.consume_current().ok_or(
            ErrorType::ScriptError {
                description: "Unexpected lack of identifier for method call".to_string(),
            }
            .into_error(),
        )?;

        let method_name = method_token.get_lexeme();

        if !self.environment.method_declared(&method_name) {
            return Err(Self::token_into_error(
                &method_token,
                "There is no method declared with that name.",
            ));
        } else {
            let open_brace = self.consume_current();

            if open_brace.is_none() || !open_brace.as_ref().unwrap().is_open_round_brace() {
                if open_brace.is_some() {
                    return Err(Self::token_into_error(
                        &open_brace.unwrap(),
                        &format!("Expected '(' after {}", method_name),
                    ));
                } else {
                    return Err(ErrorType::ScriptError {
                        description: format!("Expected '(' after {}", method_name),
                    }
                    .into_error());
                }
            }

            let close_brace = self.consume_current();

            if close_brace.is_none() || !close_brace.as_ref().unwrap().is_close_round_brace() {
                if close_brace.is_some() {
                    return Err(Self::token_into_error(
                        &close_brace.unwrap(),
                        "Expected ')' after '('",
                    ));
                } else {
                    return Err(ErrorType::ScriptError {
                        description: String::from("Expected ')' after '('"),
                    }
                    .into_error());
                }
            }

            if let Some(method_commands) = self.environment.retrieve_method(&method_name) {
                self.commands.append(&mut method_commands.clone());
            } else {
                return Err(Self::token_into_error(
                    &method_token,
                    "There is no method declared with that name.",
                ));
            }
        }

        return Ok(());
    }

    fn consume_current(&mut self) -> Option<Token> {
        let tok = self.current_token().map(|tok| tok.clone())?;

        self.current_index += 1;

        return Some(tok);
    }

    fn current_token(&self) -> Option<&Token> {
        if self.current_index >= self.tokens.len() {
            return None;
        } else {
            return Some(&self.tokens[self.current_index]);
        }
    }

    fn peak_token(&self) -> Option<&Token> {
        if self.current_index + 1 >= self.tokens.len() {
            return None;
        } else {
            return Some(&self.tokens[self.current_index + 1]);
        }
    }

    fn token_into_error(token: &Token, description: &str) -> Error {
        match token.get_file() {
            Some(file) => {
                return ErrorType::ScriptError {
                    description: format!(
                        "{} ({}:{}) \"{}\" -> {}",
                        file,
                        token.get_row(),
                        token.get_col(),
                        token.get_lexeme(),
                        description
                    ),
                }
                .into_error();
            }
            None => {
                return ErrorType::ScriptError {
                    description: format!(
                        "{}:{} \"{}\" -> {}",
                        token.get_row(),
                        token.get_col(),
                        token.get_lexeme(),
                        description
                    ),
                }
                .into_error();
            }
        }
    }
}
