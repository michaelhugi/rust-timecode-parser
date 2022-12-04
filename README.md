### Rust timecode coder

A pure Rust **no_std** library for encoding and decoding timecode in real-time.

## Decode LTC

Add dependency to Cargo.toml

```toml
[dependencies]
timecode_coder = { version = "0.1.0", features = ["decode_ltc"] }
```

Let's say you have a function that receives buffers from your audio interface:

```rust
use timecode_coder::ltc_decoder::LtcDecoder;

struct MyAudioHandler {
    decoder: LtcDecoder<u16>,
}

impl MyAudioHandler {
    // Sampling rate can by any Type that implements `FromPrimitive` 
    fn new(sampling_rate: u32) -> Self {
        Self {
            decoder: LtcDecoder::new(sampling_rate)
        }
    }
    fn new_buffer(&mut self, samples: [u16; 512]) {
        for sample in samples {
            if let Some(timecode_frame) = get_timecode_frame(sample) {
                /// New TimecodeFrame received
            }
        }
    }
}

```
`TimecodeFrame` provides:
- hours
- minutes
- seconds
- frames
- frame-rate (auto detected)

<strong>Warning. Drop frames are not yet supported. They will be detected as normal '25fps' or '30fps'</strong>

## Encode LTC

**not yet implemented**

## Decode MIDI

**not yet implemented**

## Encode MIDI

**not yet implemented**