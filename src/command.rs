#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum Command {
    EnterSingleCharacterCommand,
    CloseSelectedPanelCommand,
    OpenPanelCommand,
    FocusWorkspaceCommand(usize),
    SubdivideSelectedVerticalCommand,
    SubdivideSelectedHorizontalCommand,
    FocusPanelLeftCommand,
    FocusPanelRightCommand,
    FocusPanelUpCommand,
    FocusPanelDownCommand,
    MergePanelCommand,
    ScrollUpCommand,
    ScrollDownCommand,
    HelpMessageCommand,
    LockCommand,
    QuitCommand,
}

impl Command {
    pub fn get_name(&self) -> &str {
        return match self {
            Self::EnterSingleCharacterCommand => "EnterSingleCharacter",
            Self::CloseSelectedPanelCommand => "CloseSelectedPanel",
            Self::OpenPanelCommand => "OpenPanel",
            Self::FocusWorkspaceCommand(_) => "FocusWorkspace",
            Self::SubdivideSelectedVerticalCommand => "SubdivideSelectedVertical",
            Self::SubdivideSelectedHorizontalCommand => "SubdivideSelectedHorizontal",
            Self::FocusPanelLeftCommand => "FocusPanelLeft",
            Self::FocusPanelRightCommand => "FocusPanelRight",
            Self::FocusPanelUpCommand => "FocusPanelUp",
            Self::FocusPanelDownCommand => "FocusPanelDown",
            Self::MergePanelCommand => "MergePanel",
            Self::ScrollUpCommand => "ScrollUp",
            Self::ScrollDownCommand => "ScrollDown",
            Self::HelpMessageCommand => "Help",
            Self::LockCommand => "Lock",
            Self::QuitCommand => "Quit",
        };
    }

    pub fn help_text(&self) -> Option<String> {
        return Some(match self {
            Self::CloseSelectedPanelCommand => "Close selected panel".to_string(),
            Self::OpenPanelCommand => "Open new panel".to_string(),
            Self::FocusWorkspaceCommand(n) => format!("Focus workspace {}", n),
            Self::SubdivideSelectedVerticalCommand => {
                "Split panel with a vertical line".to_string()
            }
            Self::SubdivideSelectedHorizontalCommand => {
                "Split panel with a horizontal line".to_string()
            }
            Self::FocusPanelLeftCommand => "Focus panel to the left".to_string(),
            Self::FocusPanelRightCommand => "Focus panel to the right".to_string(),
            Self::FocusPanelUpCommand => "Focus panel upwards".to_string(),
            Self::FocusPanelDownCommand => "Focus panel downwards".to_string(),
            Self::MergePanelCommand => "Merge empty split".to_string(),
            Self::ScrollUpCommand => "Scroll panel up".to_string(),
            Self::ScrollDownCommand => "Scroll panel down".to_string(),
            Self::HelpMessageCommand => "Display help".to_string(),
            Self::LockCommand => "Lock the display".to_string(),
            Self::QuitCommand => "Quit".to_string(),
            _ => return None,
        });
    }

    pub fn args(&self) -> Vec<String> {
        return match self {
            Command::FocusWorkspaceCommand(a) => vec![format!("{}", a)],
            _ => Vec::new(),
        };
    }

    pub fn try_from_string(name: String, mut args: Vec<String>) -> Result<Self, String> {
        let lowered_name = name.to_lowercase();

        let mut required_1_arg = true;

        let cmd = match lowered_name.as_str() {
            "entersinglecharacter" => Self::EnterSingleCharacterCommand,
            "openpanel" => Self::OpenPanelCommand,
            "quit" => Self::QuitCommand,
            "subdivideselectedhorizontal" => Self::SubdivideSelectedHorizontalCommand,
            "subdivideselectedvertical" => Self::SubdivideSelectedVerticalCommand,
            "focuspanelleft" => Self::FocusPanelLeftCommand,
            "focuspanelright" => Self::FocusPanelRightCommand,
            "focuspanelup" => Self::FocusPanelUpCommand,
            "focuspaneldown" => Self::FocusPanelDownCommand,
            "mergepanel" => Self::MergePanelCommand,
            "closeselectedpanel" => Self::CloseSelectedPanelCommand,
            "lock" => Self::LockCommand,
            "scrollup" => Self::ScrollUpCommand,
            "scrolldown" => Self::ScrollDownCommand,
            "help" => Self::HelpMessageCommand,
            "focusworkspace" => {
                if args.len() != 1 {
                    return Err(
                        "The focus workspace command must be supplied an integer argument."
                            .to_string(),
                    );
                }

                let arg = args.pop().unwrap().parse::<usize>().map_err(|_| {
                    "The focus workspace command must be supplied an integer argument.".to_string()
                })?;

                required_1_arg = false;
                Self::FocusWorkspaceCommand(arg)
            }
            _ => return Err(format!("Unknown command: {}", name)),
        };

        if required_1_arg && args.len() != 0 {
            return Err(format!(
                "The {} command expects 0 arguments but {} were provided",
                cmd.get_name(),
                args.len()
            ));
        }

        return Ok(cmd);
    }
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", self.get_name());
    }
}
