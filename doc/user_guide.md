# Qubit Binary Codec User Guide

`qubit-codec-binary` contains buffer-level binary codecs. It is intended for
parsers, binary formats, and stream adapters that already own their buffers and
want explicit byte indexes.

## Layers

- Use `BinaryCodec<T, O>` for explicit-width primitive integers and floats.
- Use `Leb128Codec<T, P>` for unsigned and signed LEB128 values.
- Use `ZigZagCodec<T, P>` when signed values should be compact around zero.
- Use `Strict` to reject non-canonical LEB128 payloads and `NonStrict` to allow
  permissive decoding.

The crate re-exports only the `qubit-codec` primitives that are part of the
binary codec surface: `Codec`, `ByteOrder`, `ByteOrderSpec`, `BigEndian`,
and `LittleEndian`. It also exposes the sealed `Leb128DecodePolicy` trait for
the built-in LEB128 policy markers. Import generic adapters, engines, hooks,
and value traits directly from `qubit-codec`.

## Fixed-Width Values

`BinaryCodec` supports `u8`, `i8`, `u16`, `i16`, `u32`, `i32`, `u64`, `i64`,
`u128`, `i128`, `f32`, and `f64`. It does not implement `usize` or `isize`
because their byte width is platform-dependent.

```rust
use qubit_codec_binary::{
    BigEndian,
    BinaryCodec,
};

let mut output = [0_u8; BinaryCodec::<u32, BigEndian>::MAX_UNITS_PER_VALUE];
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

## LEB128 Decode Errors

`Leb128DecodeError` separates the attempted value start from the point where an
error became observable:

- `start_index()` returns the byte index where the attempted value starts.
- `error_index()` returns the byte index where decoding detected the error. For
  incomplete input this is the one-past-available boundary where the next byte
  would be required.
- `required()`, `available()`, and `additional()` describe incomplete input.
- `consumed()` describes how many invalid bytes a caller may consume after
  malformed or non-canonical input.

## Unsafe Boundary

These codecs are low-level building blocks. Their unchecked methods do not own
the responsibility of discovering whether a buffer has enough space:

- Fixed-width `BinaryCodec` decode and encode calls require
  `MIN_UNITS_PER_VALUE` readable bytes or `MAX_UNITS_PER_VALUE` writable bytes
  from the supplied index. For fixed-width values these bounds are equal.
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

## Safe Adapters and Streams

This crate intentionally stops at buffer-level binary codecs. If you need owned
single-value adapters, generic buffered engines, or conversion traits, import
them from `qubit-codec` and use these binary codecs as the low-level codec
implementations.

When writing your own safe wrapper around `decode_unchecked`, check that the
start index has at least `MIN_UNITS_PER_VALUE` readable byte before calling the
unsafe method. For single-value decoders, also decide whether trailing bytes
after the returned `consumed` count are allowed by your format.

Use `qubit-io-binary` when you need `std::io::Read` / `Write` adapters around
these codecs.
