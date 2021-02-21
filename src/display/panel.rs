use crate::geometry::{Point, Size};
use std::cell::RefCell;
use std::rc::Rc;

/// Defines a method that calls a method with the same name and args defined in panel from PanelPtr
macro_rules! wrap_panel_method {
    ($method_name:ident, mut, $($arg_name:ident: $arg_type:ty),* $(=> $return_type:ty)?) => {
        fn $method_name (&mut self, $($arg_name : $arg_type),*) $(-> $return_type)? {
            let mut mut_ref = self.0.borrow_mut();
            return mut_ref.$method_name($($arg_name : $arg_type),*);
        }
    };

        ($method_name:ident, $($arg_name:ident: $arg_type:ty),* $(=> $return_type:ty)?) => {
        fn $method_name (&self, $($arg_name : $arg_type),*) $(-> $return_type)? {
            return self.0.borrow().$method_name($($arg_name : $arg_type),*);
        }
    };

    ($method_name:ident, pub mut, $($arg_name:ident: $arg_type:ty),* $(=> $return_type:ty)?) => {
        pub fn $method_name (&mut self, $($arg_name : $arg_type),*) $(-> $return_type)? {
            let mut mut_ref = self.0.borrow_mut();
            return mut_ref.$method_name($($arg_name),*);
        }
    };

    ($method_name:ident, pub, $($arg_name:ident: $arg_type:ty),* $(=> $return_type:ty)?) => {
        pub fn $method_name (&self, $($arg_name : $arg_type),*) $(-> $return_type)? {
            return self.0.borrow().$method_name($($arg_name : $arg_type),*);
        }
    };
}

#[derive(Clone)]
/// A wrapper of the panel struct that acts as a pointer
pub struct PanelPtr(Rc<RefCell<Panel>>);

/// A panel is all the information required for a process.
struct Panel {
    id: usize,
    size: Size,
    content: Vec<Vec<u8>>,
    hide_cursor: bool,
    cursor_col: u16,
    cursor_row: u16,
    location: (u16, u16), // (col, row). The location in the global space of the top left (the first) cell
}

impl PanelPtr {
    pub fn new(id: usize, size: Size, location: (u16, u16)) -> Self {
        return Self(Rc::new(RefCell::new(Panel::new(id, size, location))));
    }

    wrap_panel_method!(set_location, pub mut, location: (u16, u16));
    wrap_panel_method!(get_cursor_position, pub, => Point<u16>);
    wrap_panel_method!(set_cursor_position, pub mut, col: u16, row: u16);
    wrap_panel_method!(set_content, pub mut, content: Vec<Vec<u8>>);
    wrap_panel_method!(get_content, pub, => Vec<Vec<u8>>);
    wrap_panel_method!(get_id, pub, => usize);
    wrap_panel_method!(set_size, pub mut, size: Size);
    wrap_panel_method!(get_size, pub, => Size);
    wrap_panel_method!(get_hide_cursor, pub, => bool);
    wrap_panel_method!(set_hide_cursor, pub mut, hide: bool);
}

impl Panel {
    pub fn new(id: usize, size: Size, location: (u16, u16)) -> Self {
        return Self {
            content: Vec::new(),
            size,
            id,
            location,
            hide_cursor: false,
            cursor_col: 0,
            cursor_row: 0,
        };
    }

    /// Set the origin of the panel's top left corner in the global display. (col, row).
    pub fn set_location(&mut self, location: (u16, u16)) {
        self.location = location;
    }

    /// Returns the cursor position in the global space.
    pub fn get_cursor_position(&self) -> Point<u16> {
        return Point::new_origin(self.cursor_col, self.cursor_row, self.location);
    }

    pub fn set_cursor_position(&mut self, col: u16, row: u16) {
        self.cursor_col = col;
        self.cursor_row = row;
    }

    /// Set the content of this panel
    pub fn set_content(&mut self, content: Vec<Vec<u8>>) {
        self.content = content;
    }

    /// Returns an immutable reference to the content of this panel
    pub fn get_content(&self) -> Vec<Vec<u8>> {
        return self.content.clone();
    }

    pub fn get_id(&self) -> usize {
        return self.id;
    }

    pub fn set_size(&mut self, size: Size) {
        self.size = size;
    }

    pub fn get_size(&self) -> Size {
        return self.size;
    }

    pub fn get_hide_cursor(&self) -> bool {
        return self.hide_cursor;
    }

    pub fn set_hide_cursor(&mut self, hide: bool) {
        self.hide_cursor = hide;
    }
}
