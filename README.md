# Qubit Binary Codec

[![Rust CI](https://github.com/qubit-ltd/rs-codec-binary/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-codec-binary/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-codec-binary/coverage-badge.json)](https://qubit-ltd.github.io/rs-codec-binary/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-codec-binary.svg?color=blue)](https://crates.io/crates/qubit-codec-binary)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Chinese Document](https://img.shields.io/badge/Document-Chinese-blue.svg)](README.zh_CN.md)

Buffer-oriented binary codecs for Rust.

## Overview

Qubit Binary Codec provides low-level codecs for caller-managed byte buffers. It
does not depend on `std::io`; stream reader and writer adapters live in
`qubit-io-binary`.

This crate provides:

- `BinaryCodec` for fixed-width scalar encoding and decoding.
- `Leb128Codec` for unsigned and signed LEB128 values.
- `ZigZagCodec` for ZigZag signed integer mapping over unsigned LEB128.
- `Strict` and `NonStrict` decode policies.
- `Leb128DecodeError` and `Leb128DecodeErrorKind`.
- Re-exports of `ByteOrder`, `BigEndian`, `LittleEndian`, and `Coder` core
  primitives from `qubit-codec`.

## Installation

```toml
[dependencies]
qubit-codec-binary = "0.1"
```

## Quick Example

```rust
use qubit_codec_binary::{
    BigEndian,
    BinaryCodec,
    Leb128Codec,
    NonStrict,
};

let mut fixed = [0_u8; BinaryCodec::<u32, BigEndian>::REQUIRED_MIN_BUFFER_LEN];
unsafe {
    BinaryCodec::<u32, BigEndian>::write_unchecked(&mut fixed, 0, 0x0102_0304);
}
assert_eq!([1, 2, 3, 4], fixed);

let mut compact = [0_u8; Leb128Codec::<u64, NonStrict>::REQUIRED_MIN_BUFFER_LEN];
let written = unsafe { Leb128Codec::<u64, NonStrict>::write_unchecked(&mut compact, 0, 300) };
assert_eq!(2, written);
```

## Crate Boundary

`qubit-codec-binary` only contains buffer-level binary codecs. Use
`qubit-codec` for shared core traits, `qubit-io` for generic `std::io` helpers,
and `qubit-io-binary` for stream-oriented binary readers and writers.

## Development

```bash
./align-ci.sh
RS_CI_SKIP_TOOLCHAIN_UPDATE=1 ./ci-check.sh
```

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for the
full license text.
