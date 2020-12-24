/// Internal Constant
#[doc(Hidden)]
pub const UPPER_LIMIT: i64 = 9999;

macro_rules! derive_perform {
    ($($name:tt)*) => {
        impl vte::Perform for $($name)* {
            fn print(&mut self, ch: char) {
                self.display_text(ch);
            }

            fn execute(&mut self, byte: u8) {
                match byte {
                    0 => (),
                    7 => self.backspace(),
                    9 => self.horizontal_tab(),
                    10 => self.line_feed(),
                    11 => self.vert_tab(),
                    12 => self.form_feed(),
                    13 => self.carriage_return(),
                    _ => self.unsupported_execute_byte(byte),
                }
            }

            fn hook(&mut self, params: &vte::Params, intermediates: &[u8], _ignore: bool, action: char) {
                match intermediates.get(0) {
                    Some(i) => {
                        self.log_hook(&format!("Unhandled DCS sequence {} {} {}", i, $crate::vte_handler::params_to_string(params), action));
                    },
                    None => {
                        self.log_hook(&format!("Unhandled DCS sequence {} {}", $crate::vte_handler::params_to_string(params), action));
                    }
                }
            }

            fn put(&mut self, _byte: u8) {}
            fn unhook(&mut self) {}

            fn osc_dispatch(&mut self, params: &[&[u8]], bell_terminated: bool) {
                // Reference: https://xtermjs.org/docs/api/vtfeatures/#osc
                if let (Some(code), Some(string)) = (params.get(0), params.get(1)) {
                    if code == &b"0" {
                        self.set_title_icon_name_u8(string);
                    } else if code == &b"1" {
                        self.set_icon_name_u8(string);
                    } else if code == &b"2" {
                        self.set_window_title_u8(string);
                    }

                    self.log_osc(&format!("Unhandled OSC dispatch {{code: {:?}, bell_terminated: {}}}", code, bell_terminated));

                    return;
                }

                self.log_osc(&format!("Unhandled OSC dispatch {{params: {:?}, bell_terminated: {}}}", params, bell_terminated));
            }

            fn csi_dispatch(&mut self, params: &vte::Params, intermediates: &[u8], ignore: bool, action: char) {
                if intermediates.len() == 0 {
                    match action {
                        '@' => self.insert_character(convert_to_u16!($crate::vte_handler::verify_parameters_single(params, 1), $crate::vte_handler::UPPER_LIMIT)),
                        'A' => self.cursor_up(convert_to_u16!($crate::vte_handler::verify_parameters_single(params, 1), $crate::vte_handler::UPPER_LIMIT)),
                        'B' => self.cursor_down(convert_to_u16!($crate::vte_handler::verify_parameters_single(params, 1), $crate::vte_handler::UPPER_LIMIT)),
                        'C' => self.cursor_forward(convert_to_u16!($crate::vte_handler::verify_parameters_single(params, 1), $crate::vte_handler::UPPER_LIMIT)),
                        'D' => self.cursor_backward(convert_to_u16!($crate::vte_handler::verify_parameters_single(params, 1), $crate::vte_handler::UPPER_LIMIT)),
                        'E' => self.cursor_next_line(convert_to_u16!($crate::vte_handler::verify_parameters_single(params, 1), $crate::vte_handler::UPPER_LIMIT)),
                        'F' => self.cursor_previous_line(convert_to_u16!($crate::vte_handler::verify_parameters_single(params, 1), $crate::vte_handler::UPPER_LIMIT)),
                        'G' => self.cursor_horizontal_absolute(convert_to_u16!($crate::vte_handler::verify_parameters_single(params, 1), $crate::vte_handler::UPPER_LIMIT)),
                        'H' => {
                            let (lines, columns) = $crate::vte_handler::verify_parameters_double(params, (1, 1));
                            self.cursor_position(convert_to_u16!(lines, $crate::vte_handler::UPPER_LIMIT), convert_to_u16!(columns, $crate::vte_handler::UPPER_LIMIT));
                        },
                        'I' => self.cursor_horizontal_tab(convert_to_u16!($crate::vte_handler::verify_parameters_single(params, 1), $crate::vte_handler::UPPER_LIMIT)),
                        'J' => self.erase_in_display(convert_to_u16!($crate::vte_handler::verify_parameters_single_range(params, 0, 0, 3), $crate::vte_handler::UPPER_LIMIT)),
                        'K' => self.erase_in_line(convert_to_u16!($crate::vte_handler::verify_parameters_single_range(params, 0, 0, 2), $crate::vte_handler::UPPER_LIMIT)),
                        'L' => self.insert_line(convert_to_u16!($crate::vte_handler::verify_parameters_single(params, 1), $crate::vte_handler::UPPER_LIMIT)),
                        'M' => self.delete_line(convert_to_u16!($crate::vte_handler::verify_parameters_single(params, 1), $crate::vte_handler::UPPER_LIMIT)),
                        'P' => self.delete_character(convert_to_u16!($crate::vte_handler::verify_parameters_single(params, 1), $crate::vte_handler::UPPER_LIMIT)),
                        'S' => self.scroll_up(convert_to_u16!($crate::vte_handler::verify_parameters_single(params, 1), $crate::vte_handler::UPPER_LIMIT)),
                        'T' => self.scroll_down(convert_to_u16!($crate::vte_handler::verify_parameters_single(params, 1), $crate::vte_handler::UPPER_LIMIT)),
                        'X' => self.erase_character(convert_to_u16!($crate::vte_handler::verify_parameters_single(params, 1), $crate::vte_handler::UPPER_LIMIT)),
                        'Z' => self.cursor_backward_tab(convert_to_u16!($crate::vte_handler::verify_parameters_single(params, 1), $crate::vte_handler::UPPER_LIMIT)),
                        '`' => self.cursor_horizontal_absolute(convert_to_u16!($crate::vte_handler::verify_parameters_single(params, 1), $crate::vte_handler::UPPER_LIMIT)), // Horizontal position absolute
                        'a' => self.cursor_forward(convert_to_u16!($crate::vte_handler::verify_parameters_single(params, 1), $crate::vte_handler::UPPER_LIMIT)), // Horizontal position relative
                        'b' => self.repeat_preceding(convert_to_u16!($crate::vte_handler::verify_parameters_single(params, 1), $crate::vte_handler::UPPER_LIMIT)),
                        'd' => self.vertical_position_absolute(convert_to_u16!($crate::vte_handler::verify_parameters_single(params, 1), $crate::vte_handler::UPPER_LIMIT)),
                        'e' => self.vertical_position_relative(convert_to_u16!($crate::vte_handler::verify_parameters_single(params, 1), $crate::vte_handler::UPPER_LIMIT)),
                        'f' => {
                            let (lines, columns) = $crate::vte_handler::verify_parameters_double(params, (1, 1));
                            self.cursor_position(convert_to_u16!(lines, $crate::vte_handler::UPPER_LIMIT), convert_to_u16!(columns, $crate::vte_handler::UPPER_LIMIT));
                        },
                        'h' => self.set_mode(params),
                        'l' => self.reset_mode(params),
                        _ => {
                            self.log_csi(&format!("Unhandled CSI dispatch {{params: {:?}, action: {}, intermediates: {:?}}}", params, action, intermediates));
                        }
                    }
                } else if intermediates[0] == b'?' {
                    match action {
                        'J' => self.selective_erase_in_display(convert_to_u16!($crate::vte_handler::verify_parameters_single_range(params, 0, 0, 3), $crate::vte_handler::UPPER_LIMIT)),
                        'K' => self.selective_erase_in_line(convert_to_u16!($crate::vte_handler::verify_parameters_single_range(params, 0, 0, 2), $crate::vte_handler::UPPER_LIMIT)),
                        'm' => self.select_graphic_rendition(params),
                        'h' => self.dec_private_set_mode(params),
                        'l' => self.dec_private_reset_mode(params),
                        _ => {
                            self.log_csi(&format!("Unhandled CSI dispatch {{params: {:?}, action: {}, intermediates: {:?}}}", params, action, intermediates));
                        }
                    }
                } else if intermediates.len() == 1 {
                   self.log_csi(&format!("Unhandled CSI dispatch {{ {} {}, params: {:?}}}", intermediates[0] as char, action, params));
                } else {
                    self.log_csi(&format!("Unhandled CSI dispatch {{params: {:?}, action: {}, intermediates: {:?}}}", params, action, intermediates));
                }
            }

            fn esc_dispatch(&mut self, intermediates: &[u8], ignore: bool, byte: u8) {
                if intermediates.len() > 0 {
                    self.log_esc(&format!("Unhandled ESC dispatch {{intermediates: {:?}, ignore: {}, byte: {}}}", intermediates, ignore, byte));
                }

                match byte {
                    b'7' => self.save_cursor(),
                    b'8' => self.restore_cursor(),
                    b'D' => self.index(),
                    b'E' => self.next_line(),
                    b'H' => self.horizontal_tabulation_set(),
                    b'M' => self.reverse_index(),
                    b'c' => self.reset_to_initial_state(),
                    b'=' => self.keypad_application_mode(),
                    b'>' => self.keypad_numeric_mode(),
                    _ => self.log_esc(&format!("Unhandled ESC dispatch {{intermediates: {:?}, ignore: {}, byte: {}}}", intermediates, ignore, byte)),
                }
            }
        }
    }
}

macro_rules! prototype_declaration {
    ($name:ident) => {
        fn $name(&mut self);
    };

    ($name:ident, $($var_name:ident : $tp:ty),*) => {
        fn $name(&mut self, $($var_name : $tp),*);
    }
}

macro_rules! convert_to_u16 {
    ($value:expr) => {
        $value as u16
    };

    ($value:expr, $limit:expr) => {
        if $value <= $limit {
            $value as u16
        } else {
            $limit as u16
        }
    };
}

pub trait VTEHandler {
    prototype_declaration!(display_text, ch: char);
    prototype_declaration!(backspace);
    prototype_declaration!(horizontal_tab);
    prototype_declaration!(line_feed);

    fn vert_tab(&mut self) {
        self.line_feed();
    }

    fn form_feed(&mut self) {
        self.line_feed();
    }

    prototype_declaration!(carriage_return);
    prototype_declaration!(unsupported_execute_byte, byte: u8);

    // Hook
    prototype_declaration!(log_hook, message: &str);

    // OSC
    prototype_declaration!(log_osc, message: &str);
    prototype_declaration!(set_window_title, title: &str);
    prototype_declaration!(set_icon_name, name: &str);

    fn set_window_title_u8(&mut self, title: &[u8]) {
        if let Ok(title) = std::str::from_utf8(title) {
            self.set_window_title(title);
        }
    }

    fn set_icon_name_u8(&mut self, name: &[u8]) {
        if let Ok(name) = std::str::from_utf8(name) {
            self.set_icon_name(name);
        }
    }

    fn set_title_icon_name_u8(&mut self, string: &[u8]) {
        self.set_window_title_u8(string);
        self.set_icon_name_u8(string);
    }

    // CSI
    prototype_declaration!(insert_character, n: u16);
    prototype_declaration!(cursor_up, lines: u16);
    prototype_declaration!(cursor_down, lines: u16);
    prototype_declaration!(cursor_forward, columns: u16);
    prototype_declaration!(cursor_backward, columns: u16);
    prototype_declaration!(cursor_next_line, lines: u16);
    prototype_declaration!(cursor_previous_line, lines: u16);
    prototype_declaration!(cursor_horizontal_absolute, column: u16);
    prototype_declaration!(cursor_position, line: u16, column: u16);
    prototype_declaration!(set_mode, args: &vte::Params);
    prototype_declaration!(reset_mode, args: &vte::Params);
    prototype_declaration!(cursor_horizontal_tab, n: u16);
    prototype_declaration!(erase_in_display, n: u16);
    prototype_declaration!(erase_in_line, n: u16);
    prototype_declaration!(insert_line, n: u16);
    prototype_declaration!(delete_line, n: u16);
    prototype_declaration!(delete_character, n: u16);
    prototype_declaration!(scroll_up, n: u16);
    prototype_declaration!(scroll_down, n: u16);
    prototype_declaration!(erase_character, n: u16);
    prototype_declaration!(cursor_backward_tab, n: u16);
    prototype_declaration!(repeat_preceding, n: u16);
    prototype_declaration!(vertical_position_absolute, n: u16);
    prototype_declaration!(vertical_position_relative, n: u16);
    prototype_declaration!(selective_erase_in_display, n: u16);
    prototype_declaration!(selective_erase_in_line, n: u16);
    prototype_declaration!(select_graphic_rendition, args: &vte::Params);
    prototype_declaration!(dec_private_set_mode, args: &vte::Params);
    prototype_declaration!(dec_private_reset_mode, args: &vte::Params);

    prototype_declaration!(log_csi, message: &str);

    // ESC
    prototype_declaration!(save_cursor); // ESC 7
    prototype_declaration!(restore_cursor); // ESC 8
    prototype_declaration!(index); // ESC D
    prototype_declaration!(next_line); // ESC E
    prototype_declaration!(horizontal_tabulation_set); // ESC H
    prototype_declaration!(reverse_index); // ESC M
    prototype_declaration!(reset_to_initial_state); // ESC  c
    prototype_declaration!(keypad_application_mode); // ESC =
    prototype_declaration!(keypad_numeric_mode); // ESC >

    prototype_declaration!(log_esc, message: &str);
}

derive_perform!(dyn VTEHandler);

#[inline]
#[doc(Hidden)]
pub fn params_to_string(params: &vte::Params) -> String {
    let mut str = "{".to_string();

    let mut i = 0;
    for e in params.iter() {
        str.push('[');
        for i in 0..e.len() {
            str.push_str(&format!("{}", e[i]));

            if i != e.len() - 1 {
                str.push_str(", ");
            }
        }
        str.push(']');

        if i != params.len() - 1 {
            str.push_str(", ");
        }

        i = i + 1;
    }

    str.push('}');
    return str;
}

#[inline]
#[doc(Hidden)]
pub fn verify_parameters_single(params: &vte::Params, default: i64) -> i64 {
    match params.iter().next() {
        Some(sub_params) => {
            if sub_params.len() == 0 {
                return default;
            } else {
                return sub_params[0];
            }
        }
        None => return default,
    }
}

#[inline]
#[doc(Hidden)]
pub fn verify_parameters_single_range(
    params: &vte::Params,
    default: i64,
    lower: i64,
    upper: i64,
) -> i64 {
    match params.iter().next() {
        Some(sub_params) => {
            if sub_params.len() == 0 {
                return default;
            } else {
                if sub_params[0] < lower {
                    return default;
                } else if sub_params[0] > upper {
                    return default;
                } else {
                    return sub_params[0];
                }
            }
        }
        None => return default,
    }
}

#[inline]
#[doc(Hidden)]
pub fn verify_parameters_double(params: &vte::Params, default: (i64, i64)) -> (i64, i64) {
    match params.iter().next() {
        Some(sub_params) => {
            if sub_params.len() < 2 {
                return default;
            } else {
                return (sub_params[0], sub_params[1]);
            }
        }
        None => return default,
    }
}
