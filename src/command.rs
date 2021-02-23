use termion::event::Key;

#[derive(Clone, PartialEq, Debug)]
pub enum Command {
    EnterSingleCharacterCommand,
    FocusCommandPromptCommand,
    StopEnteringInputCommand,
    ToggleInputCommand,
    ArrowLeftCommand,
    ArrowRightCommand,
    ArrowUpCommand,
    ArrowDownCommand,
    ClosePanelCommand(usize),
    CloseMostRecentPanelCommand,
    OpenPanelCommand,
    SwapPanelsCommand(usize, usize),
    FocusPanelCommand(usize),
    IdentifyPanelsCommand,
    MapCommand(Key, Box<Command>),
    UnMapCommand(Key),
    SubdivideSelectedVertical,
    SubdivideSelectedHorizontal,
    QuitCommand,
}

impl Command {
    pub fn get_name(&self) -> &str {
        return match self {
            Self::EnterSingleCharacterCommand => "EnterSingleCharacter",
            Self::FocusCommandPromptCommand => "FocusCommandPrompt",
            Self::StopEnteringInputCommand => "StopEnteringInput",
            Self::ToggleInputCommand => "ToggleInput",
            Self::ArrowLeftCommand => "ArrowLeft",
            Self::ArrowRightCommand => "ArrowRight",
            Self::ArrowUpCommand => "ArrowUp",
            Self::ArrowDownCommand => "ArrowDown",
            Self::ClosePanelCommand(_) => "ClosePanel",
            Self::CloseMostRecentPanelCommand => "CloseMostRecentPanel",
            Self::OpenPanelCommand => "OpenPanel",
            Self::SwapPanelsCommand(_, _) => "SwapPanels",
            Self::FocusPanelCommand(_) => "FocusPanel",
            Self::IdentifyPanelsCommand => "Identify",
            Self::MapCommand(_, _) => "Map",
            Self::UnMapCommand(_) => "UnMap",
            Self::SubdivideSelectedVertical => "SubdivideSelectedVertical",
            Self::SubdivideSelectedHorizontal => "SubdivideSelectedHorizontal",
            Self::QuitCommand => "Quit",
        };
    }

    pub fn try_from_string(name: String, mut args: Vec<String>) -> Result<Self, String> {
        let lowered_name = name.to_lowercase();

        let mut required_1_arg = true;

        let cmd = match lowered_name.as_str() {
            "entersinglecharacter" => Self::EnterSingleCharacterCommand,
            "focuscommandprompt" => Self::FocusCommandPromptCommand,
            "stopenteringinput" => Self::StopEnteringInputCommand,
            "toggleinput" => Self::ToggleInputCommand,
            "arrowleft" => Self::ArrowLeftCommand,
            "arrowright" => Self::ArrowRightCommand,
            "arrowup" => Self::ArrowUpCommand,
            "arrowdown" => Self::ArrowDownCommand,
            "openpanel" => Self::OpenPanelCommand,
            "identify" => Self::IdentifyPanelsCommand,
            "quit" => Self::QuitCommand,
            "subdivideselectedhorizontal" => Self::SubdivideSelectedHorizontal,
            "subdivideselectedvertical" => Self::SubdivideSelectedVertical,
            "closepanel" => {
                if args.len() != 1 {
                    return Err(
                        "The close panel command must be supplied an integer argument.".to_string(),
                    );
                }

                let arg = args.pop().unwrap().parse::<usize>().map_err(|_| {
                    "The close panel command must be supplied an integer argument.".to_string()
                })?;

                required_1_arg = false;
                Self::ClosePanelCommand(arg)
            }
            "focuspanel" => {
                if args.len() != 1 {
                    return Err(
                        "The focus panel command must be supplied an integer argument.".to_string(),
                    );
                }

                let arg = args.pop().unwrap().parse::<usize>().map_err(|_| {
                    "The focus panel command must be supplied an integer argument.".to_string()
                })?;

                required_1_arg = false;
                Self::FocusPanelCommand(arg)
            }
            "swappanelscommand" => {
                if args.len() != 2 {
                    return Err(
                        "The swap panels command must be supplied 2 integer arguments.".to_string(),
                    );
                }

                let arg_2 = args.pop().unwrap();
                let arg_1 = args.pop().unwrap();

                let arg_2 = arg_2.parse::<usize>().map_err(|_| "The swap panels command must be supplied 2 integer arguments. Arg 2 was not an integer.".to_string())?;
                let arg_1 = arg_1.parse::<usize>().map_err(|_| "The swap panels command must be supplied 2 integer arguments. Arg 1 was not an integer.".to_string())?;

                required_1_arg = false;
                Self::SwapPanelsCommand(arg_1, arg_2)
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
