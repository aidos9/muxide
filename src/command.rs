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
    MergePanelLeftCommand,
    MergePanelRightCommand,
    MergePanelUpCommand,
    MergePanelDownCommand,
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
            Self::MergePanelLeftCommand => "MergePanelLeft",
            Self::MergePanelRightCommand => "MergePanelRight",
            Self::MergePanelUpCommand => "MergePanelUp",
            Self::MergePanelDownCommand => "MergePanelDown",
            Self::LockCommand => "Lock",
            Self::QuitCommand => "Quit",
        };
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
            "mergepanelleft" => Self::MergePanelLeftCommand,
            "mergepanelright" => Self::MergePanelRightCommand,
            "mergepanelup" => Self::MergePanelUpCommand,
            "mergepaneldown" => Self::MergePanelDownCommand,
            "closeselectedpanel" => Self::CloseSelectedPanelCommand,
            "lock" => Self::LockCommand,
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
