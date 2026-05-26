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
- Re-exports of `Codec`, `ByteOrder`, `BigEndian`, `LittleEndian`, and `Coder`
  core primitives from `qubit-codec`.

## Design Goals

- **Buffer First**: operate on caller-owned byte slices without requiring
  `Read` or `Write`.
- **Hot-Path Efficiency**: provide unchecked static codec methods for callers
  that already validated buffer bounds, plus `Codec<Value, u8>` implementations
  for generic codec pipelines.
- **Precise Layering**: depend only on `qubit-codec`, leaving stream adapters to
  `qubit-io-binary`.
- **Canonical Encoding**: always emit canonical LEB128 bytes while allowing
  configurable decode strictness.
- **Typed Byte Order**: support both runtime and type-level endian selection.
- **Small Dependency Graph**: keep binary wire-format code usable by low-level
  crates without pulling in generic I/O utilities.

## Features

### Fixed-Width Binary Scalars

- **Integer Coverage**: encodes and decodes primitive signed and unsigned
  integer types.
- **Byte Order Support**: supports `BigEndian` and `LittleEndian` type markers.
- **Unchecked Hot Path**: `read_unchecked` and `write_unchecked` avoid repeated
  bounds checks after the caller validates capacity.

### LEB128 Values

- **Unsigned Values**: supports `u8`, `u16`, `u32`, `u64`, `u128`, and `usize`.
- **Signed Values**: supports `i8`, `i16`, `i32`, `i64`, `i128`, and `isize`.
- **Strict Decode Policy**: `Strict` rejects non-canonical payloads.
- **Non-Strict Decode Policy**: `NonStrict` accepts compatible payloads when
  canonical form is not required.

### ZigZag Values

- **Signed Integer Mapping**: maps signed integers to unsigned LEB128 payloads.
- **Buffered Decode Support**: exposes partial-buffer decode entry points used by
  stream readers.

### Focused Public API

- **`prelude` module**: imports binary codec types and core byte-order markers.
- **Core codec trait**: `BinaryCodec`, `Leb128Codec`, and `ZigZagCodec`
  implement `qubit_codec::Codec<Value, u8>`.
- **No `std::io` adapters**: stream helpers live in `qubit-io-binary`.

## Documentation

- [User Guide](doc/user_guide.md)
- [API Reference](https://docs.rs/qubit-codec-binary)
- [Chinese README](README.zh_CN.md)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
qubit-codec-binary = "0.1"
```

## Quick Start

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

## API Reference

### `BinaryCodec` Operations

| Item | Description |
|------|-------------|
| `Codec<Value, u8>` | Decode and encode one fixed-width scalar through the core trait |
| `REQUIRED_MIN_BUFFER_LEN` | Minimum bytes required for the scalar type |
| `read_unchecked(input, index)` | Decode one fixed-width scalar without bounds checks |
| `write_unchecked(output, index, value)` | Encode one fixed-width scalar without bounds checks |

### `Leb128Codec` Operations

| Item | Description |
|------|-------------|
| `Codec<Value, u8>` | Decode and encode one LEB128 value through the core trait |
| `REQUIRED_MIN_BUFFER_LEN` | Maximum bytes needed for the integer type |
| `read_unchecked(input, index)` | Decode one complete LEB128 value |
| `read_available_unchecked(input, index, available)` | Decode from a partial available buffer |
| `write_unchecked(output, index, value)` | Encode one canonical LEB128 value |

### `ZigZagCodec` Operations

| Item | Description |
|------|-------------|
| `Codec<Value, u8>` | Decode and encode one ZigZag LEB128 value through the core trait |
| `REQUIRED_MIN_BUFFER_LEN` | Maximum bytes needed for the signed integer type |
| `read_unchecked(input, index)` | Decode ZigZag over unsigned LEB128 |
| `read_available_unchecked(input, index, available)` | Decode from a partial available buffer |
| `write_unchecked(output, index, value)` | Encode signed integer as ZigZag plus unsigned LEB128 |

### Decode Policies

| Policy | Meaning |
|--------|---------|
| `Strict` | Reject non-canonical LEB128 encodings |
| `NonStrict` | Accept compatible encodings when the decoded value fits |

## Crate Boundary

`qubit-codec-binary` only contains buffer-level binary codecs. Use
`qubit-codec` for shared core traits, `qubit-io` for generic `std::io` helpers,
and `qubit-io-binary` for stream-oriented binary readers and writers.

## Performance Considerations

`BinaryCodec`, `Leb128Codec`, and `ZigZagCodec` are zero-sized codec types with
no runtime allocation. Their unchecked methods and `Codec<Value, u8>`
implementations are intended for validated hot paths where a caller has already
checked buffer capacity or is operating inside a buffered stream adapter.

## Testing & Code Coverage

This project keeps binary wire-format behavior covered by integration tests
under `tests/`.

### Running Tests

```bash
# Run all tests
cargo test

# Run with coverage report
./coverage.sh

# Generate text format report
./coverage.sh text

# Align code with CI requirements
./align-ci.sh

# Run CI checks (format, clippy, test, coverage, audit)
RS_CI_SKIP_TOOLCHAIN_UPDATE=1 ./ci-check.sh
```

## Dependencies

Runtime dependencies are intentionally small:

- `qubit-codec` provides shared byte-order and coder primitives.
- `thiserror` provides the public LEB128 error type implementation.

## License

Copyright (c) 2026. Haixing Hu.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.

See [LICENSE](LICENSE) for the full license text.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Development Guidelines

- Keep this crate focused on buffer-level binary codecs.
- Add tests for canonical and non-canonical wire-format cases.
- Document public APIs and safety contracts.
- Ensure all checks pass before submitting a PR.

## Author

**Haixing Hu**

## Related Projects

- [qubit-codec](https://github.com/qubit-ltd/rs-codec): shared core codec
  traits and byte-order markers.
- [qubit-io-binary](https://github.com/qubit-ltd/rs-io-binary): stream adapters
  for these binary codecs.
- [qubit-io](https://github.com/qubit-ltd/rs-io): generic `std::io` helpers.
- More Rust libraries from Qubit are available under the
  [qubit-ltd](https://github.com/qubit-ltd) GitHub organization.

---

Repository: [https://github.com/qubit-ltd/rs-codec-binary](https://github.com/qubit-ltd/rs-codec-binary)
