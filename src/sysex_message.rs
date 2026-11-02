//! SysEx message helper.

/// A MIDI SysEx message with internal buffer.
#[derive(Debug, Clone, PartialEq, Eq)]
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
    pub fn append(&mut self, byte: u8) -> Result<Option<&[u8]>, MidiSysexMessageError> {
        if self.length >= CAPACITY {
            return Err(MidiSysexMessageError::CapacityExceeded);
        }

        match byte {
            0xF0 => {
                // Start of message.
                self.buffer[0] = byte;
                self.length = 1;
                self.running = true;
                Ok(None)
            }
            0xF7 if self.running => {
                // End of message.
                self.buffer[self.length] = byte;
                self.length += 1;
                self.running = false;
                Ok(Some(&self.buffer[..self.length]))
            }
            0x00..=0x7F if self.running => {
                // Data byte.
                self.buffer[self.length] = byte;
                self.length += 1;
                Ok(None)
            }
            0x00..=0x7F | 0xF7 if !self.running => Err(MidiSysexMessageError::NoStartByte),
            _ => Err(MidiSysexMessageError::InvalidData),
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

/// Errors for `MidiSysexMessage`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum MidiSysexMessageError {
    /// Message has not been started with 0xF0.
    NoStartByte,

    /// Invalid data byte.
    InvalidData,

    /// Capacity of the message is exceeded.
    CapacityExceeded,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Message without anything special.
    #[test]
    fn sysex_message() {
        let mut sysex_message = MidiSysexMessage::<256>::new();

        let bytes = [0xF0, 0x10, 0x20, 0x7F, 0x30, 0xF7];
        let messages = [
            Ok(None),
            Ok(None),
            Ok(None),
            Ok(None),
            Ok(None),
            Ok(Some([0xF0, 0x10, 0x20, 0x7F, 0x30, 0xF7].as_ref())),
        ];

        for (byte, message) in bytes.iter().zip(messages.iter()) {
            let result = sysex_message.append(*byte);
            assert_eq!(result, *message);
        }
    }

    /// Message with an invalid data byte.
    #[test]
    fn sysex_message_invalid_data() {
        let mut sysex_message = MidiSysexMessage::<256>::new();

        let bytes = [0xF0, 0x10, 0x20, 0x80, 0x7F, 0x30, 0xF7];
        let messages = [
            Ok(None),
            Ok(None),
            Ok(None),
            Err(MidiSysexMessageError::InvalidData),
            Ok(None),
            Ok(None),
            Ok(Some([0xF0, 0x10, 0x20, 0x7F, 0x30, 0xF7].as_ref())),
        ];

        for (byte, message) in bytes.iter().zip(messages.iter()) {
            let result = sysex_message.append(*byte);
            assert_eq!(result, *message);
        }
    }

    /// Message with 2 consecutive messages and unsound bytes in-between.
    #[test]
    fn sysex_message_consecutive_unsound() {
        let mut sysex_message = MidiSysexMessage::<256>::new();

        let bytes = [
            0xF0, 0x10, 0x20, 0x7F, 0x30, 0xF7, 0x60, 0xF7, 0xF0, 0x11, 0x21, 0x31, 0xF7,
        ];
        let messages = [
            Ok(None),
            Ok(None),
            Ok(None),
            Ok(None),
            Ok(None),
            Ok(Some([0xF0, 0x10, 0x20, 0x7F, 0x30, 0xF7].as_ref())),
            Err(MidiSysexMessageError::NoStartByte),
            Err(MidiSysexMessageError::NoStartByte),
            Ok(None),
            Ok(None),
            Ok(None),
            Ok(None),
            Ok(Some([0xF0, 0x11, 0x21, 0x31, 0xF7].as_ref())),
        ];

        for (byte, message) in bytes.iter().zip(messages.iter()) {
            let result = sysex_message.append(*byte);
            assert_eq!(result, *message);
        }
    }

    /// Message with overflow.
    #[test]
    fn sysex_message_overflow() {
        let mut sysex_message = MidiSysexMessage::<4>::new();

        let bytes = [0xF0, 0x10, 0x20, 0x7F, 0x30, 0xF7, 0xF0, 0x10, 0x20, 0xF7];
        let messages = [
            Ok(None),
            Ok(None),
            Ok(None),
            Ok(None),
            Err(MidiSysexMessageError::CapacityExceeded),
            Err(MidiSysexMessageError::CapacityExceeded),
            Err(MidiSysexMessageError::CapacityExceeded),
            Err(MidiSysexMessageError::CapacityExceeded),
            Err(MidiSysexMessageError::CapacityExceeded),
            Err(MidiSysexMessageError::CapacityExceeded),
        ];

        for (byte, message) in bytes.iter().zip(messages.iter()) {
            let result = sysex_message.append(*byte);
            assert_eq!(result, *message);
        }
    }

    /// Message capacity checks.
    #[test]
    fn sysex_message_capacity() {
        let mut sysex_message = MidiSysexMessage::<4>::new();

        assert_eq!(sysex_message.capacity(), 4);

        let _ = sysex_message.append(0x37);
        assert_eq!(sysex_message.len(), 0);
        assert_eq!(sysex_message.is_empty(), true);

        let _ = sysex_message.append(0xF0);
        assert_eq!(sysex_message.len(), 1);
        assert_eq!(sysex_message.free_capacity(), 3);

        let _ = sysex_message.append(0x10);
        let _ = sysex_message.append(0x20);
        assert_eq!(sysex_message.is_empty(), false);
        assert_eq!(sysex_message.is_full(), false);

        let _ = sysex_message.append(0x30);
        assert_eq!(sysex_message.len(), 4);
        assert_eq!(sysex_message.free_capacity(), 0);
        assert_eq!(sysex_message.is_full(), true);

        let _ = sysex_message.append(0xF7);
        assert_eq!(sysex_message.len(), 4);
        assert_eq!(sysex_message.free_capacity(), 0);
        assert_eq!(sysex_message.is_full(), true);

        sysex_message.clear();
        assert_eq!(sysex_message.is_empty(), true);

        let _ = sysex_message.append(0xF0);
        assert_eq!(sysex_message.len(), 1);
        assert_eq!(sysex_message.free_capacity(), 3);
    }
}
