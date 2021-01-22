use super::token::Token;
use crate::error::Error;

pub fn tokenize_string(input: String, file_name: Option<String>) -> Result<Vec<Token>, Error> {
    return Tokenizer::new(input, file_name).execute();
}

struct Tokenizer {
    tokens: Vec<Token>,
    buffer: Vec<char>,
    row: usize,
    col: usize,
    file: Option<String>,
}

impl Tokenizer {
    pub fn new(input: String, file: Option<String>) -> Self {
        return Self {
            tokens: Vec::new(),
            buffer: input.chars().collect(),
            row: 1,
            col: 1,
            file,
        };
    }

    pub fn execute(mut self) -> Result<Vec<Token>, Error> {
        let mut current_token = Vec::new();

        while self.buffer.len() > 0 {
            let ch = self.buffer.remove(0);

            if ch == '\n' {
                self.tokens.push(Token::from_lexeme(
                    current_token.iter().collect(),
                    self.row,
                    self.col,
                    self.file.clone(),
                ));

                current_token = Vec::new();
                self.row += 1;
                self.col = 1;
            } else if ch.is_whitespace() {
                if current_token.len() > 0 {
                    self.tokens.push(Token::from_lexeme(
                        current_token.iter().collect(),
                        self.row,
                        self.col,
                        self.file.clone(),
                    ));

                    current_token = Vec::new();
                }

                self.col += 1;
            } else if ch.is_alphanumeric()
                || ch == '{'
                || ch == '}'
                || ch == '('
                || ch == ')'
                || ch == ','
            {
                current_token.push(ch);
                self.col += 1;
            } else {
                self.col += 1;
            }
        }

        if current_token.len() > 0 {
            self.tokens.push(Token::from_lexeme(
                current_token.iter().collect(),
                self.row,
                self.col,
                self.file.clone(),
            ));
        }

        return Ok(self.tokens);
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
}
