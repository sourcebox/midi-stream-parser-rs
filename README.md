# midi-stream-parser

This `no_std` Rust crate contains a parser that takes a stream of bytes from a MIDI source (typically a serial input on an embedded device) and converts them into well-formed messages for further processing.

Currently, only MIDI 1.0 messages are supported.

## Usage Example

Feed the stream into the parser byte-per-byte and process the result. This is required because *System Realtime* messages can be present in-between other messages and must be processed with priority.

```rust
// Maximum length of internal SysEx buffer in bytes
const SYSEX_MAX_LEN: usize = 256;

// Get an instance of the parser
let mut parser = midi_stream_parser::MidiStreamParser::<SYSEX_MAX_LEN>::new();

// Read the bytes from the stream, just some demo data here.
let bytes = [0x90, 60, 127, 61, 40];

// Feed each byte into the parser. For simplicity, errors are discarded here by using `ok()`.
// Whenever a message is ready, it will be returned, otherwise `None`.
for byte in bytes {
    if let Some(message) = parser.parse(byte).ok() {
        println!("Message: {:?}", message);
    }
}
```

## Tests

Run `cargo test` for the unit tests.

## License

Published under the MIT license. Any contribution to this project must be provided under the same license conditions.

Author: Oliver Rockstedt <info@sourcebox.de>
