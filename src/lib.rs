#![doc = include_str!("../README.md")]
#![no_std]

mod sysex_message;

pub use sysex_message::*;

/// A byte stream parser.
#[derive(Debug, Default)]
pub struct MidiStreamParser {
    /// Buffer for message.
    message: [u8; 3],

    /// Length of message in buffer.
    message_length: usize,

    /// Byte used for running status.
    running_status: Option<u8>,

    /// Single byte realtime message buffer.
    realtime_message: [u8; 1],

    /// State of SysEx parsing.
    sysex_running: bool,
}

/// Parser output returned by the `feed` function.
#[derive(Debug, PartialEq, Eq)]
pub enum ParserOutput<'a> {
    /// Slice of a regular message with a length of 1-3 bytes
    /// according to its type.
    Message(&'a [u8]),

    /// Single byte of a SysEx message.
    SysexByte(u8),
}

impl MidiStreamParser {
    /// Returns a new parser.
    pub fn new() -> Self {
        Self::default()
    }

    /// Feeds a byte into the parser and returns an option with the output.
    ///
    /// The option is either `Some(ParserOutput)` or `None`
    /// in case the message is not complete yet.
    pub fn feed<'a>(&'a mut self, byte: u8) -> Option<ParserOutput<'a>> {
        match byte {
            0x00..=0x7F => {
                // Data byte
                if self.sysex_running {
                    return Some(ParserOutput::SysexByte(byte));
                } else {
                    if self.message_length == 0 {
                        let status = self.running_status?;
                        self.message[0] = status;
                        self.message_length = 1;
                    }
                    self.message[self.message_length] = byte;
                    self.message_length += 1;
                    if self.message_length == 3 {
                        // 3-byte message ready, keep first byte for running status
                        self.message_length = 1;
                        return Some(ParserOutput::Message(&self.message));
                    } else if matches!(self.message[0] & 0xF0, 0xC0 | 0xD0)
                        || matches!(self.message[0], 0xF1 | 0xF3)
                    {
                        // 2-byte message ready, keep first byte for running status
                        self.message_length = 1;
                        return Some(ParserOutput::Message(&self.message[0..2]));
                    }
                }
            }
            0x80..=0xEF => {
                // Status byte for channel voice message.
                self.message[0] = byte;
                self.message_length = 1;
                self.running_status = Some(byte);
                self.sysex_running = false;
            }
            0xF0..=0xF7 => {
                // Status byte for system common message.
                match byte {
                    0xF0 => {
                        // Start of SysEx.
                        self.message[0] = 0;
                        self.message_length = 0;
                        self.sysex_running = true;
                        return Some(ParserOutput::SysexByte(byte));
                    }
                    0xF4 | 0xF5 => {
                        self.sysex_running = false;
                    }
                    0xF6 => {
                        // Tune request.
                        self.message[0] = byte;
                        self.message_length = 0;
                        self.sysex_running = false;
                        return Some(ParserOutput::Message(&self.message[0..1]));
                    }
                    0xF7 => {
                        // End of SysEx.
                        self.sysex_running = false;
                        self.running_status = None;
                        return Some(ParserOutput::SysexByte(byte));
                    }
                    _ => {
                        self.message[0] = byte;
                        self.message_length = 1;
                        self.sysex_running = false;
                    }
                }
            }
            0xF8..=0xFF => {
                // Status byte for system realtime message.
                self.realtime_message[0] = byte;
                return Some(ParserOutput::Message(&self.realtime_message));
            }
        }

        None
    }

    /// Resets the parser to its initial state.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Note off message.
    #[test]
    fn note_off() {
        let mut parser = MidiStreamParser::new();

        let bytes = [0x82, 37, 10];
        let messages = [None, None, Some(ParserOutput::Message(&[0x82, 37, 10]))];

        for (byte, message) in bytes.iter().zip(messages.iter()) {
            let result = parser.feed(*byte);
            assert_eq!(result, *message);
        }
    }

    /// Incomplete note off message followed by a note on message.
    #[test]
    fn note_off_incomplete_note_on() {
        let mut parser = MidiStreamParser::new();

        let bytes = [0x82, 37, 0x93, 14, 80];
        let messages = [
            None,
            None,
            None,
            None,
            Some(ParserOutput::Message(&[0x93, 14, 80])),
        ];

        for (byte, message) in bytes.iter().zip(messages.iter()) {
            let result = parser.feed(*byte);
            assert_eq!(result, *message);
        }
    }

    /// Note off message with realtime messages in-between.
    #[test]
    fn note_off_with_realtime() {
        let mut parser = MidiStreamParser::new();

        let bytes = [0x82, 0xFA, 37, 0xF8, 10];
        let messages = [
            None,
            Some(ParserOutput::Message(&[0xFA])),
            None,
            Some(ParserOutput::Message(&[0xF8])),
            Some(ParserOutput::Message(&[0x82, 37, 10])),
        ];

        for (byte, message) in bytes.iter().zip(messages.iter()) {
            let result = parser.feed(*byte);
            assert_eq!(result, *message);
        }
    }

    /// Note on message.
    #[test]
    fn note_on() {
        let mut parser = MidiStreamParser::new();

        let bytes = [0x94, 75, 82];
        let messages = [None, None, Some(ParserOutput::Message(&[0x94, 75, 82]))];

        for (byte, message) in bytes.iter().zip(messages.iter()) {
            let result = parser.feed(*byte);
            assert_eq!(result, *message);
        }
    }

    /// Two note on messages sharing the same status byte.
    #[test]
    fn note_on_running_status() {
        let mut parser = MidiStreamParser::new();

        let bytes = [0x90, 60, 127, 61, 40];
        let messages = [
            None,
            None,
            Some(ParserOutput::Message(&[0x90, 60, 127])),
            None,
            Some(ParserOutput::Message(&[0x90, 61, 40])),
        ];

        for (byte, message) in bytes.iter().zip(messages.iter()) {
            let result = parser.feed(*byte);
            assert_eq!(result, *message);
        }
    }

    /// Two note on messages sharing the same status byte
    /// with a realtime message in-between.
    #[test]
    fn note_on_running_status_with_realtime() {
        let mut parser = MidiStreamParser::new();

        let bytes = [0x90, 60, 127, 61, 0xF8, 40];
        let messages = [
            None,
            None,
            Some(ParserOutput::Message(&[0x90, 60, 127])),
            None,
            Some(ParserOutput::Message(&[0xF8])),
            Some(ParserOutput::Message(&[0x90, 61, 40])),
        ];

        for (byte, message) in bytes.iter().zip(messages.iter()) {
            let result = parser.feed(*byte);
            assert_eq!(result, *message);
        }
    }

    /// Two note on messages sharing the same status byte
    /// with a tune request message in-between.
    #[test]
    fn note_on_running_status_with_tune_request() {
        let mut parser = MidiStreamParser::new();

        let bytes = [0x90, 60, 127, 0xF6, 61, 40];
        let messages = [
            None,
            None,
            Some(ParserOutput::Message(&[0x90, 60, 127])),
            Some(ParserOutput::Message(&[0xF6])),
            None,
            Some(ParserOutput::Message(&[0x90, 61, 40])),
        ];

        for (byte, message) in bytes.iter().zip(messages.iter()) {
            let result = parser.feed(*byte);
            assert_eq!(result, *message);
        }
    }

    /// Two note on messages sharing the same status byte
    /// with an invalid message in-between.
    #[test]
    fn note_on_running_status_with_invalid() {
        let mut parser = MidiStreamParser::new();

        let bytes = [0x90, 60, 127, 0xF4, 61, 40];
        let messages = [
            None,
            None,
            Some(ParserOutput::Message(&[0x90, 60, 127])),
            None,
            None,
            Some(ParserOutput::Message(&[0x90, 61, 40])),
        ];

        for (byte, message) in bytes.iter().zip(messages.iter()) {
            let result = parser.feed(*byte);
            assert_eq!(result, *message);
        }
    }

    /// Poly key pressure message.
    #[test]
    fn poly_key_pressure() {
        let mut parser = MidiStreamParser::new();

        let bytes = [0xA1, 52, 80];
        let messages = [None, None, Some(ParserOutput::Message(&[0xA1, 52, 80]))];

        for (byte, message) in bytes.iter().zip(messages.iter()) {
            let result = parser.feed(*byte);
            assert_eq!(result, *message);
        }
    }

    /// Control change message.
    #[test]
    fn control_change() {
        let mut parser = MidiStreamParser::new();

        let bytes = [0xBC, 21, 40];
        let messages = [None, None, Some(ParserOutput::Message(&[0xBC, 21, 40]))];

        for (byte, message) in bytes.iter().zip(messages.iter()) {
            let result = parser.feed(*byte);
            assert_eq!(result, *message);
        }
    }

    /// Program change message.
    #[test]
    fn program_change() {
        let mut parser = MidiStreamParser::new();

        let bytes = [0xC5, 17];
        let messages = [None, Some(ParserOutput::Message(&[0xC5, 17]))];

        for (byte, message) in bytes.iter().zip(messages.iter()) {
            let result = parser.feed(*byte);
            assert_eq!(result, *message);
        }
    }

    /// Channel pressure message.
    #[test]
    fn channel_pressure() {
        let mut parser = MidiStreamParser::new();

        let bytes = [0xD7, 102];
        let messages = [None, Some(ParserOutput::Message(&[0xD7, 102]))];

        for (byte, message) in bytes.iter().zip(messages.iter()) {
            let result = parser.feed(*byte);
            assert_eq!(result, *message);
        }
    }

    /// Pitch bend message.
    #[test]
    fn pitch_bend() {
        let mut parser = MidiStreamParser::new();

        let bytes = [0xE9, 94, 112];
        let messages = [None, None, Some(ParserOutput::Message(&[0xE9, 94, 112]))];

        for (byte, message) in bytes.iter().zip(messages.iter()) {
            let result = parser.feed(*byte);
            assert_eq!(result, *message);
        }
    }

    /// MTC quarter frame message.
    #[test]
    fn mtc_quarter_frame() {
        let mut parser = MidiStreamParser::new();

        let bytes = [0xF1, 54];
        let messages = [None, Some(ParserOutput::Message(&[0xF1, 54]))];

        for (byte, message) in bytes.iter().zip(messages.iter()) {
            let result = parser.feed(*byte);
            assert_eq!(result, *message);
        }
    }

    /// Song position pointer message.
    #[test]
    fn song_position_pointer() {
        let mut parser = MidiStreamParser::new();

        let bytes = [0xF2, 19, 78];
        let messages = [None, None, Some(ParserOutput::Message(&[0xF2, 19, 78]))];

        for (byte, message) in bytes.iter().zip(messages.iter()) {
            let result = parser.feed(*byte);
            assert_eq!(result, *message);
        }
    }

    /// Song select message.
    #[test]
    fn song_select() {
        let mut parser = MidiStreamParser::new();

        let bytes = [0xF3, 12];
        let messages = [None, Some(ParserOutput::Message(&[0xF3, 12]))];

        for (byte, message) in bytes.iter().zip(messages.iter()) {
            let result = parser.feed(*byte);
            assert_eq!(result, *message);
        }
    }

    /// Undefined messages.
    #[test]
    fn undefined() {
        let mut parser = MidiStreamParser::new();

        let bytes = [0xF4, 0xF5];
        let messages = [None, None];

        for (byte, message) in bytes.iter().zip(messages.iter()) {
            let result = parser.feed(*byte);
            assert_eq!(result, *message);
        }
    }

    /// Tune request message.
    #[test]
    fn tune_request() {
        let mut parser = MidiStreamParser::new();

        let bytes = [0xF6];
        let messages = [Some(ParserOutput::Message(&[0xF6]))];

        for (byte, message) in bytes.iter().zip(messages.iter()) {
            let result = parser.feed(*byte);
            assert_eq!(result, *message);
        }
    }

    /// Multiple system realtime messages.
    #[test]
    fn system_realtime() {
        let mut parser = MidiStreamParser::new();

        let bytes = [0xF8, 0xFA, 0xFB, 0xFC, 0xFE, 0xFF];
        let messages = [
            Some(ParserOutput::Message(&[0xF8])),
            Some(ParserOutput::Message(&[0xFA])),
            Some(ParserOutput::Message(&[0xFB])),
            Some(ParserOutput::Message(&[0xFC])),
            Some(ParserOutput::Message(&[0xFE])),
            Some(ParserOutput::Message(&[0xFF])),
        ];

        for (byte, message) in bytes.iter().zip(messages.iter()) {
            let result = parser.feed(*byte);
            assert_eq!(result, *message);
        }
    }

    /// SysEx message without anything special.
    #[test]
    fn sysex() {
        let mut parser = MidiStreamParser::new();

        let bytes = [0xF0, 0x10, 0x20, 0x7F, 0x30, 0xF7];
        let messages = [
            Some(ParserOutput::SysexByte(0xF0)),
            Some(ParserOutput::SysexByte(0x10)),
            Some(ParserOutput::SysexByte(0x20)),
            Some(ParserOutput::SysexByte(0x7F)),
            Some(ParserOutput::SysexByte(0x30)),
            Some(ParserOutput::SysexByte(0xF7)),
        ];

        for (byte, message) in bytes.iter().zip(messages.iter()) {
            let result = parser.feed(*byte);
            assert_eq!(result, *message);
        }
    }

    /// SysEx message with a clock message in-between.
    #[test]
    fn sysex_with_realtime() {
        let mut parser = MidiStreamParser::new();

        let bytes = [0xF0, 0x10, 0xF8, 0x20, 0x7F, 0x30, 0xF7];
        let messages = [
            Some(ParserOutput::SysexByte(0xF0)),
            Some(ParserOutput::SysexByte(0x10)),
            Some(ParserOutput::Message(&[0xF8])),
            Some(ParserOutput::SysexByte(0x20)),
            Some(ParserOutput::SysexByte(0x7F)),
            Some(ParserOutput::SysexByte(0x30)),
            Some(ParserOutput::SysexByte(0xF7)),
        ];

        for (byte, message) in bytes.iter().zip(messages.iter()) {
            let result = parser.feed(*byte);
            assert_eq!(result, *message);
        }
    }

    /// Incomplete SysEx message followed by a note on message.
    #[test]
    fn sysex_incomplete_note_on() {
        let mut parser = MidiStreamParser::new();

        let bytes = [0xF0, 0x10, 0x20, 0x97, 23, 98];
        let messages = [
            Some(ParserOutput::SysexByte(0xF0)),
            Some(ParserOutput::SysexByte(0x10)),
            Some(ParserOutput::SysexByte(0x20)),
            None,
            None,
            Some(ParserOutput::Message(&[0x97, 23, 98])),
        ];

        for (byte, message) in bytes.iter().zip(messages.iter()) {
            let result = parser.feed(*byte);
            assert_eq!(result, *message);
        }
    }
}
