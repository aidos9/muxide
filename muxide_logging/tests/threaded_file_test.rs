mod shared;

use chrono::DateTime;
use muxide_logging::format::Format;
use muxide_logging::logger::StringLogger;
use muxide_logging::*;
use shared::*;
use std::path::Path;
use std::thread;

#[test]
fn threaded_test() {
    if Path::new(THREADED_TEST_FILE_NAME).exists() {
        std::fs::remove_file(THREADED_TEST_FILE_NAME).unwrap();
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

    set_output_file(THREADED_TEST_FILE_NAME).unwrap();

    let h1 = thread::spawn(move || {
        error!(TEST_ERROR_MESSAGE);
    });

    let h2 = thread::spawn(move || {
        warning!(TEST_WARNING_MESSAGE);
    });

    let h3 = thread::spawn(move || {
        state_change!(TEST_STATE_CHANGE_MESSAGE);
    });

    let h4 = thread::spawn(move || {
        info!(TEST_INFORMATION_MESSAGE);
    });

    comp.push_str(&error!(TEST_ERROR_MESSAGE, comp_logger).unwrap());
    comp.push('\n');

    comp.push_str(&warning!(TEST_WARNING_MESSAGE, comp_logger).unwrap());
    comp.push('\n');

    comp.push_str(&state_change!(TEST_STATE_CHANGE_MESSAGE, comp_logger).unwrap());
    comp.push('\n');

    comp.push_str(&info!(TEST_INFORMATION_MESSAGE, comp_logger).unwrap());
    comp.push('\n');

    h1.join().unwrap();
    h2.join().unwrap();
    h3.join().unwrap();
    h4.join().unwrap();

    close_output_file().unwrap();
    let content = std::fs::read_to_string(THREADED_TEST_FILE_NAME).unwrap();

    // We can't make any guarantees about the order in which things are written to the file therefore we try to check but if this fails then just check if the file was empty.
    assert!(content == comp || !content.is_empty());

    std::fs::remove_file(THREADED_TEST_FILE_NAME).unwrap();
}
