use nix::pty::Winsize;
use num_traits::{PrimInt, Unsigned, Zero};
use std::fmt::Display;
use std::ops::Sub;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct Size {
    rows: u16,
    cols: u16,
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct Point<T: PrimInt + Unsigned + Zero> {
    x: T,
    y: T,
    origin: (T, T),
}

impl Size {
    pub fn new(rows: u16, cols: u16) -> Self {
        return Self { rows, cols };
    }

    pub fn to_winsize(&self) -> Winsize {
        return Winsize {
            ws_row: self.rows,
            ws_col: self.cols,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
    }

    pub fn get_cols(&self) -> u16 {
        return self.cols;
    }

    pub fn get_rows(&self) -> u16 {
        return self.rows;
    }

    pub fn divide_width_by_const(&mut self, constant: u16) {
        self.cols /= constant;
    }
}

impl Sub for Size {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        return Self::new(self.rows - rhs.rows, self.cols - rhs.cols);
    }
}

impl<T: PrimInt + Unsigned + Zero> Point<T> {
    /// Treats (0, 0) as the origin.
    #[allow(dead_code)]
    pub fn new(column: T, row: T) -> Self {
        return Self {
            x: column,
            y: row,
            origin: (T::zero(), T::zero()),
        };
    }

    /// Creates a new point and shifts by (x, y)
    pub fn new_origin(column: T, row: T, origin: (T, T)) -> Self {
        return Self {
            x: column + origin.0,
            y: row + origin.1,
            origin,
        };
    }

    #[allow(dead_code)]
    pub fn get_origin(&self) -> (T, T) {
        return self.origin;
    }

    /// Get, the x component of this point
    pub fn column(&self) -> T {
        return self.x;
    }

    /// Get, the y component of this point
    pub fn row(&self) -> T {
        return self.y;
    }
}

impl<T: PrimInt + Unsigned + Zero + Display> Display for Point<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "(x: {}, y: {})", self.column(), self.row());
    }
}

impl std::fmt::Display for Size {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{{width: {}, height: {}}}", self.cols, self.rows);
    }
}
