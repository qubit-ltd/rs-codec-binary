# Qubit Binary Codec User Guide

[中文](user_guide.zh_CN.md) · [README](../README.md) · [API reference](https://docs.rs/qubit-codec-binary)

This guide applies to `qubit-codec-binary` 0.6. It is for Rust developers
implementing byte-buffer protocols or file formats that need fixed-width and
LEB128 fields while retaining ownership of buffering and I/O.

## Conceptual Model

The crate works at one boundary: one value and caller-managed byte slices.

```text
application buffer -> codec<T, policy> -> value or wire bytes
                         |
                         +-> malformed/incomplete/non-canonical diagnostics
```

`BinaryCodec<T, O>` handles fixed-width values with a type-level byte order.
`Leb128Codec<T, P>` handles signed or unsigned variable-width integers;
`ZigZagCodec<T, P>` represents signed values as ZigZag followed by unsigned
LEB128. All implement `qubit_codec::Codec` with `Unit = u8`.

Multi-byte `BinaryCodec` values support `BigEndian`, `LittleEndian`, and
`NativeEndian`. Use big- or little-endian encoding for persistent or
cross-platform data. Native-endian encoding is only suitable for data that is
read on the same platform class that wrote it.

## Scenario: Decode a Compact Record

Assume a record begins with a big-endian `u32` identifier followed by a signed
ZigZag LEB128 delta. The encoder reserves each codec's declared maximum before
entering the unsafe layer; the decoder supplies a complete record slice.

## Installation and Minimal Configuration

```toml
[dependencies]
qubit-codec-binary = "0.6"
qubit-codec = "0.17"
```

Import byte-order markers and the shared `Codec` trait from `qubit-codec`.
For ordinary one-value conversion, import `CodecValueEncoder` and
`CodecValueDecoder` as checked adapters:

```rust
use qubit_codec::{
    CodecValueDecoder,
    CodecValueEncoder,
    ValueDecoder,
    ValueEncoder,
};
use qubit_codec_binary::{Leb128Codec, NonStrict};

let mut encoder =
    CodecValueEncoder::new(Leb128Codec::<u64, NonStrict>::default());
let encoded = encoder.encode(&300).expect("u64 is always encodable");

let mut decoder =
    CodecValueDecoder::new(Leb128Codec::<u64, NonStrict>::default());
let decoded = decoder.decode(&encoded).expect("encoded value is valid");
assert_eq!(300, decoded);
```

`CodecValueDecoder::decode` expects a slice containing exactly one value. An
empty slice is reported as incomplete input, while bytes after the decoded
value are reported as trailing input. For fixed-width fields, slice the known
width before calling the adapter. For variable-width fields, let the protocol
provide the value boundary, or use a protocol or stream reader that can track
that boundary. The adapter does not return a consumed prefix or an offset for
the next field.

## Core Workflow

```rust
use qubit_codec::BigEndian;
use qubit_codec_binary::{BinaryCodec, Strict, ZigZagCodec};

let mut record = [0_u8; 9];
let id_len = unsafe {
    BinaryCodec::<u32, BigEndian>::encode(0x0102_0304, &mut record, 0)
};
let delta_len = unsafe {
    ZigZagCodec::<i32, Strict>::encode(-42, &mut record, id_len)
};
assert_eq!(5, id_len + delta_len);

let (id, used_id) = unsafe { BinaryCodec::<u32, BigEndian>::decode(&record, 0) };
let (delta, used_delta) = unsafe {
    ZigZagCodec::<i32, Strict>::decode(&record, used_id.get())
}
.expect("the encoded delta is canonical");

assert_eq!(0x0102_0304, id);
assert_eq!(-42, delta);
assert_eq!(id_len, used_id.get());
assert_eq!(delta_len, used_delta.get());
```

The `MAX_ENCODE_UNITS_PER_VALUE` allocation is deliberately conservative for LEB128
and ZigZag; encode returns the actual byte count. Fixed-width codecs consume
and write their exact scalar width.
Use `MAX_DECODE_UNITS_PER_VALUE` for decode-side bounded buffers and type-width
validation. The two bounds are equal for the current binary codecs but have
independent contracts.

## LEB128 Policies and Errors

Encoding is always canonical. Select the decoding policy according to the wire
contract; every `Leb128Codec<T, P>` and `ZigZagCodec<T, P>` instantiation must
name `P` explicitly:

| Policy | Use when | Result for `80 00` as an unsigned zero |
| --- | --- | --- |
| `Strict` | Your format requires a unique byte representation. | `NonCanonical` error. |
| `NonStrict` | You must accept compatible legacy or permissive input. | Decodes to `0`. |

The policy does not affect encoding. Encoding-only code should conventionally
instantiate the codec with `NonStrict`.

`NonStrict` accepts non-canonical representations only within the target
integer's declared maximum width. Longer unterminated or over-width payloads
remain malformed rather than being accepted as arbitrarily long encodings.

The following `Leb128DecodeError` example uses the direct codec path. It reports
`Incomplete`, `Malformed`, or `NonCanonical`:
`start_index()` identifies the attempted value, and `error_index()` identifies
where failure became observable. For incomplete data, use `required()`,
`available()`, and `additional()` to decide whether to refill a buffer.

```rust
use qubit_codec_binary::{Leb128Codec, Leb128DecodeErrorKind, NonStrict};

let error = unsafe { Leb128Codec::<u16, NonStrict>::decode(&[0xac], 0) }
    .expect_err("one continuation byte is incomplete");
assert_eq!(Leb128DecodeErrorKind::Incomplete, error.kind());
assert_eq!(Some(2), error.required().map(|count| count.get()));
assert_eq!(Some(1), error.available());
```

The checked single-value path returns
`TranscodeDecodeError<Leb128DecodeError>`. Framework failures such as an empty
slice or trailing bytes are available through `failure_ref()`; codec-specific
errors are available through `domain_ref()`:

```rust
use qubit_codec::CodecValueDecoder;
use qubit_codec::TranscodeFailure;
use qubit_codec_binary::Leb128Codec;
use qubit_codec_binary::Leb128DecodeErrorKind;
use qubit_codec_binary::Strict;

fn main() {
    let mut decoder = CodecValueDecoder::new(Leb128Codec::<u16, Strict>::default());
    assert_eq!(300, decoder.decode(&[0xac, 0x02]).unwrap());

    let empty = decoder.decode(&[]).unwrap_err();
    assert!(matches!(empty.failure_ref(), Some(TranscodeFailure::IncompleteInput { .. })));

    let truncated = decoder.decode(&[0xac]).unwrap_err();
    assert_eq!(
        Leb128DecodeErrorKind::Incomplete,
        truncated.domain_ref().unwrap().kind(),
    );

    let noncanonical = decoder.decode(&[0x80, 0x00]).unwrap_err();
    assert_eq!(
        Leb128DecodeErrorKind::NonCanonical,
        noncanonical.domain_ref().unwrap().kind(),
    );

    let trailing = decoder.decode(&[0x01, 0x02]).unwrap_err();
    assert!(matches!(trailing.failure_ref(), Some(TranscodeFailure::TrailingInput { .. })));
}
```

Choose the next action according to the input boundary and error layer:

| Input | Error layer | Application action |
| --- | --- | --- |
| Empty slice | Framework `IncompleteInput` | Check the boundary. If more data may arrive, retain the input and retry after the outer layer supplies it. |
| `[0xac]` | Domain `Incomplete` | This call failed; the outer layer may append data and retry. Reject it if the protocol has reached final EOF. |
| `[0x80, 0x00]` under `Strict` | Domain `NonCanonical` | Reject it under this protocol. Select `NonStrict` only when the protocol allows that representation; do not downgrade automatically. |
| `[0x01, 0x02]` | Framework `TrailingInput` | Check the single-value slice boundary or use a record reader; do not silently discard the trailing byte. |

## Unsafe Boundary and Best Practices

The direct codec methods do not check slice bounds. Before calling them:

1. For `BinaryCodec`, make at least `MIN_UNITS_PER_VALUE` bytes readable or
   `MAX_ENCODE_UNITS_PER_VALUE` bytes writable from the index.
2. For LEB128 and ZigZag encoding, reserve `MAX_ENCODE_UNITS_PER_VALUE` writable
   bytes, then retain only the returned prefix.
3. For LEB128 and ZigZag decoding, supply at least one readable byte and,
   where possible, all bytes currently buffered up to
   `MAX_DECODE_UNITS_PER_VALUE`.
4. Keep `usize` and `isize` out of persistent or cross-platform formats: their
   bounds follow the target pointer width. Prefer fixed-width integers.

Use the checked adapters above, or a checked protocol reader/writer, when input
comes from untrusted buffers. Use `qubit-io-binary` for stream-oriented binary
adapters; this crate neither owns buffering nor maps errors to `std::io::Error`.

## Further Reading

- [README](../README.md)
- [中文用户指南](user_guide.zh_CN.md)
- [API reference](https://docs.rs/qubit-codec-binary)
