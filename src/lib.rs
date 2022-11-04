//! Parser to convert a stream of MIDI bytes into well-formed messages.

#![doc = include_str!("../README.md")]
#![no_std]

/// Parser type with internal states.
/// Owns a buffer of `SYSEX_MAX_LEN` bytes for constructing SysEx messages.
#[derive(Debug)]
pub struct MidiStreamParser<const SYSEX_MAX_LEN: usize> {
    /// Buffer for message to be created.
    message: [u8; 3],

    /// Length of message in buffer.
    message_length: usize,

    /// Single byte realtime message buffer.
    realtime_message: [u8; 1],

    /// State of SysEx parsing.
    sysex_running: bool,

    /// SysEx message buffer.
    sysex_message: [u8; SYSEX_MAX_LEN],

    /// SysEx message length.
    sysex_message_length: usize,
}

/// Error variants.
#[derive(Debug)]
pub enum ParserError {
    /// No valid status byte.
    InvalidStatus,

    /// SysEx message longer than SYSEX_MAX_LEN bytes.
    SysexOverflow,
}

impl<const SYSEX_MAX_LEN: usize> Default for MidiStreamParser<SYSEX_MAX_LEN> {
    /// Returns a new parser with default values.
    fn default() -> Self {
        Self::new()
    }
}

impl<const SYSEX_MAX_LEN: usize> MidiStreamParser<SYSEX_MAX_LEN> {
    /// Returns a new parser.
    pub fn new() -> Self {
        Self {
            message: [0; 3],
            message_length: 0,
            realtime_message: [0; 1],
            sysex_running: false,
            sysex_message: [0; SYSEX_MAX_LEN],
            sysex_message_length: 0,
        }
    }

    /// Feed a byte into the parser and return result.
    /// The `Ok` variant is an option that contains either the constructed message or `None`
    /// in case the message is not ready yet.
    pub fn parse(&mut self, byte: u8) -> Result<Option<&[u8]>, ParserError> {
        match byte {
            0x00..=0x7F => {
                // Data byte
                if self.sysex_running {
                    if self.sysex_message_length >= SYSEX_MAX_LEN {
                        return Err(ParserError::SysexOverflow);
                    }
                    self.sysex_message[self.sysex_message_length] = byte;
                    self.sysex_message_length += 1;
                } else {
                    if self.message_length == 0 {
                        // No valid status byte found.
                        return Err(ParserError::InvalidStatus);
                    }
                    self.message[self.message_length] = byte;
                    self.message_length += 1;
                    if self.message_length == 3 {
                        // 3-byte message ready, keep first byte for running status
                        self.message_length = 1;
                        return Ok(Some(&self.message));
                    } else if matches!(self.message[0] & 0xF0, 0xC0 | 0xD0)
                        || matches!(self.message[0], 0xF1 | 0xF3)
                    {
                        // 2-byte message ready, keep first byte for running status
                        self.message_length = 1;
                        return Ok(Some(&self.message[0..2]));
                    }
                }
            }
            0x80..=0xEF => {
                // Status byte for channel voice message.
                self.message[0] = byte;
                self.message_length = 1;
            }
            0xF0..=0xF7 => {
                // Status byte for system common message.
                match byte {
                    0xF0 => {
                        // Start of SysEx.
                        self.message[0] = 0;
                        self.message_length = 0;
                        self.sysex_running = true;
                        self.sysex_message[0] = byte;
                        self.sysex_message_length = 1;
                    }
                    0xF7 => {
                        // End of SysEx.
                        self.sysex_running = false;
                        if self.sysex_message_length >= SYSEX_MAX_LEN {
                            return Err(ParserError::SysexOverflow);
                        }
                        self.sysex_message[self.sysex_message_length] = byte;
                        self.sysex_message_length += 1;
                        return Ok(Some(&self.sysex_message[0..self.sysex_message_length]));
                    }
                    _ => {
                        self.message[0] = byte;
                        self.message_length = 1;
                    }
                }
            }
            0xF8..=0xFF => {
                // Status byte for system realtime message.
                self.realtime_message[0] = byte;
                return Ok(Some(&self.realtime_message));
            }
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
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
}
