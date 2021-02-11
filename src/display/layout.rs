use super::panel::PanelPtr;

/// The different supported layouts of panels
pub enum Layout {
    Empty,
    Single {
        panel: PanelPtr,
    },
    VerticalStack {
        lower: PanelPtr,
        upper: PanelPtr,
    },
    HorizontalStack {
        left: PanelPtr,
        right: PanelPtr,
    },
    QuadStack {
        lower_left: PanelPtr,
        lower_right: PanelPtr,
        upper_left: PanelPtr,
        upper_right: PanelPtr,
    },
}

impl Layout {
    pub fn requires_vertical_line(&self) -> bool {
        return match self {
            Layout::Empty | Layout::Single { .. } | Layout::VerticalStack { .. } => false,
            Layout::HorizontalStack { .. } | Layout::QuadStack { .. } => true,
        };
    }

    pub fn requires_horizontal_line(&self) -> bool {
        return match self {
            Layout::Empty | Layout::Single { .. } | Layout::HorizontalStack { .. } => false,
            Layout::VerticalStack { .. } | Layout::QuadStack { .. } => true,
        };
    }
}
