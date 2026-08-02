#![doc = include_str!("../README.md")]
#![no_std]

/// A byte stream parser.
#[derive(Debug, Default)]
pub struct MidiStreamParser {
    /// Buffer for message.
    message: [u8; 3],

    /// Length of message in buffer.
    message_length: usize,

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
                        // No valid status byte found.
                        return None;
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
                    0xF6 => {
                        // Tune request.
                        self.message[0] = byte;
                        self.message_length = 1;
                        return Some(ParserOutput::Message(&self.message[0..1]));
                    }
                    0xF7 => {
                        // End of SysEx.
                        self.sysex_running = false;
                        return Some(ParserOutput::SysexByte(byte));
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

/// A MIDI SysEx message with internal buffer.
#[derive(Debug)]
pub struct MidiSysexMessage<const CAPACITY: usize> {
    /// Buffer containing the message.
    buffer: [u8; CAPACITY],

    /// Length of the message in the buffer.
    length: usize,

    /// Flag for running message.
    running: bool,
}

impl<const CAPACITY: usize> Default for MidiSysexMessage<CAPACITY> {
    /// Returns a new parser with default values.
    fn default() -> Self {
        Self {
            buffer: [0; CAPACITY],
            length: 0,
            running: false,
        }
    }
}

impl<const CAPACITY: usize> MidiSysexMessage<CAPACITY> {
    /// Returns a new SysEx message.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends a byte to the buffer and returns an option
    /// with a `Some(&[u8])` when the message is complete.
    pub fn append(&mut self, byte: u8) -> Option<&[u8]> {
        match byte {
            0xF0 => {
                // Start of message.
                self.buffer[0] = byte;
                self.length = 1;
                self.running = true;
                None
            }
            0xF7 if self.running && self.length < CAPACITY => {
                // End of message.
                self.buffer[self.length] = byte;
                self.length += 1;
                self.running = false;
                Some(&self.buffer[..self.length])
            }
            0x00..=0x7F if self.running && self.length < CAPACITY => {
                // Data byte.
                self.buffer[self.length] = byte;
                self.length += 1;
                None
            }
            _ => None,
        }
    }

    /// Clears the data in the buffer.
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// Returns the capacity of the buffer.
    pub fn capacity(&self) -> usize {
        CAPACITY
    }

    /// Returns the length of the data in the buffer.
    pub fn len(&self) -> usize {
        self.length
    }

    /// Returns the free capacity of the buffer.
    pub fn free_capacity(&self) -> usize {
        CAPACITY - self.len()
    }

    /// Returns if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    /// Returns if the buffer is full.
    pub fn is_full(&self) -> bool {
        self.length == CAPACITY
    }
}

#[cfg(test)]
mod tests;
