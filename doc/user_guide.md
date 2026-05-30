# Qubit Binary Codec User Guide

`qubit-codec-binary` contains buffer-level binary codecs. It is intended for
parsers, binary formats, and stream adapters that already own their buffers and
want explicit byte indexes.

## Layers

- Use `BinaryCodec<T, O>` for fixed-width integers and floats.
- Use `Leb128Codec<T, P>` for unsigned and signed LEB128 values.
- Use `ZigZagCodec<T, P>` when signed values should be compact around zero.
- Use `Strict` to reject non-canonical LEB128 payloads and `NonStrict` to allow
  permissive decoding.

The crate re-exports `Codec`, `CodecValueEncoder`, `CodecBufferedEncoder`,
`CodecBufferedDecoder`, `BufferedEncodeEngine`, `BufferedDecodeEngine`,
`BufferedEncodeHooks`, `BufferedDecodeHooks`, `EncodePlan`, `CodecEncodeError`,
`CodecDecodeError`, `DecodeErrorFactory`, `ValueEncoder`, `ValueDecoder`,
`ByteOrder`, `BigEndian`, `LittleEndian`, and `Transcoder` from
`qubit-codec`.

## Fixed-Width Values

```rust
use qubit_codec_binary::{
    BigEndian,
    BinaryCodec,
};

let mut output = [0_u8; BinaryCodec::<u32, BigEndian>::REQUIRED_MIN_BUFFER_LEN];
unsafe {
    BinaryCodec::<u32, BigEndian>::encode_unchecked(0x0102_0304, &mut output, 0);
}
assert_eq!([1, 2, 3, 4], output);
```

The unchecked APIs are for hot paths where the caller has already validated
buffer capacity.

## LEB128 and ZigZag

```rust
use qubit_codec_binary::{
    Leb128Codec,
    NonStrict,
    ZigZagCodec,
};

let mut unsigned = [0_u8; Leb128Codec::<u64, NonStrict>::MAX_UNITS_PER_VALUE];
let written = unsafe { Leb128Codec::<u64, NonStrict>::encode_unchecked(300, &mut unsigned, 0) };
assert_eq!(2, written);

let mut signed = [0_u8; ZigZagCodec::<i64, NonStrict>::MAX_UNITS_PER_VALUE];
let written = unsafe { ZigZagCodec::<i64, NonStrict>::encode_unchecked(-42, &mut signed, 0) };
assert_eq!(1, written);
```

`MIN_UNITS_PER_VALUE` is useful for deciding whether decode can even start.
`MAX_UNITS_PER_VALUE` is the capacity upper bound used when sizing output
buffers or when a caller cannot otherwise prove where the terminating byte is.

## Unsafe Boundary

These codecs are low-level building blocks. Their unchecked methods do not own
the responsibility of discovering whether a buffer has enough space:

- Fixed-width `BinaryCodec` decode and encode calls require
  `REQUIRED_MIN_BUFFER_LEN` readable or writable bytes from the supplied index.
- LEB128 and ZigZag encode calls require `MAX_UNITS_PER_VALUE` writable bytes
  from the supplied index.
- LEB128 and ZigZag decode calls require at least `MIN_UNITS_PER_VALUE`
  readable byte from the supplied index. Callers should normally provide up to
  `MAX_UNITS_PER_VALUE` readable bytes unless EOF prevents that.
- If EOF prevents the caller from providing enough readable bytes to complete a
  variable-length value, `decode_unchecked` reports the incomplete value through
  `Leb128DecodeError`.

When exposing a safe API, validate these conditions before crossing the unsafe
boundary.

## Wrapping With ValueEncoder

Use `ValueEncoder` when a safe API should encode one borrowed value into an
owned output object.

```rust
use core::convert::Infallible;

use qubit_codec_binary::{
    Leb128Codec,
    NonStrict,
    ValueEncoder,
};

struct U64Leb128Encoder;

impl ValueEncoder<u64> for U64Leb128Encoder {
    type Output = Vec<u8>;
    type Error = Infallible;

    fn encode(&self, input: &u64) -> Result<Self::Output, Self::Error> {
        let mut output = vec![0_u8; Leb128Codec::<u64, NonStrict>::MAX_UNITS_PER_VALUE];
        let written = unsafe {
            Leb128Codec::<u64, NonStrict>::encode_unchecked(*input, &mut output, 0)
        };
        output.truncate(written);
        Ok(output)
    }
}
```

The wrapper allocates the maximum possible output length before calling
`encode_unchecked`, then truncates to the actual written length.

## Wrapping With ValueDecoder

Use `ValueDecoder` when a safe API should decode one borrowed input object into
an owned value.

```rust
use qubit_codec_binary::{
    Leb128Codec,
    Leb128DecodeError,
    NonStrict,
    ValueDecoder,
};

struct U64Leb128Decoder;

impl ValueDecoder<[u8]> for U64Leb128Decoder {
    type Output = u64;
    type Error = Leb128DecodeError;

    fn decode(&self, input: &[u8]) -> Result<Self::Output, Self::Error> {
        let (value, _consumed) = unsafe {
            Leb128Codec::<u64, NonStrict>::decode_unchecked(input, 0)?
        };
        Ok(value)
    }
}
```

The wrapper calls `decode_unchecked` directly because `Leb128DecodeError`
reports incomplete, malformed, and non-canonical input itself.

## Wrapping With CodecBufferedDecoder

Use `CodecBufferedDecoder<C, u8>` when a safe API should decode many binary
values into a caller-provided output buffer while leaving incomplete tails in
the caller-owned input buffer. For custom binary decoders, use
`BufferedDecodeEngine` with `BufferedDecodeHooks` to share the same decode-loop
logic while supplying domain-specific error policy.

## Wrapping With Transcoder

Use `Transcoder` when a safe API should process a sequence over caller-provided
buffers and return progress when output capacity runs out.

```rust
use core::convert::Infallible;

use qubit_codec_binary::{
    Leb128Codec,
    NonStrict,
    TranscodeProgress,
    TranscodeStatus,
    Transcoder,
};

struct U64Leb128Transcoder;

impl Transcoder<u64, u8> for U64Leb128Transcoder {
    type Error = Infallible;

    fn max_output_len(&self, input_len: usize) -> Option<usize> {
        Some(input_len.saturating_mul(
            Leb128Codec::<u64, NonStrict>::MAX_UNITS_PER_VALUE,
        ))
    }

    fn transcode(
        &mut self,
        input: &[u64],
        input_index: usize,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        let mut read = 0;
        let mut written = 0;
        while input_index + read < input.len() {
            let cursor = output_index + written;
            let available = output.len().saturating_sub(cursor);
            let required = Leb128Codec::<u64, NonStrict>::MAX_UNITS_PER_VALUE;
            if available < required {
                return Ok(TranscodeProgress::new(
                    TranscodeStatus::NeedOutput {
                        output_index: cursor,
                        required,
                        available,
                    },
                    read,
                    written,
                ));
            }
            let value = input[input_index + read];
            let len = unsafe {
                Leb128Codec::<u64, NonStrict>::encode_unchecked(value, output, cursor)
            };
            read += 1;
            written += len;
        }
        Ok(TranscodeProgress::complete(read, written))
    }
}
```

The transcoder checks output capacity before every unsafe encode call and
returns `NeedOutput` instead of writing into a short buffer.

Use `qubit-io-binary` when you need `std::io::Read` / `Write` adapters around
these codecs.
