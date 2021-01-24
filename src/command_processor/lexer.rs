use super::token::Token;
use crate::error::Error;

pub fn tokenize_string(input: String, file_name: Option<String>) -> Result<Vec<Token>, Error> {
    return Lexer::new(input, file_name).execute();
}

struct Lexer {
    tokens: Vec<Token>,
    buffer: Vec<char>,
    index: usize,
    row: usize,
    col: usize,
    file: Option<String>,
}

impl Lexer {
    pub fn new(input: String, file: Option<String>) -> Self {
        return Self {
            tokens: Vec::new(),
            buffer: input.chars().collect(),
            index: 0,
            row: 1,
            col: 1,
            file,
        };
    }

    pub fn execute(mut self) -> Result<Vec<Token>, Error> {
        while self.index < self.buffer.len() {
            let current_char = self.current_char();

            if current_char.is_alphanumeric() {
                self.identifier();
            } else if current_char == '\n' {
                self.row += 1;
                self.col = 1;
                self.index += 1;
            } else if current_char.is_whitespace() {
                self.increment();
            } else if current_char == '{'
                || current_char == '}'
                || current_char == '('
                || current_char == ')'
                || current_char == ','
            {
                self.tokens.push(Token::from_lexeme(
                    current_char.to_string(),
                    self.row,
                    self.col,
                    self.file.clone(),
                ));
                self.increment();
            }
        }

        return Ok(self.tokens);
    }

    fn identifier(&mut self) {
        let mut ident = String::new();

        loop {
            if self.index >= self.buffer.len() {
                break;
            }

            let ch = self.current_char();
            if !ch.is_alphanumeric() {
                break;
            }

            ident.push(ch);

            self.increment();
        }

        if ident.len() > 0 {
            self.tokens.push(Token::from_lexeme(
                ident,
                self.row,
                self.col,
                self.file.clone(),
            ));
        }
    }

    #[inline]
    fn current_char(&self) -> char {
        return self.buffer[self.index];
    }

    #[inline]
    fn increment(&mut self) {
        self.col += 1;
        self.index += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use paste::paste;

    macro_rules! test_token_variants {
        ($($input:expr => $method:ident),*) => {
            paste! {
                $(
                    #[test]
                    fn [<test_$method>]() {
                        let input = $input.to_string();
                        let tokens = tokenize_string(input, None).unwrap();
                        assert_eq!(tokens.len(), 1);
                        assert!(tokens[0].[<is_$method>]());
                    }
                )*
            }

        };
    }

    test_token_variants!(
        "enterinput" => enter_input,
        "stopinput" => stop_input,
        "toggleinput" => toggle_input,
        "arrowleft" => arrow_left,
        "arrowright" => arrow_right,
        "arrowup" => arrow_up,
        "arrowdown" => arrow_down,
        "closepanel" => close_panel,
        "swappanels" => swap_panels,
        "focuspanel" => focus_panel,
        "identify" => identify_panels,
        "map" => map,
        "unmap" => unmap,
        "layout" => change_layout,
        "quit" => quit,
        "{" => open_curly_brace,
        "}" => close_curly_brace,
        "(" => open_round_brace,
        ")" => close_round_brace,
        "," => comma,
        "string" => string,
        "integer" => integer,
        "boolean" => boolean,
        "bob" => identifier
    );

    #[test]
    fn test_multiple() {
        let input = "enterinput\nstopinput".to_string();
        let tokens = tokenize_string(input, None).unwrap();
        assert_eq!(tokens.len(), 2);
        assert!(tokens[0].is_enter_input());
        assert!(tokens[1].is_stop_input());
    }

    #[test]
    fn test_multiple_2() {
        let input = "ClosePanel(Integer(5))".to_string();
        let tokens = tokenize_string(input, None).unwrap();
        assert_eq!(tokens.len(), 7);
        assert!(tokens[0].is_close_panel());
        assert!(tokens[1].is_open_round_brace());
        assert!(tokens[2].is_integer());
        assert!(tokens[3].is_open_round_brace());
        assert!(tokens[4].is_identifier());
        assert!(tokens[5].is_close_round_brace());
        assert!(tokens[6].is_close_round_brace());
    }
}
