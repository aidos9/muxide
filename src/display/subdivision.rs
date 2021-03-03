use super::panel::PanelPtr;
use crate::{
    geometry::{Direction, Point, Size},
    Config, ErrorType, MuxideError,
};
use crossterm::{cursor, queue, style};
use std::io::{Stdout, Write};

/// The text that is displayed when there are no open panels.
const EMPTY_TEXT: &'static str = "No Panels Open";

macro_rules! queue_map_err {
    ($($v:expr),*) => {
        queue!($($v),*).map_err(|e| {
            ErrorType::QueueExecuteError {
                reason: e.to_string(),
            }
            .into_error()
        });
    };
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct SubdivisionPath {
    elements: Vec<SubdivisionPathElement>,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
enum SubdivisionPathElement {
    A,
    B,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum SubDivisionSplit {
    Horizontal,
    Vertical,
}

#[derive(Clone, Debug)]
/// A subdivision either contains a panel or contains two other subdivisions
pub struct SubDivision {
    panel: Option<PanelPtr>,
    subdiv_a: Option<Box<SubDivision>>,
    subdiv_b: Option<Box<SubDivision>>,
    /// Whether or not this subdivision is split vertically, horizontally or not at all.
    split: Option<SubDivisionSplit>,
    origin: Point<u16>,
    dimensions: Size,
}

impl SubDivision {
    pub const fn new(origin: Point<u16>, dimensions: Size) -> Self {
        return Self {
            panel: None,
            subdiv_a: None,
            subdiv_b: None,
            split: None,
            origin,
            dimensions,
        };
    }

    pub fn close_panel_with_id(&mut self, id: usize) -> bool {
        if let Some(path) = self.path_for_panel_id(id) {
            return self.close_panel_at_path(path);
        } else {
            return false;
        }
    }

    fn close_panel_at_path(&mut self, mut path: SubdivisionPath) -> bool {
        match path.pop() {
            Some(SubdivisionPathElement::A) => {
                if let Some(subdiv) = self.subdiv_a.as_mut() {
                    return subdiv.close_panel_at_path(path);
                } else {
                    return false;
                }
            }
            Some(SubdivisionPathElement::B) => {
                if let Some(subdiv) = self.subdiv_b.as_mut() {
                    return subdiv.close_panel_at_path(path);
                } else {
                    return false;
                }
            }
            None => {
                if self.panel.is_none() {
                    return false;
                } else {
                    self.panel = None;
                    return true;
                }
            }
        }
    }

    pub fn next_panel_details(&self) -> Option<(SubdivisionPath, Size, Point<u16>)> {
        if self.subdiv_a.is_some() && self.subdiv_b.is_some() {
            if let Some(mut path) = self.subdiv_a.as_ref().unwrap().next_panel_details() {
                path.0.push(SubdivisionPathElement::A);
                return Some(path);
            } else if let Some(mut path) = self.subdiv_b.as_ref().unwrap().next_panel_details() {
                path.0.push(SubdivisionPathElement::B);
                return Some(path);
            } else {
                return None;
            }
        } else if self.panel.is_none() {
            return Some((SubdivisionPath::new(), self.dimensions, self.origin));
        } else {
            return None;
        }
    }

    pub fn open_panel_at_path(
        &mut self,
        panel: PanelPtr,
        mut path: SubdivisionPath,
    ) -> Result<(), MuxideError> {
        match path.pop() {
            Some(SubdivisionPathElement::A) => {
                if self.subdiv_a.is_none() {
                    panic!("Invalid path");
                } else {
                    return self
                        .subdiv_a
                        .as_mut()
                        .unwrap()
                        .open_panel_at_path(panel, path);
                }
            }
            Some(SubdivisionPathElement::B) => {
                if self.subdiv_b.is_none() {
                    panic!("Invalid path");
                } else {
                    return self
                        .subdiv_b
                        .as_mut()
                        .unwrap()
                        .open_panel_at_path(panel, path);
                }
            }
            None => {
                if self.panel.is_some() {
                    panic!("Invalid path");
                } else {
                    self.panel = Some(panel);
                }
            }
        }

        return Ok(());
    }

    pub fn focus_next_id(&self, selected_id: usize, focus_direction: Direction) -> Option<usize> {
        let path = self.path_for_panel_id(selected_id)?;

        return self.focus_next_id_internal(path, focus_direction);
    }

    fn focus_next_id_internal(
        &self,
        mut selected_path: SubdivisionPath,
        focus_direction: Direction,
    ) -> Option<usize> {
        match selected_path.pop() {
            Some(SubdivisionPathElement::A) => {
                if let Some(subdiv_a) = self.subdiv_a.as_ref() {
                    let alt =
                        self.check_for_possible_id(SubdivisionPathElement::A, focus_direction);

                    return subdiv_a
                        .focus_next_id_internal(selected_path, focus_direction)
                        .or(alt);
                } else {
                    return None;
                }
            }
            Some(SubdivisionPathElement::B) => {
                if let Some(subdiv_b) = self.subdiv_b.as_ref() {
                    let alt =
                        self.check_for_possible_id(SubdivisionPathElement::B, focus_direction);

                    return subdiv_b
                        .focus_next_id_internal(selected_path, focus_direction)
                        .or(alt);
                } else {
                    return None;
                }
            }
            None => {
                return None;
            }
        }
    }

    fn check_for_possible_id(
        &self,
        path_element: SubdivisionPathElement,
        focus_direction: Direction,
    ) -> Option<usize> {
        match focus_direction {
            Direction::Up => {
                if self.split == Some(SubDivisionSplit::Horizontal) {
                    if path_element.is_b() {
                        return self.subdiv_a.as_ref().unwrap().tail_b_for_id();
                    }
                }

                return None;
            }
            Direction::Down => {
                if self.split == Some(SubDivisionSplit::Horizontal) {
                    if path_element.is_a() {
                        return self.subdiv_b.as_ref().unwrap().tail_a_for_id();
                    }
                }

                return None;
            }
            Direction::Left => {
                if self.split == Some(SubDivisionSplit::Vertical) {
                    if path_element.is_b() {
                        return self.subdiv_a.as_ref().unwrap().tail_b_for_id();
                    }
                }

                return None;
            }
            Direction::Right => {
                if self.split == Some(SubDivisionSplit::Vertical) {
                    if path_element.is_a() {
                        return self.subdiv_b.as_ref().unwrap().tail_a_for_id();
                    }
                }

                return None;
            }
        }
    }

    fn tail_b_for_id(&self) -> Option<usize> {
        if self.panel.is_some() {
            return Some(self.panel.as_ref().unwrap().get_id());
        } else if let (Some(subdiv_a), Some(subdiv_b)) =
            (self.subdiv_a.as_ref(), self.subdiv_b.as_ref())
        {
            let mut res = subdiv_b.tail_b_for_id();

            if res.is_none() {
                res = subdiv_a.tail_b_for_id();
            }

            return res;
        } else {
            return None;
        }
    }

    fn tail_a_for_id(&self) -> Option<usize> {
        if self.panel.is_some() {
            return Some(self.panel.as_ref().unwrap().get_id());
        } else if let (Some(subdiv_a), Some(subdiv_b)) =
            (self.subdiv_a.as_ref(), self.subdiv_b.as_ref())
        {
            let mut res = subdiv_a.tail_b_for_id();

            if res.is_none() {
                res = subdiv_b.tail_a_for_id();
            }

            return res;
        } else {
            return None;
        }
    }

    fn path_for_panel_id(&self, id: usize) -> Option<SubdivisionPath> {
        if let Some(panel) = self.panel.as_ref() {
            if panel.get_id() == id {
                return Some(SubdivisionPath::new());
            } else {
                return None;
            }
        } else if let (Some(subdiv_a), Some(subdiv_b)) =
            (self.subdiv_a.as_ref(), self.subdiv_b.as_ref())
        {
            if let Some(mut path) = subdiv_a.path_for_panel_id(id) {
                path.push(SubdivisionPathElement::A);
                return Some(path);
            } else if let Some(mut path) = subdiv_b.path_for_panel_id(id) {
                path.push(SubdivisionPathElement::B);
                return Some(path);
            } else {
                return None;
            }
        } else {
            return None;
        }
    }

    pub fn split_panel(
        &mut self,
        panel_id: Option<usize>,
        direction: SubDivisionSplit,
    ) -> (Option<Size>, bool) {
        if panel_id.is_none() {
            if self.panel.is_none() && self.subdiv_a.is_none() && self.subdiv_b.is_none() {
                match direction {
                    SubDivisionSplit::Horizontal => self.subdivide_horizontal(),
                    SubDivisionSplit::Vertical => self.subdivide_vertical(),
                }

                return (None, true);
            } else {
                return (None, false);
            }
        }

        let panel_id = panel_id.unwrap();

        if self.panel.is_some() && self.panel.as_ref().unwrap().get_id() == panel_id {
            match direction {
                SubDivisionSplit::Horizontal => self.subdivide_horizontal(),
                SubDivisionSplit::Vertical => self.subdivide_vertical(),
            }

            let new_size = self
                .subdiv_a
                .as_mut()
                .unwrap()
                .set_panel(self.panel.take().unwrap());

            return (Some(new_size), true);
        } else if self.panel.is_none() && self.subdiv_a.is_some() && self.subdiv_b.is_some() {
            let res_a = self
                .subdiv_a
                .as_mut()
                .unwrap()
                .split_panel(Some(panel_id), direction);
            if res_a.1 {
                return res_a;
            } else {
                return self
                    .subdiv_b
                    .as_mut()
                    .unwrap()
                    .split_panel(Some(panel_id), direction);
            }
        } else {
            return (None, false);
        }
    }

    fn set_panel(&mut self, mut panel: PanelPtr) -> Size {
        panel.set_location((self.origin.column(), self.origin.row()));

        self.panel = Some(panel);
        return self.dimensions;
    }

    fn subdivide_vertical(&mut self) {
        let mut subdiv_a_dimensions = self.dimensions - Size::new(0, 1); // -1 for the center column
        subdiv_a_dimensions.divide_width_by_const(2);

        let subdiv_b_dimensinos =
            self.dimensions - Size::new(0, 1) - Size::new(0, subdiv_a_dimensions.get_cols());

        self.subdiv_a = Some(Box::new(SubDivision::new(self.origin, subdiv_a_dimensions)));

        self.subdiv_b = Some(Box::new(SubDivision::new(
            self.origin + Point::new(subdiv_a_dimensions.get_cols() + 1, 0),
            subdiv_b_dimensinos,
        )));

        self.split = Some(SubDivisionSplit::Vertical); // The split line will be drawn vertically.
    }

    fn subdivide_horizontal(&mut self) {
        let mut subdiv_a_dimensions = self.dimensions - Size::new(1, 0); // -1 for the center row
        subdiv_a_dimensions.divide_height_by_const(2);

        let subdiv_b_dimensinos =
            self.dimensions - Size::new(1, 0) - Size::new(subdiv_a_dimensions.get_rows(), 0);

        self.subdiv_a = Some(Box::new(SubDivision::new(self.origin, subdiv_a_dimensions)));

        //TODO: Test if this works
        self.subdiv_b = Some(Box::new(SubDivision::new(
            self.origin + Point::new(0, subdiv_a_dimensions.get_rows() + 1),
            subdiv_b_dimensinos,
        )));

        self.split = Some(SubDivisionSplit::Horizontal); // The split line will be drawn vertically.
    }

    pub fn render(&self, stdout: &mut Stdout, config: &Config) -> Result<(), MuxideError> {
        if self.panel.is_none() && self.subdiv_a.is_none() && self.subdiv_b.is_none() {
            let (mut col, mut row) = (self.dimensions.get_cols(), self.dimensions.get_rows());

            // Determine the center
            col /= 2;
            // Align the empty text to the center
            col -= EMPTY_TEXT.len() as u16 / 2;

            // Determine the center
            row /= 2;
            // Subtract 1 for the height of the text
            row -= 1;

            // Add 1 to offset by the left and top borders. Obviously it is useless having
            // the + and - operations that cancel each other but for clarity's sake they have
            // been used.
            queue_map_err!(
                stdout,
                cursor::MoveTo(self.origin.column() + col, self.origin.row() + row),
                style::Print(EMPTY_TEXT)
            )?;

            return Ok(());
        } else if self.panel.is_none() && self.subdiv_a.is_some() && self.subdiv_b.is_some() {
            self.subdiv_a.as_ref().unwrap().render(stdout, config)?;
            self.subdiv_b.as_ref().unwrap().render(stdout, config)?;

            Self::reset_stdout_style(stdout)?;

            match &self.split {
                Some(SubDivisionSplit::Vertical) => {
                    let center_col = self.dimensions.get_cols() / 2 + self.origin.column() - 1;
                    self.queue_vertical_line(stdout, config, center_col)?;
                }
                Some(SubDivisionSplit::Horizontal) => {
                    let center_row = self.dimensions.get_rows() / 2 + self.origin.row() - 1;
                    self.queue_horizontal_line(stdout, config, center_row)?;
                }
                None => panic!("Unexpected internal error."), // This shouldn't ever happen.
            }

            return Ok(());
        } else if let Some(panel) = &self.panel {
            for (row_number, row) in panel.get_content().into_iter().enumerate() {
                queue_map_err!(
                    stdout,
                    cursor::MoveTo(self.origin.column(), self.origin.row() + row_number as u16),
                    style::ResetColor
                )?;

                stdout
                    .write(&row)
                    .map_err(|e| ErrorType::new_display_qe_error(e))?;
            }

            return Ok(());
        } else {
            return Err(ErrorType::InvalidSubdivisionState.into_error());
        }
    }

    fn queue_vertical_line(
        &self,
        stdout: &mut Stdout,
        config: &Config,
        col: u16,
    ) -> Result<(), MuxideError> {
        let ch = config.get_borders_ref().get_vertical_char();

        for r in 0..self.dimensions.get_rows() {
            queue_map_err!(
                stdout,
                cursor::MoveTo(col, self.origin.row() + r),
                style::Print(ch)
            )?;
        }

        return Ok(());
    }

    fn queue_horizontal_line(
        &self,
        stdout: &mut Stdout,
        config: &Config,
        row: u16,
    ) -> Result<(), MuxideError> {
        let ch = config.get_borders_ref().get_horizontal_char();

        for c in 0..self.dimensions.get_cols() {
            queue_map_err!(
                stdout,
                cursor::MoveTo(self.origin.column() + c, row),
                style::Print(ch)
            )?;
        }

        return Ok(());
    }

    fn reset_stdout_style(stdout: &mut Stdout) -> Result<(), MuxideError> {
        queue_map_err!(stdout, style::ResetColor)?;

        return Ok(());
    }
}

impl Default for SubDivision {
    fn default() -> Self {
        return Self::new(Point::new(0, 0), Size::new(0, 0));
    }
}

impl SubdivisionPath {
    fn new() -> Self {
        return Self {
            elements: Vec::new(),
        };
    }

    fn push(&mut self, element: SubdivisionPathElement) {
        self.elements.push(element);
    }

    fn pop(&mut self) -> Option<SubdivisionPathElement> {
        return self.elements.pop();
    }
}

impl SubdivisionPathElement {
    pub fn is_a(&self) -> bool {
        return *self == SubdivisionPathElement::A;
    }

    pub fn is_b(&self) -> bool {
        return *self == SubdivisionPathElement::B;
    }
}
