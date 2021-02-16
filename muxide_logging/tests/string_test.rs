mod shared;

use chrono::DateTime;
use muxide_logging::format::Format;
use muxide_logging::logger::StringLogger;
use muxide_logging::*;
use shared::*;

pub fn create_string_logger() -> StringLogger {
    let mut logger = StringLogger::new();

    logger.override_format(Format::default().set_constant_time(DateTime::from(
        DateTime::parse_from_rfc2822("Tue, 1 Jul 2003 10:52:37 +0000").unwrap(),
    )));

    return logger;
}

#[test]
fn test_error_macro() {
    let res = error!(TEST_ERROR_MESSAGE, create_string_logger()).unwrap();

    assert_eq!(
        res,
        format!(
            "[20:52:37] ({} {}:15) Error: {}",
            module_path!(),
            line!() - 7,
            TEST_ERROR_MESSAGE,
        )
    );
}

#[test]
fn test_warning_macro() {
    let res = warning!(TEST_WARNING_MESSAGE, create_string_logger()).unwrap();

    assert_eq!(
        res,
        format!(
            "[20:52:37] ({} {}:15) Warning: {}",
            module_path!(),
            line!() - 7,
            TEST_WARNING_MESSAGE,
        )
    );
}

#[test]
fn test_state_change_macro() {
    let res = state_change!(TEST_STATE_CHANGE_MESSAGE, create_string_logger()).unwrap();

    assert_eq!(
        res,
        format!(
            "[20:52:37] ({} {}:15) StateChange: {}",
            module_path!(),
            line!() - 7,
            TEST_STATE_CHANGE_MESSAGE,
        )
    );
}

#[test]
fn test_info_macro() {
    let res = info!(TEST_INFORMATION_MESSAGE, create_string_logger()).unwrap();

    assert_eq!(
        res,
        format!(
            "[20:52:37] ({} {}:15) Information: {}",
            module_path!(),
            line!() - 7,
            TEST_INFORMATION_MESSAGE,
        )
    );
}
