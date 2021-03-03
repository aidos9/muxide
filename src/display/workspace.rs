use super::{panel::PanelPtr, subdivision::SubDivision};

#[derive(Clone, Debug)]
pub struct Workspace {
    pub panels: Vec<PanelPtr>,
    pub selected_panel: Option<PanelPtr>,
    pub root_subdivision: SubDivision,
}

impl Workspace {
    pub fn new() -> Self {
        return Self {
            panels: Vec::new(),
            selected_panel: None,
            root_subdivision: SubDivision::default(),
        };
    }
}
