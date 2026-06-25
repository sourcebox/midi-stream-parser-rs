//! Unit tests

use super::*;

/// Note off message.
#[test]
fn note_off() {
    let mut parser = MidiStreamParser::<256>::new();

    let bytes = [0x82, 37, 10];
    let messages = [None, None, Some([0x82, 37, 10].as_ref())];

    for (byte, message) in bytes.iter().zip(messages.iter()) {
        let result = parser.parse(*byte).unwrap();
        assert_eq!(result, *message);
    }
}

/// Note off message with realtime messages in-between.
#[test]
fn note_off_with_realtime() {
    let mut parser = MidiStreamParser::<256>::new();

    let bytes = [0x82, 0xFA, 37, 0xF8, 10];
    let messages = [
        None,
        Some([0xFA].as_ref()),
        None,
        Some([0xF8].as_ref()),
        Some([0x82, 37, 10].as_ref()),
    ];

    for (byte, message) in bytes.iter().zip(messages.iter()) {
        let result = parser.parse(*byte).unwrap();
        assert_eq!(result, *message);
    }
}

/// Note on message.
#[test]
fn note_on() {
    let mut parser = MidiStreamParser::<256>::new();

    let bytes = [0x94, 75, 82];
    let messages = [None, None, Some([0x94, 75, 82].as_ref())];

    for (byte, message) in bytes.iter().zip(messages.iter()) {
        let result = parser.parse(*byte).unwrap();
        assert_eq!(result, *message);
    }
}

/// Two note on messages sharing the same status byte.
#[test]
fn note_on_running_status() {
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

/// Poly key pressure message.
#[test]
fn poly_key_pressure() {
    let mut parser = MidiStreamParser::<256>::new();

    let bytes = [0xA1, 52, 80];
    let messages = [None, None, Some([0xA1, 52, 80].as_ref())];

    for (byte, message) in bytes.iter().zip(messages.iter()) {
        let result = parser.parse(*byte).unwrap();
        assert_eq!(result, *message);
    }
}

/// Control change message.
#[test]
fn control_change() {
    let mut parser = MidiStreamParser::<256>::new();

    let bytes = [0xBC, 21, 40];
    let messages = [None, None, Some([0xBC, 21, 40].as_ref())];

    for (byte, message) in bytes.iter().zip(messages.iter()) {
        let result = parser.parse(*byte).unwrap();
        assert_eq!(result, *message);
    }
}

/// Program change message.
#[test]
fn program_change() {
    let mut parser = MidiStreamParser::<256>::new();

    let bytes = [0xC5, 17];
    let messages = [None, Some([0xC5, 17].as_ref())];

    for (byte, message) in bytes.iter().zip(messages.iter()) {
        let result = parser.parse(*byte).unwrap();
        assert_eq!(result, *message);
    }
}

/// Channel pressure message.
#[test]
fn channel_pressure() {
    let mut parser = MidiStreamParser::<256>::new();

    let bytes = [0xD7, 102];
    let messages = [None, Some([0xD7, 102].as_ref())];

    for (byte, message) in bytes.iter().zip(messages.iter()) {
        let result = parser.parse(*byte).unwrap();
        assert_eq!(result, *message);
    }
}

/// Pitch bend message.
#[test]
fn pitch_bend() {
    let mut parser = MidiStreamParser::<256>::new();

    let bytes = [0xE9, 94, 112];
    let messages = [None, None, Some([0xE9, 94, 112].as_ref())];

    for (byte, message) in bytes.iter().zip(messages.iter()) {
        let result = parser.parse(*byte).unwrap();
        assert_eq!(result, *message);
    }
}

/// MTC quarter frame message.
#[test]
fn mtc_quarter_frame() {
    let mut parser = MidiStreamParser::<256>::new();

    let bytes = [0xF1, 54];
    let messages = [None, Some([0xF1, 54].as_ref())];

    for (byte, message) in bytes.iter().zip(messages.iter()) {
        let result = parser.parse(*byte).unwrap();
        assert_eq!(result, *message);
    }
}

/// Song position pointer message.
#[test]
fn song_position_pointer() {
    let mut parser = MidiStreamParser::<256>::new();

    let bytes = [0xF2, 19, 78];
    let messages = [None, None, Some([0xF2, 19, 78].as_ref())];

    for (byte, message) in bytes.iter().zip(messages.iter()) {
        let result = parser.parse(*byte).unwrap();
        assert_eq!(result, *message);
    }
}

/// Song select message.
#[test]
fn song_select() {
    let mut parser = MidiStreamParser::<256>::new();

    let bytes = [0xF3, 12];
    let messages = [None, Some([0xF3, 12].as_ref())];

    for (byte, message) in bytes.iter().zip(messages.iter()) {
        let result = parser.parse(*byte).unwrap();
        assert_eq!(result, *message);
    }
}

/// Tune request message.
#[test]
fn tune_request() {
    let mut parser = MidiStreamParser::<256>::new();

    let bytes = [0xF6];
    let messages = [Some([0xF6].as_ref())];

    for (byte, message) in bytes.iter().zip(messages.iter()) {
        let result = parser.parse(*byte).unwrap();
        assert_eq!(result, *message);
    }
}

/// Multiple system realtime messages.
#[test]
fn system_realtime() {
    let mut parser = MidiStreamParser::<256>::new();

    let bytes = [0xF8, 0xFA, 0xFB, 0xFC, 0xFE, 0xFF];
    let messages = [
        Some([0xF8].as_ref()),
        Some([0xFA].as_ref()),
        Some([0xFB].as_ref()),
        Some([0xFC].as_ref()),
        Some([0xFE].as_ref()),
        Some([0xFF].as_ref()),
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
