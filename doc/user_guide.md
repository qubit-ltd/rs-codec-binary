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

The crate re-exports `ByteOrder`, `BigEndian`, `LittleEndian`, and `Coder` from
`qubit-codec`.

## Fixed-Width Values

```rust
use qubit_codec_binary::{
    BigEndian,
    BinaryCodec,
};

let mut output = [0_u8; BinaryCodec::<u32, BigEndian>::REQUIRED_MIN_BUFFER_LEN];
unsafe {
    BinaryCodec::<u32, BigEndian>::write_unchecked(&mut output, 0, 0x0102_0304);
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

let mut unsigned = [0_u8; Leb128Codec::<u64, NonStrict>::REQUIRED_MIN_BUFFER_LEN];
let written = unsafe { Leb128Codec::<u64, NonStrict>::write_unchecked(&mut unsigned, 0, 300) };
assert_eq!(2, written);

let mut signed = [0_u8; ZigZagCodec::<i64, NonStrict>::REQUIRED_MIN_BUFFER_LEN];
let written = unsafe { ZigZagCodec::<i64, NonStrict>::write_unchecked(&mut signed, 0, -42) };
assert_eq!(1, written);
```

Use `qubit-io-binary` when you need `std::io::Read` / `Write` adapters around
these codecs.
