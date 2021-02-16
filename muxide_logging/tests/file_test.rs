mod shared;

use chrono::DateTime;
use muxide_logging::format::Format;
use muxide_logging::logger::StringLogger;
use muxide_logging::*;
use shared::*;
use std::path::Path;

#[test]
fn file_test() {
    if Path::new(TEST_FILE_NAME).exists() {
        std::fs::remove_file(TEST_FILE_NAME).unwrap();
    }

    let mut comp_logger = StringLogger::new();
    let mut comp = String::new();

    DEFAULT_LOGGER.lock().unwrap().set_override(
        Format::default()
            .set_column(0)
            .set_line(12)
            .set_constant_time(DateTime::from(
                DateTime::parse_from_rfc2822("Tue, 1 Jul 2003 10:52:37 +0000").unwrap(),
            )),
    );

    comp_logger.override_format(
        Format::default()
            .set_column(0)
            .set_line(12)
            .set_constant_time(DateTime::from(
                DateTime::parse_from_rfc2822("Tue, 1 Jul 2003 10:52:37 +0000").unwrap(),
            )),
    );

    set_output_file(TEST_FILE_NAME).unwrap();

    error!(TEST_ERROR_MESSAGE);
    comp.push_str(&error!(TEST_ERROR_MESSAGE, comp_logger).unwrap());
    comp.push('\n');

    warning!(TEST_WARNING_MESSAGE);
    comp.push_str(&warning!(TEST_WARNING_MESSAGE, comp_logger).unwrap());
    comp.push('\n');

    state_change!(TEST_STATE_CHANGE_MESSAGE);
    comp.push_str(&state_change!(TEST_STATE_CHANGE_MESSAGE, comp_logger).unwrap());
    comp.push('\n');

    info!(TEST_INFORMATION_MESSAGE);
    comp.push_str(&info!(TEST_INFORMATION_MESSAGE, comp_logger).unwrap());
    comp.push('\n');

    close_output_file().unwrap();
    let content = std::fs::read_to_string(TEST_FILE_NAME).unwrap();

    assert_eq!(content, comp);

    std::fs::remove_file(TEST_FILE_NAME).unwrap();
}
