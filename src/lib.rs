#![doc = include_str!("../README.md")]
#![cfg_attr(not(test), no_std)]

/// Parser type with internal states.
#[derive(Debug)]
pub struct MidiStreamParser {
    /// Buffer for message to be created.
    message: [u8; 3],

    /// Length of message in buffer.
    message_length: usize,

    /// Single byte realtime message buffer.
    realtime_message: [u8; 1],

    /// State of SysEx parsing.
    sysex_running: bool,
}

/// Parser output variants.
#[derive(Debug, PartialEq)]
pub enum ParserOutput<'a> {
    /// Regular message.
    Message(&'a [u8]),

    /// Byte of a SysEx message.
    SysexByte(u8),
}

/// Error variants.
#[derive(Debug)]
pub enum ParserError {
    /// No valid status byte.
    InvalidStatus,
}

impl Default for MidiStreamParser {
    /// Returns a new parser with default values.
    fn default() -> Self {
        Self::new()
    }
}

impl MidiStreamParser {
    /// Returns a new parser.
    pub fn new() -> Self {
        Self {
            message: [0; 3],
            message_length: 0,
            realtime_message: [0; 1],
            sysex_running: false,
        }
    }

    /// Feed a byte into the parser and return result.
    /// The `Ok` variant is an option that contains either the constructed message or `None`
    /// in case the message is not ready yet.
    pub fn parse<'a>(&'a mut self, byte: u8) -> Result<Option<ParserOutput<'a>>, ParserError> {
        match byte {
            0x00..=0x7F => {
                // Data byte
                if self.sysex_running {
                    return Ok(Some(ParserOutput::SysexByte(byte)));
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
                        return Ok(Some(ParserOutput::Message(&self.message)));
                    } else if matches!(self.message[0] & 0xF0, 0xC0 | 0xD0)
                        || matches!(self.message[0], 0xF1 | 0xF3)
                    {
                        // 2-byte message ready, keep first byte for running status
                        self.message_length = 1;
                        return Ok(Some(ParserOutput::Message(&self.message[0..2])));
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
                        return Ok(Some(ParserOutput::SysexByte(byte)));
                    }
                    0xF6 => {
                        // Tune request.
                        self.message[0] = byte;
                        self.message_length = 1;
                        return Ok(Some(ParserOutput::Message(&self.message[0..1])));
                    }
                    0xF7 => {
                        // End of SysEx.
                        self.sysex_running = false;
                        return Ok(Some(ParserOutput::SysexByte(byte)));
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
                return Ok(Some(ParserOutput::Message(&self.realtime_message)));
            }
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests;
