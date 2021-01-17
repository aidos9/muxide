use paste::paste;

macro_rules! impl_token {
    ($($name:ident: $method_name:ident),*) => {
        #[derive(PartialEq, Clone, Debug)]
        pub enum Token {
            $($name {
                lexeme: String,
                row: usize,
                col: usize,
                file: Option<String>,
            },)*
        }

        paste! {
            impl Token {
                $(
                    pub fn [<new_$method_name>](lexeme: String, row: usize, col: usize, file: Option<String>) -> Self {
                        return Self::$name {
                            lexeme,
                            row,
                            col,
                            file,
                        };
                    }

                    pub fn [<is_$method_name>](&self) -> bool {
                        return std::mem::discriminant(&Token::$name { lexeme: String::new(),
                            row: 0,
                            col: 0,
                            file: None,
                        }) == std::mem::discriminant(self);
                    }
                )*
            }
        }
    };
}

impl_token!(
    EnterInputToken: enter_input,
    StopInputToken: stop_input,
    ToggleInputToken: toggle_input,
    ArrowLeftToken: arrow_left,
    ArrowRightToken: arrow_right,
    ArrowUpToken: arrow_up,
    ArrowDownToken: arrow_down,
    ClosePanelToken: close_panel,
    SwapPanelsToken: swap_panels,
    FocusPanelToken: focus_panel,
    IdentifyPanelsToken: identify_panels,
    MapToken: map,
    UnMapKey: unmap,
    IdentifierToken: identifier,
    ChangeLayoutToken: change_layout,
    OpenCurlyBraceToken: open_curly_brace,
    CloseCurlyBraceToken: close_curly_brace,
    QuitToken: quit
);

impl Token {
    pub fn from_lexeme(lexeme: String, row: usize, col: usize, file: Option<String>) -> Self {
        match lexeme.to_lowercase().as_str() {
            "enterinput" => return Self::new_enter_input(lexeme, row, col, file),
            "stopinput" => return Self::new_stop_input(lexeme, row, col, file),
            "toggleinput" => return Self::new_toggle_input(lexeme, row, col, file),
            "arrowleft" => return Self::new_arrow_left(lexeme, row, col, file),
            "arrowright" => return Self::new_arrow_right(lexeme, row, col, file),
            "arrowup" => return Self::new_arrow_up(lexeme, row, col, file),
            "arrowdown" => return Self::new_arrow_down(lexeme, row, col, file),
            "closepanel" => return Self::new_close_panel(lexeme, row, col, file),
            "swappanels" => return Self::new_swap_panels(lexeme, row, col, file),
            "focuspanel" => return Self::new_focus_panel(lexeme, row, col, file),
            "identify" => return Self::new_identify_panels(lexeme, row, col, file),
            "map" => return Self::new_map(lexeme, row, col, file),
            "unmap" => return Self::new_unmap(lexeme, row, col, file),
            "layout" => return Self::new_change_layout(lexeme, row, col, file),
            "quit" => return Self::new_quit(lexeme, row, col, file),
            "{" => return Self::new_open_curly_brace(lexeme, row, col, file),
            "}" => return Self::new_close_curly_brace(lexeme, row, col, file),
            _ => return Self::new_identifier(lexeme, row, col, file),
        }
    }
}
