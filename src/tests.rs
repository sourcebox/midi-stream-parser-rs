//! Unit tests

use super::*;

/// Two note on messages sharing the same status byte.
#[test]
fn running_status() {
    let mut parser = MidiStreamParser::<256>::new();

    let bytes = [0x90, 60, 127, 61, 40];
    let messages = [
        None,
        None,
        Some([0x90, 60, 127].as_ref()),
        None,
        Some([0x90, 61, 40].as_ref()),
    ];

    for (byte, message) in bytes.iter().zip(messages.iter()) {
        let result = parser.parse(*byte).unwrap();
        assert_eq!(result, *message);
    }
}

/// SysEx message without anything special.
#[test]
fn sysex() {
    let mut parser = MidiStreamParser::<256>::new();

    let bytes = [0xF0, 0x10, 0x20, 0x7F, 0x30, 0xF7];
    let messages = [
        None,
        None,
        None,
        None,
        None,
        Some([0xF0, 0x10, 0x20, 0x7F, 0x30, 0xF7].as_ref()),
    ];

    for (byte, message) in bytes.iter().zip(messages.iter()) {
        let result = parser.parse(*byte).unwrap();
        assert_eq!(result, *message);
    }
}

/// SysEx message with a clock message in-between.
#[test]
fn sysex_with_realtime() {
    let mut parser = MidiStreamParser::<256>::new();

    let bytes = [0xF0, 0x10, 0xF8, 0x20, 0x7F, 0x30, 0xF7];
    let messages = [
        None,
        None,
        Some([0xF8].as_ref()),
        None,
        None,
        None,
        Some([0xF0, 0x10, 0x20, 0x7F, 0x30, 0xF7].as_ref()),
    ];

    for (byte, message) in bytes.iter().zip(messages.iter()) {
        let result = parser.parse(*byte).unwrap();
        assert_eq!(result, *message);
    }
}

/// SysEx message with more bytes than parser can buffer,
/// followed by a shorter one that can be processed.
#[test]
fn sysex_overflow() {
    let mut parser = MidiStreamParser::<4>::new();

    let bytes = [0xF0, 0x01, 0x02, 0x03, 0x04, 0xF7, 0xF0, 0x11, 0x12, 0xF7];
    let messages = [
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some([0xF0, 0x11, 0x12, 0xF7].as_ref()),
    ];

    for (byte, message) in bytes.iter().zip(messages.iter()) {
        let result = parser.parse(*byte);
        match result {
            Ok(result) => {
                assert_eq!(result, *message);
            }
            Err(result) => {
                assert!(matches!(result, ParserError::SysexOverflow));
            }
        }
    }
}
