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

    DisplayNotRunningError,
    InputManagerRunningError,
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
