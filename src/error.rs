use paste::paste;
use std::error::Error;

macro_rules! define_error_type {
    ($($name:ident [$snake_name:ident] ($debug_description:literal $(($($debug_arg:ident),*))?, $(#$description:literal $(($($arg:ident),*))?,)? $terminate:literal) {$($field_name:ident: $field_type:ty),*};)*) => {
        #[derive(Clone, PartialEq, Debug, Hash)]
        pub enum ErrorType {
            $($name {
                $(
                    $field_name : $field_type
                ),*
            }),*
        }

        impl ErrorType {
            $(
                 paste! {
                    pub fn [<new_$snake_name>] ($($field_name : $field_type),*) -> Self {
                        return Self::$name {
                            $($field_name),*
                        };
                    }

                    pub fn [<new_$snake_name _error>]($($field_name : $field_type),*) -> MuxideError {
                        return Self::[<new_$snake_name>]($($field_name),*).into_error();
                    }
                }
            )*

            pub fn into_error(self) -> MuxideError {
                return match self {
                    $(
                        Self::$name { $($field_name),* } => {
                            #[allow(unused_variables)]
                            let description = format!($debug_description, $($($debug_arg),*)?);
                            $(let description = format!($description, $($($arg),*)?);)?

                            MuxideError {
                                debug_description: format!($debug_description, $($($debug_arg),*)?),
                                description,
                                terminate: $terminate,
                            }
                        }
                    ),*
                };
            }
        }
    };
}

define_error_type!(
    IOCTLError [ioctl] ("ioctl call returned error code: {}. {}" (code, outcome), #"ioctl call returned error code: {}. {}" (code, outcome), true) {
        code: i32,
        outcome: String
    };

    PTYSpawnError [pty_spawn] ("Failed to spawn new PTY. Reason {}" (description), #"Failed to spawn new PTY.", true) {
        description: String
    };

    PollCreationError [poll_creation] ("Failed to create the IO poll. Reason: {}" (reason), #"Failed to create the IO poll", true) {
        reason: String
    };

    DetermineTerminalSizeError [determine_terminal_size] ("Failed to determine terminal size. Reason: {}" (reason), #"Failed to determine terminal size.", true) {
        reason: String
    };

    PollingError [polling] ("Failed to poll the IO poll. Reason: {}" (reason), #"Failed to poll the IO poll.", true) {
        reason: String
    };

    ReadIOError [read_io] ("Failed to read from {}. Reason: {}" (target, reason), #"Failed to read from {}" (target), true) {
        target: String,
        reason: String
    };

    WriteIOError [write_io] ("Failed to write to {}. Reason: {}" (target, reason), #"Failed to write to {}" (target), true) {
        target: String,
        reason: String
    };

    StdoutFlushError [stdout_flush] ("Failed to flush stdout. Reason: {}" (reason), #"Failed to flush stdout", true) {
        reason: String
    };

    OpenPTYError [open_pty] ("Failed to open pty. Reason: {}" (reason), #"Failed to open pty", true) {
        reason: String
    };

    FCNTLError [fcntl] ("Failed fcntl call. Reason: {}" (reason), #"Failed fcntl call.", true) {
        reason: String
    };

    FailedTTYAcquisitionError [failed_tty_acquisition] ("Failed to acquire TTY. Reason: {}" (reason), #"Failed to acquire TTY.", true) {
        reason: String
    };

    EnterRawModeError [enter_raw_mode] ("Failed to enter TTY raw mode. Reason: {}" (reason), #"Failed to enter TTY raw mode.", true) {
        reason: String
    };

    NoPanelWithIDError [no_panel_with_id] ("No panel with the id: {}" (id), #"No panel with the id: {}" (id), false) {
        id: usize
    };

    QueueExecuteError [queue_execute] ("Failed to queue or execute display element. Reason: {}" (reason), #"Failed to queue or execute display element.", true) {
        reason: String
    };

    PTYWriteError [pty_write] ("Failed to write to PTY. Description: {}" (description), #"Failed to write to PTY.", true) {
        description: String
    };

    CommandError[command]  ("{}" (description), false) {
        description: String
    };

    EventParsingError [event_parsing] ("Error occurred whilst processing a vt100 event: {}" (message), #"Failed to process a terminal event." ,false) {
        message: String
    };

    DisplayNotRunningError [display_not_running] ("Display is not running", true) { };
    InputManagerRunningError [input_manager_running] ("The input manager is already running", true) { };
    InvalidSubdivisionStateError[invalid_subdivision_state]  ("The subdivision is in an invalid state.", #"Failed to render due to invalid subdivision state.", true) { };
    NoAvailableSubdivisionError [no_available_subdivision] ("No empty subdivisions.", false) { };
    FailedSubdivisionError [failed_subdivision] ("Failed to subdivide panel.", false) { };
    PtyStdinReceiverClosedError [pty_stdin_receiver_closed] ("The pty's stdin receiver closed.", true) { };
    FailedReadPollError [failed_read_poll] ("Failed to poll the pty for data.", false) { };
    FailedToSendMessageError [failed_to_send_message] ("Failed to send message from pty thread.", false) { };
    FailedToReadPTYError [failed_to_read_pty] ("Failed to read data from pty.", false) { };
    PTYStoppedRunningError [pty_stopped_running] ("PTY unexpectedly stopped running.", false) { };
    NoWorkspaceWithIDError [no_workspace_with_id] ("No workspace with id: {}" (id), false) { id: usize };
    DisplayLockedError [display_locked] ("Display is locked.", false) { };
    InvalidPasswordError [invalid_password] ("Incorrect Password.", false){ };
    FailedToCheckPasswordError [failed_to_check_password] ("Hash comparison failed.", #"Failed to compare password.", true) { };
    NoAvailableSubdivisionToMergeError [no_available_subdivision_to_merge] ("No open subdivision to merge.", false) { };
    NoSubdivisionAtPathError [no_subdivision_at_path] ("No subdivision at path.", true) { };
    NoPanelAtPathError [no_panel_at_path] ("No panel at path end.", true) { };
);

#[derive(Clone, PartialEq, Hash)]
pub struct MuxideError {
    debug_description: String,
    description: String,
    terminate: bool,
}

impl ErrorType {
    pub fn new_display_qe_error(io_error: std::io::Error) -> MuxideError {
        return Self::QueueExecuteError {
            reason: io_error.to_string(),
        }
        .into_error();
    }
}

impl MuxideError {
    pub fn from_error_type(tp: ErrorType) -> Self {
        return tp.into_error();
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
