use std::error::Error;

#[derive(Clone, PartialEq, Debug, Hash)]
pub enum ErrorType {
    IOCTLError {
        code: i32,
        outcome: String,
    },

    PTYSpawnError {
        description: String,
    },

    PollCreationError {
        reason: String,
    },

    DetermineTerminalSizeError {
        reason: String,
    },

    PollingError {
        reason: String,
    },

    IOError {
        read: bool,
        target: String,
        reason: String,
    },

    StdoutFlushError {
        reason: String,
    },

    OpenPTYError {
        reason: String,
    },

    FCNTLError {
        reason: String,
    },

    FailedTTYAcquisitionError {
        reason: String,
    },

    EnterRawModeError {
        reason: String,
    },

    NoPanelWithIDError {
        id: usize,
    },

    QueueExecuteError {
        reason: String,
    },

    ScriptError {
        description: String,
    },

    PTYWriteError {
        description: String,
    },

    CommandError {
        description: String,
    },

    EventParsingError {
        message: String,
    },

    DisplayNotRunningError,
    InputManagerRunningError,
    InvalidSubdivisionState,
    NoAvailableSubdivision,
    FailedSubdivision,
    PtyStdinReceiverClosed,
    FailedReadPoll,
    FailedToSendMessage,
    FailedToReadPTY,
    PTYStoppedRunning,
    FailedToWriteToPTY,
    NoWorkspaceWithID(usize),
    DisplayLocked,
    InvalidPassword,
    FailedToCheckPassword,
    NoAvailableSubdivisionToMerge,
    NoSubdivisionAtPath,
    NoPanelAtPath,
}

#[derive(Clone, PartialEq, Hash)]
pub struct MuxideError {
    debug_description: String,
    description: String,
    terminate: bool,
}

impl ErrorType {
    pub fn into_error(self) -> MuxideError {
        return MuxideError::new(self);
    }

    pub fn new_display_qe_error(io_error: std::io::Error) -> MuxideError {
        return Self::QueueExecuteError {
            reason: io_error.to_string(),
        }
        .into_error();
    }
}

impl MuxideError {
    pub fn new(tp: ErrorType) -> Self {
        return match tp {
            ErrorType::IOCTLError { code, outcome } => Self::new_ioctl_error(code, outcome),
            ErrorType::PTYSpawnError { description } => Self::new_pty_spawn_error(description),
            ErrorType::PollCreationError { reason } => Self::new_poll_creation_error(reason),
            ErrorType::DetermineTerminalSizeError { reason } => {
                Self::new_determine_terminal_size_error(reason)
            }
            ErrorType::PollingError { reason } => Self::new_polling_error(reason),
            ErrorType::IOError {
                read,
                target,
                reason,
            } => {
                if read {
                    Self::new_read_io_error(target, reason)
                } else {
                    Self::new_write_io_error(target, reason)
                }
            }
            ErrorType::StdoutFlushError { reason } => return Self::new_stdout_flush_error(reason),
            ErrorType::OpenPTYError { reason } => return Self::new_open_pty_error(reason),
            ErrorType::FCNTLError { reason } => return Self::new_fcntl_error(reason),
            ErrorType::DisplayNotRunningError => return Self::new_display_not_running_error(),
            ErrorType::InputManagerRunningError => return Self::new_input_manager_running_error(),
            ErrorType::FailedTTYAcquisitionError { reason } => {
                return Self::new_failed_tty_acquisition_error(reason)
            }

            ErrorType::EnterRawModeError { reason } => {
                return Self::new_enter_raw_mode_error(reason)
            }

            ErrorType::NoPanelWithIDError { id } => {
                return Self::new_no_panel_with_id(id);
            }

            ErrorType::QueueExecuteError { reason } => {
                return Self::new_queue_execute_error(reason);
            }

            ErrorType::ScriptError { description } => {
                return Self::new_script_error(description);
            }

            ErrorType::PTYWriteError { description } => {
                return Self::new_pty_write_error(description);
            }

            ErrorType::CommandError { description } => {
                return Self::new_command_error(description);
            }

            ErrorType::EventParsingError { message } => {
                return Self::new_event_parsing_error(message);
            }

            ErrorType::InvalidSubdivisionState => {
                return Self::new_invalid_subdivision_state_error();
            }

            ErrorType::NoAvailableSubdivision => {
                return Self::new_no_available_subdivision_error();
            }

            ErrorType::FailedSubdivision => {
                return Self {
                    debug_description: "Failed to subdivide panel.".to_string(),
                    description: "Failed to subdivide panel.".to_string(),
                    terminate: false,
                };
            }

            ErrorType::PtyStdinReceiverClosed => {
                return Self {
                    debug_description: "The pty's stdin receiver closed.".to_string(),
                    description: "The pty's stdin receiver closed.".to_string(),
                    terminate: true,
                };
            }

            ErrorType::FailedReadPoll => {
                return Self {
                    debug_description: "Failed to poll the pty for data.".to_string(),
                    description: "Failed to poll the pty for data.".to_string(),
                    terminate: false,
                };
            }

            ErrorType::FailedToSendMessage => {
                return Self {
                    debug_description: "Failed to send message from pty thread.".to_string(),
                    description: "Failed to communicate data from the pty.".to_string(),
                    terminate: false,
                };
            }

            ErrorType::FailedToReadPTY => {
                return Self {
                    debug_description: "Failed to read data from pty.".to_string(),
                    description: "Failed to read data from the pty.".to_string(),
                    terminate: false,
                };
            }

            ErrorType::PTYStoppedRunning => {
                return Self {
                    debug_description: "PTY unexpectedly stopped running.".to_string(),
                    description: "PTY unexpectedly stopped running.".to_string(),
                    terminate: false,
                };
            }

            ErrorType::FailedToWriteToPTY => {
                return Self {
                    debug_description: "Failed to write data to PTY.".to_string(),
                    description: "Failed to write data to PTY.".to_string(),
                    terminate: false,
                };
            }

            ErrorType::NoWorkspaceWithID(id) => {
                return Self {
                    debug_description: format!("No workspace with id: {}", id),
                    description: format!("No workspace number {}", id),
                    terminate: false,
                };
            }

            ErrorType::DisplayLocked => {
                return Self {
                    debug_description: "Display is locked.".to_string(),
                    description: "Display is locked.".to_string(),
                    terminate: false,
                };
            }

            ErrorType::InvalidPassword => {
                return Self {
                    debug_description: "Incorrect Password.".to_string(),
                    description: "Incorrect Password.".to_string(),
                    terminate: false,
                };
            }

            ErrorType::FailedToCheckPassword => {
                return Self {
                    debug_description: "Hash comparison failed.".to_string(),
                    description: "Failed to compare password.".to_string(),
                    terminate: false,
                };
            }

            ErrorType::NoAvailableSubdivisionToMerge => {
                return Self {
                    debug_description: "No open subdivision to merge.".to_string(),
                    description: "No open subdivision to merge.".to_string(),
                    terminate: false,
                };
            }

            ErrorType::NoSubdivisionAtPath => {
                return Self {
                    debug_description: "No subdivision at path.".to_string(),
                    description: "No subdivision at path.".to_string(),
                    terminate: false,
                };
            }

            ErrorType::NoPanelAtPath => {
                return Self {
                    debug_description: "No panel at path end.".to_string(),
                    description: "No panel at path end.".to_string(),
                    terminate: false,
                };
            }
        };
    }

    pub fn description(&self) -> String {
        return format!("PTY Error: {}", self.description);
    }

    pub fn debug_description(&self) -> String {
        return format!("PTY Error: {}", self.debug_description);
    }

    pub fn should_terminate(&self) -> bool {
        return self.terminate;
    }

    fn new_ioctl_error(code: i32, outcome: String) -> Self {
        return Self {
            debug_description: format!("ioctl call returned error code: {}. {}", code, outcome),
            description: format!("ioctl call returned error code: {}. {}", code, outcome),
            terminate: true,
        };
    }

    fn new_pty_spawn_error(description: String) -> Self {
        return Self {
            debug_description: format!("Failed to spawn new PTY. Reason {}", description),
            description: format!("Failed to spawn new PTY."),
            terminate: true,
        };
    }

    fn new_poll_creation_error(reason: String) -> Self {
        return Self {
            debug_description: format!("Failed to create the IO poll. Reason: {}", reason),
            description: format!("Failed to create the IO poll."),
            terminate: true,
        };
    }

    fn new_determine_terminal_size_error(reason: String) -> Self {
        return Self {
            debug_description: format!("Failed to determine terminal size. Reason: {}", reason),
            description: format!("Failed to determine terminal size."),
            terminate: true,
        };
    }

    fn new_polling_error(reason: String) -> Self {
        return Self {
            debug_description: format!("Failed to poll the IO poll. Reason: {}", reason),
            description: format!("Failed to poll the IO poll."),
            terminate: true,
        };
    }

    fn new_read_io_error(target: String, reason: String) -> Self {
        return Self {
            debug_description: format!("Failed to read from {}. Reason: {}", target, reason),
            description: format!("Failed to read from {}.", target),
            terminate: true,
        };
    }

    fn new_write_io_error(target: String, reason: String) -> Self {
        return Self {
            debug_description: format!("Failed to write to {}. Reason: {}", target, reason),
            description: format!("Failed to write to {}.", target),
            terminate: true,
        };
    }

    fn new_stdout_flush_error(reason: String) -> Self {
        return Self {
            debug_description: format!("Failed to flush stdout. Reason: {}", reason),
            description: "Failed to flush stdout".to_string(),
            terminate: true,
        };
    }

    fn new_open_pty_error(reason: String) -> Self {
        return Self {
            debug_description: format!("Failed to open pty. Reason: {}", reason),
            description: "Failed to open pty.".to_string(),
            terminate: true,
        };
    }

    fn new_fcntl_error(reason: String) -> Self {
        return Self {
            debug_description: format!("Failed fcntl call. Reason: {}", reason),
            description: "Failed fcntl call.".to_string(),
            terminate: true,
        };
    }

    fn new_display_not_running_error() -> Self {
        return Self {
            debug_description: "Display is not running".to_string(),
            description: "Display is not running".to_string(),
            terminate: true,
        };
    }

    fn new_input_manager_running_error() -> Self {
        return Self {
            debug_description: "The input manager is already running".to_string(),
            description: "The input manager is already running".to_string(),
            terminate: true,
        };
    }

    fn new_failed_tty_acquisition_error(reason: String) -> Self {
        return Self {
            debug_description: format!("Failed to acquire TTY. Reason: {}", reason),
            description: "Failed to acquire TTY.".to_string(),
            terminate: true,
        };
    }

    fn new_enter_raw_mode_error(reason: String) -> Self {
        return Self {
            debug_description: format!("Failed to enter TTY raw mode. Reason: {}", reason),
            description: "Failed to enter TTY raw mode".to_string(),
            terminate: true,
        };
    }

    fn new_no_panel_with_id(id: usize) -> Self {
        return Self {
            debug_description: format!("No panel with the id: {}", id),
            description: format!("No panel with the id: {}", id),
            terminate: true,
        };
    }

    fn new_queue_execute_error(reason: String) -> Self {
        return Self {
            debug_description: format!(
                "Failed to queue or execute display element. Reason: {}",
                reason
            ),
            description: format!(
                "Failed to queue or execute display element. Reason: {}",
                reason
            ),
            terminate: true,
        };
    }

    fn new_script_error(description: String) -> Self {
        return Self {
            debug_description: description.clone(),
            description,
            terminate: false,
        };
    }

    fn new_pty_write_error(description: String) -> Self {
        return Self {
            debug_description: description.clone(),
            description,
            terminate: true,
        };
    }

    fn new_command_error(description: String) -> Self {
        return Self {
            debug_description: description.clone(),
            description,
            terminate: false,
        };
    }

    fn new_event_parsing_error(message: String) -> Self {
        return Self {
            debug_description: format!(
                "Error occurred whilst processing a vt100 event: {}",
                message
            ),
            description: "Failed to process a terminal event.".to_string(),
            terminate: false,
        };
    }

    fn new_invalid_subdivision_state_error() -> Self {
        return Self {
            debug_description: "The subdivision is in an invalid state.".to_string(),
            description: "Failed to render due to invalid subdivision state.".to_string(),
            terminate: false,
        };
    }

    fn new_no_available_subdivision_error() -> Self {
        return Self {
            debug_description: "No empty subdivisions.".to_string(),
            description: "No empty subdivisions".to_string(),
            terminate: false,
        };
    }
}

impl std::fmt::Display for MuxideError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", self.description);
    }
}

impl std::fmt::Debug for MuxideError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", self.debug_description);
    }
}

impl Error for MuxideError {}
