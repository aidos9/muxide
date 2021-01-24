use super::environment::Environment;
use super::token::Token;
use crate::command_processor::Literal;
use crate::config::Command;
use crate::error::{Error, ErrorType};
use std::convert::{TryFrom, TryInto};
use termion::event::Key;

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
            } else if current_token.is_method() {
                self.consume_method_declaration()?;
            } else {
                let stmt = self.consume_statement()?;
                self.commands.push(stmt);

                // return Err(ErrorType::ScriptError {
                //     description: format!("Unexpected identifier: {}", current_token.get_lexeme()),
                // }
                // .into_error());
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

    fn consume_method_declaration(&mut self) -> Result<(), Error> {
        let method_token = self.consume_current().ok_or(
            ErrorType::ScriptError {
                description: "Unexpected lack of 'method' keyword for method declaration"
                    .to_string(),
            }
            .into_error(),
        )?;

        let name = self.consume_current().ok_or(
            ErrorType::ScriptError {
                description: "Expected method name after method keyword.".to_string(),
            }
            .into_error(),
        )?;

        let opening_brace = self.consume_current().ok_or(
            ErrorType::ScriptError {
                description: "Expected opening brace: '{' after method name".to_string(),
            }
            .into_error(),
        )?;

        if !opening_brace.is_open_curly_brace() {
            return Err(Self::token_into_error(
                &opening_brace,
                "Expected  '{' after method name",
            ));
        }

        let body = self.consume_statements(true)?;

        self.environment.declare_method(name.get_lexeme(), body);

        return Ok(());
    }

    /// Consumes statements until end of tokens or if it is a block a closing brace is found.
    fn consume_statements(&mut self, block: bool) -> Result<Vec<Command>, Error> {
        let mut commands = Vec::new();

        loop {
            if self.current_token().is_none()
                || (block && self.current_token().unwrap().is_close_curly_brace())
            {
                break;
            }

            commands.push(self.consume_statement()?);
        }

        return Ok(commands);
    }

    fn consume_statement(&mut self) -> Result<Command, Error> {
        let current_token = self.consume_current();

        if current_token.is_none() {
            return Err(ErrorType::ScriptError {
                description: "Expected a statement identifier".to_string(),
            }
            .into_error());
        }

        let current_token = current_token.unwrap();

        return Ok({
            if current_token.is_enter_input() {
                Command::EnterInputCommand
            } else if current_token.is_stop_input() {
                Command::StopInputCommand
            } else if current_token.is_toggle_input() {
                Command::ToggleInputCommand
            } else if current_token.is_arrow_left() {
                Command::ArrowLeftCommand
            } else if current_token.is_arrow_right() {
                Command::ArrowRightCommand
            } else if current_token.is_arrow_up() {
                Command::ArrowUpCommand
            } else if current_token.is_arrow_down() {
                Command::ArrowDownCommand
            } else if current_token.is_open_panel() {
                Command::OpenPanelCommand
            } else if current_token.is_identify_panels() {
                Command::IdentifyPanelsCommand
            } else if current_token.is_close_panel() {
                match self.consume_current() {
                    Some(tok) => {
                        if !tok.is_open_round_brace() {
                            return Err(Self::token_into_error(
                                &tok,
                                &format!("Expected '(' after {}", current_token.get_lexeme()),
                            ));
                        }
                    }
                    None => {
                        return Err(ErrorType::ScriptError {
                            description: format!(
                                "Expected '(' after {}",
                                current_token.get_lexeme()
                            ),
                        }
                        .into_error());
                    }
                }

                let literal = self.consume_literal()?;

                if !literal.is_integer() {
                    return Err(ErrorType::ScriptError {
                        description: String::from("Expected an integer panel ID."),
                    }
                    .into_error());
                }

                match self.consume_current() {
                    Some(tok) => {
                        if !tok.is_close_round_brace() {
                            return Err(Self::token_into_error(
                                &tok,
                                "Expected ')' after panel ID.",
                            ));
                        }
                    }
                    None => {
                        return Err(ErrorType::ScriptError {
                            description: String::from("Expected ')' after panel ID."),
                        }
                        .into_error());
                    }
                }

                let mut integer_value = literal.integer_value().unwrap();

                let integer_value = {
                    if integer_value < 0 {
                        0
                    } else {
                        usize::try_from(integer_value).unwrap_or(usize::max_value())
                    }
                };

                Command::ClosePanelCommand(integer_value)
            } else if current_token.is_map() {
                match self.consume_current() {
                    Some(tok) => {
                        if !tok.is_open_round_brace() {
                            return Err(Self::token_into_error(
                                &tok,
                                &format!("Expected '(' after {}", current_token.get_lexeme()),
                            ));
                        }
                    }
                    None => {
                        return Err(ErrorType::ScriptError {
                            description: format!(
                                "Expected '(' after {}",
                                current_token.get_lexeme()
                            ),
                        }
                        .into_error());
                    }
                }

                let key = self.consume_key(&current_token)?;

                match self.consume_current() {
                    Some(tok) => {
                        if !tok.is_comma() {
                            return Err(Self::token_into_error(&tok, "Expected ',' after key."));
                        }
                    }
                    None => {
                        return Err(ErrorType::ScriptError {
                            description: String::from("Expected ',' after key."),
                        }
                        .into_error());
                    }
                }

                let command = self.consume_literal()?;

                if !command.is_string() {
                    return Err(Self::token_into_error(
                        &current_token,
                        "Expected a string literal command.",
                    ));
                }

                let command = command.string_value();

                match self.consume_current() {
                    Some(tok) => {
                        if !tok.is_close_round_brace() {
                            return Err(Self::token_into_error(
                                &tok,
                                "Expected ')' after command.",
                            ));
                        }
                    }
                    None => {
                        return Err(ErrorType::ScriptError {
                            description: String::from("Expected ')' after command."),
                        }
                        .into_error());
                    }
                }

                panic!();
            } else {
                unimplemented!();
            }
        });
    }

    fn consume_literal(&mut self) -> Result<Literal, Error> {
        let current_token = self.consume_current().ok_or(
            ErrorType::ScriptError {
                description: "Expected a literal identifier (String, Integer, Boolean)".to_string(),
            }
            .into_error(),
        )?;

        if !current_token.is_integer() && !current_token.is_string() && !current_token.is_boolean()
        {
            return Err(Self::token_into_error(
                &current_token,
                "Expected a literal identifier (String, Integer, Boolean)",
            ));
        }

        match self.consume_current() {
            Some(tok) => {
                if !tok.is_open_round_brace() {
                    return Err(Self::token_into_error(
                        &tok,
                        &format!("Expected '(' after {}", current_token.get_lexeme()),
                    ));
                }
            }
            None => {
                return Err(ErrorType::ScriptError {
                    description: format!("Expected '(' after {}", current_token.get_lexeme()),
                }
                .into_error());
            }
        }

        let content_token = self.consume_current().ok_or(
            ErrorType::ScriptError {
                description: String::from("Expected an identifier for a literal body"),
            }
            .into_error(),
        )?;

        let content = content_token.get_lexeme();

        match self.consume_current() {
            Some(tok) => {
                if !tok.is_close_round_brace() {
                    return Err(Self::token_into_error(
                        &tok,
                        &format!("Expected ')' after {}", content),
                    ));
                }
            }
            None => {
                return Err(ErrorType::ScriptError {
                    description: format!("Expected ')' after {}", content),
                }
                .into_error());
            }
        }

        if current_token.is_integer() {
            return Literal::integer_from_string(content)
                .ok_or(Self::token_into_error(&content_token, "Invalid integer"));
        } else if current_token.is_boolean() {
            return Literal::bool_from_string(content)
                .ok_or(Self::token_into_error(&content_token, "Invalid boolean"));
        } else {
            return Ok(Literal::from(content));
        }
    }

    fn consume_key(&mut self, reference_token: &Token) -> Result<Key, Error> {
        let string_literal = self.consume_literal()?;

        if !string_literal.is_string() {
            return Err(Self::token_into_error(
                reference_token,
                "Expected a string literal.",
            ));
        }

        let string_literal: String = string_literal.string_value();
        return Self::string_into_key(string_literal)
            .map_err(|e| Self::token_into_error(reference_token, e));
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

    fn peek_token(&self) -> Option<&Token> {
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

    fn string_into_key(mut string: String) -> Result<Key, &'static str> {
        let mut first_half = String::new();
        let mut string: Vec<char> = string.chars().collect();

        while string.len() > 0 {
            if string[0] == '+' {
                if first_half.len() == 0 {
                    return Err("A single character is required to follow a '+'");
                }

                string.remove(0);
                break;
            } else {
                first_half.push(string.remove(0));
            }
        }

        if string.len() > 0 {
            let lowered = first_half.to_lowercase();

            if lowered == "ctrl" {
                if string.len() != 1 {
                    return Err("Expected a single character to follow '+'.");
                } else {
                    return Ok(Key::Ctrl(string[0]));
                }
            } else if lowered == "alt" {
                if string.len() != 1 {
                    return Err("Expected a single character to follow '+'.");
                } else {
                    return Ok(Key::Alt(string[0]));
                }
            } else {
                return Err("Only the \"Alt\" and \"Ctrl\" modifiers are supported.");
            }
        } else {
            if first_half.len() != 1 {
                return Err("A single character key or modifier '+' single character is expected.");
            } else {
                return Ok(Key::Char(first_half.remove(0)));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::lexer::tokenize_string;
    use super::*;
    use paste::paste;
    use termion::event::Key;

    macro_rules! test_commands {
        ($({$input:expr, $command_name:expr, $name:ident}),*) => {
            paste! {
                $(
                    #[test]
                    fn [<test_$name>]() {
                        let tokens = tokenize_string($input, None).unwrap();
                        let mut env = Environment::new();
                        assert_eq!(
                            process_tokens(tokens, &mut env).unwrap(),
                            vec![$command_name]
                        );
                    }
                )*
            }
        };
    }

    test_commands!(
        {
            "OpenPanel".to_string(),
            Command::OpenPanelCommand,
            open_panel
        },
        {
            "SwapPanels(Integer(1), Integer(2))".to_string(),
            Command::SwapPanelsCommand(1,2),
            swap_panels
        },
        {
            "ClosePanel(Integer(5))".to_string(),
            Command::ClosePanelCommand(5),
            close_panel
        },
        {
            "FocusPanel(Integer(1))".to_string(),
            Command::FocusPanelCommand(1),
            focus_panel
        },
        {
            "Identify".to_string(),
            Command::IdentifyPanelsCommand,
            identify
        },
        {
            "Map(String(Ctrl+N), String(OpenPanel))".to_string(),
            Command::MapCommand(Key::Ctrl('N'), Box::new(Command::OpenPanelCommand)),
            map
        },
        {
            "UnMap(String(Ctrl+N))".to_string(),
            Command::UnMapCommand(Key::Ctrl('N')),
            unmap
        },
        {
            "EnterInput".to_string(),
            Command::EnterInputCommand,
            enter_input
        },
        {
            "StopInput".to_string(),
            Command::StopInputCommand,
            stop_input
        },
        {
            "ToggleInput".to_string(),
            Command::ToggleInputCommand,
            toggle_input
        },
        {
            "ArrowLeft".to_string(),
            Command::ArrowLeftCommand,
            arrow_left
        },
        {
            "ArrowRight".to_string(),
            Command::ArrowRightCommand,
            arrow_right
        },
        {
            "ArrowUp".to_string(),
            Command::ArrowUpCommand,
            arrow_up
        },
        {
            "ArrowDown".to_string(),
            Command::ArrowDownCommand,
            arrow_down
        }
    );

    #[test]
    fn test_method_call_fail() {
        let input = "method()".to_string();
        let tokens = tokenize_string(input, None).unwrap();
        let mut env = Environment::new();
        process_tokens(tokens, &mut env).unwrap_err();
    }
}
