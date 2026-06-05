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
- `Strict` and `NonStrict` sealed LEB128 decode policies.
- `Leb128DecodePolicy` for the built-in LEB128 policy markers.
- `Leb128DecodeError` and `Leb128DecodeErrorKind`.
- Essential `qubit-codec` primitives used by the binary surface: `Codec`,
  `ByteOrder`, `ByteOrderSpec`, `BigEndian`, and `LittleEndian`.

## Design Goals

- **Buffer First**: operate on caller-owned byte slices without requiring
  `Read` or `Write`.
- **Hot-Path Efficiency**: provide unchecked static codec methods for callers
  that already validated buffer bounds, plus `Codec` implementations with `Unit = u8`
  for generic codec pipelines.
- **Precise Layering**: depend only on `qubit-codec`, leaving stream adapters to
  `qubit-io-binary`.
- **Canonical Encoding**: always emit canonical LEB128 bytes while allowing
  configurable decode strictness.
- **Typed Byte Order**: select endian behavior through type-level byte-order markers.
- **Small Dependency Graph**: keep binary wire-format code usable by low-level
  crates without pulling in generic I/O utilities.

## Features

### Fixed-Width Binary Scalars

- **Integer Coverage**: encodes and decodes explicit-width primitive integer
  types: `u8`, `i8`, `u16`, `i16`, `u32`, `i32`, `u64`, `i64`, `u128`, and
  `i128`. Platform-width `usize` and `isize` are intentionally not fixed-width
  binary scalar types.
- **Floating-Point Coverage**: supports `f32` and `f64` while preserving their
  IEEE 754 bit patterns.
- **Byte Order Support**: supports `BigEndian` and `LittleEndian` type markers.
- **Unchecked Hot Path**: `decode_unchecked` and `encode_unchecked` avoid repeated
  bounds checks after the caller validates capacity.

### LEB128 Values

- **Unsigned Values**: supports `u8`, `u16`, `u32`, `u64`, `u128`, and `usize`.
- **Signed Values**: supports `i8`, `i16`, `i32`, `i64`, `i128`, and `isize`.
- **Strict Decode Policy**: `Strict` rejects non-canonical payloads.
- **Non-Strict Decode Policy**: `NonStrict` accepts compatible payloads when
  canonical form is not required.
- **Sealed Policy Trait**: `Leb128DecodePolicy` is intentionally sealed; use the
  built-in `Strict` or `NonStrict` markers.

### ZigZag Values

- **Signed Integer Mapping**: maps signed integers to unsigned LEB128 payloads.
- **Incomplete Input Reporting**: reports partial LEB128 payloads through
  `Leb128DecodeError`.

### Focused Public API

- **`prelude` module**: imports binary codec types and core byte-order markers.
- **Core codec trait**: `BinaryCodec`, `Leb128Codec`, and `ZigZagCodec`
  implement `qubit_codec::Codec` with `Unit = u8` and a codec-specific `Value`.
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

let mut fixed = [0_u8; BinaryCodec::<u32, BigEndian>::MAX_UNITS_PER_VALUE];
unsafe {
    BinaryCodec::<u32, BigEndian>::encode_unchecked(0x0102_0304, &mut fixed, 0);
}
assert_eq!([1, 2, 3, 4], fixed);

let mut compact = [0_u8; Leb128Codec::<u64, NonStrict>::MAX_UNITS_PER_VALUE];
let written = unsafe { Leb128Codec::<u64, NonStrict>::encode_unchecked(300, &mut compact, 0) };
assert_eq!(2, written);
```

## Unchecked API Contracts

The low-level codec methods are intentionally unsafe. Callers must validate
buffer bounds before using them:

- `BinaryCodec::decode_unchecked` and `BinaryCodec::encode_unchecked` require
  exactly `MIN_UNITS_PER_VALUE` readable bytes or `MAX_UNITS_PER_VALUE`
  writable bytes from `index`. For fixed-width values these bounds are equal.
- `Leb128Codec` and `ZigZagCodec` expose `MIN_UNITS_PER_VALUE` and
  `MAX_UNITS_PER_VALUE`. Their `encode_unchecked` methods require
  `MAX_UNITS_PER_VALUE` writable bytes from `index`, even when the encoded value
  is shorter.
- `Leb128Codec::decode_unchecked` and `ZigZagCodec::decode_unchecked` require
  at least `MIN_UNITS_PER_VALUE` readable byte from `index`. Callers should
  normally provide up to `MAX_UNITS_PER_VALUE` readable bytes unless EOF makes
  that impossible. Incomplete, malformed, and non-canonical input is reported
  through `Leb128DecodeError`.
- `Leb128DecodeError::start_index()` identifies where the attempted value
  starts. `error_index()` identifies where the error became observable. For
  incomplete input, `error_index()` is the one-past-available boundary and
  `additional()` reports how many more bytes are needed before decoding can
  make progress.

Higher-level code should wrap these unsafe calls behind safe APIs after checking
the appropriate bounds. Import owned-value adapters and buffered engines
directly from `qubit-codec`; this crate does not re-export generic codec
adapters. See the [User Guide](doc/user_guide.md) for binary codec examples.

## API Reference

### `BinaryCodec` Operations

| Item | Description |
|------|-------------|
| `Codec` (`Unit = u8`) | Decode and encode one fixed-width scalar through the core trait |
| `MIN_UNITS_PER_VALUE` | Minimum bytes required for the scalar type |
| `MAX_UNITS_PER_VALUE` | Maximum bytes required for the scalar type |
| `decode_unchecked(input, index)` | Decode one fixed-width scalar without bounds checks |
| `encode_unchecked(value, output, index)` | Encode one fixed-width scalar without bounds checks |

### `Leb128Codec` Operations

| Item | Description |
|------|-------------|
| `Codec` (`Unit = u8`) | Decode and encode one LEB128 value through the core trait |
| `MIN_UNITS_PER_VALUE` | Minimum readable bytes that can contain a complete value |
| `MAX_UNITS_PER_VALUE` | Maximum bytes needed for the integer type |
| `decode_unchecked(input, index)` | Decode one complete LEB128 value |
| `encode_unchecked(value, output, index)` | Encode one canonical LEB128 value |

### `ZigZagCodec` Operations

| Item | Description |
|------|-------------|
| `Codec` (`Unit = u8`) | Decode and encode one ZigZag LEB128 value through the core trait |
| `MIN_UNITS_PER_VALUE` | Minimum readable bytes that can contain a complete value |
| `MAX_UNITS_PER_VALUE` | Maximum bytes needed for the signed integer type |
| `decode_unchecked(input, index)` | Decode ZigZag over unsigned LEB128 |
| `encode_unchecked(value, output, index)` | Encode signed integer as ZigZag plus unsigned LEB128 |

### LEB128 Decode Policies

| Policy | Meaning |
|--------|---------|
| `Strict` | Reject non-canonical LEB128 encodings |
| `NonStrict` | Accept compatible encodings when the decoded value fits |
| `Leb128DecodePolicy` | Sealed policy trait implemented by the built-in policy markers |

## Crate Boundary

`qubit-codec-binary` only contains buffer-level binary codecs. Use
`qubit-codec` for shared core traits, `qubit-io` for generic `std::io` helpers,
and `qubit-io-binary` for stream-oriented binary readers and writers.

## Performance Considerations

`BinaryCodec`, `Leb128Codec`, and `ZigZagCodec` are zero-sized codec types with
no runtime allocation. Their unchecked methods and `Codec` implementations with
`Unit = u8` are intended for validated hot paths where a caller has already
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

- `qubit-codec` provides shared codec and byte-order primitives.
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
