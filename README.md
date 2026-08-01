# midi-stream-parser

This `no_std` Rust crate contains a parser that takes a stream of bytes from a MIDI source (typically a serial input on an embedded device) and converts them into well-formed messages for further processing.

## Usage Example

Feed the stream into the parser byte-per-byte and process the result. This is required because *System Realtime* messages can be present in-between other messages and must be processed with priority.

```rust no_run
use midi_stream_parser::*;

// Get an instance of the parser
let mut parser = MidiStreamParser::new();

// Read the bytes from the stream, just some demo data here.
let bytes = [0x90, 60, 127, 61, 40];

// Feed each byte into the parser.
// Whenever a message is ready, it will be returned, otherwise `None`.
for byte in bytes {
    if let Some(output) = parser.parse(byte) {
        match output {
            ParserOutput::Message(message) => {
                // Slice containing a full message.
                println!("Message: {:?}", message);                
            }
            ParserOutput::SysexByte(byte) => {
                // Single byte of a SysEx message.
                println!("SysEx byte: {}", byte);
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
