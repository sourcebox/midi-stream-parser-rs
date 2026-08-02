# midi-stream-parser

This `no_std` Rust crate contains a parser that takes a stream of bytes from a MIDI source (typically a serial input on an embedded device) and converts them into well-formed messages for further processing.

It internally handles the special cases like *Running Status* or *System Realtime* messages that can be inserted in-between other messages.

*System Exclusive* data is split from regular messages and can be processed separately with arbitrary length.

## Usage Example

Feed the stream into the parser byte-per-byte and process the result.

```rust no_run
use midi_stream_parser::*;

// Create an instance of the parser.
let mut parser = MidiStreamParser::new();

// Create an instance of a buffered SysEx message.
// The generic argument is the capacity in bytes that the 
// internal buffer can hold.
let mut sysex_message = MidiSysexMessage::<256>::new(); 

// Read the raw bytes from the stream.
// In reality, it's typically received by a UART or similar.
// For now, we just some dummy data.
let bytes = [0x90, 60, 127, 61, 40];

// Feed each byte into the parser.
// Whenever a regular message is complete, it will be returned as whole.
// SysEx data however is returned as individual bytes.
for byte in bytes {
    if let Some(output) = parser.feed(byte) {
        match output {
            ParserOutput::Message(message) => {
                // Slice containing a full message.
                println!("Message: {:?}", message);                
            }
            ParserOutput::SysexByte(byte) => {
                // Single byte of a SysEx message.
                // Here, the byte is appended to a buffered message,
                // but it can also be processed on-the-fly otherwise.
                if let Some(message) = sysex_message.append(byte) {
                    // SysEx message is now complete.
                    println!("SysEx message complete: {:?}", message);
                }
            }
        }
    }
}
```

## Tests

Run `cargo test` for the unit tests.

## License

Published under the MIT license. Any contribution to this project must be provided under the same license conditions.

Author: Oliver Rockstedt <info@sourcebox.de>
