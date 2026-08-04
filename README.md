# Qubit Binary Codec

[![Rust CI](https://github.com/qubit-ltd/rs-codec-binary/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-codec-binary/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-codec-binary/coverage-badge.json)](https://qubit-ltd.github.io/rs-codec-binary/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-codec-binary.svg?color=blue)](https://crates.io/crates/qubit-codec-binary)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

`qubit-codec-binary` supplies buffer-oriented codecs for fixed-width scalars,
LEB128 integers, and ZigZag values. It is for protocol and file-format code
that owns its byte buffers and needs exact wire bytes without a `std::io`
reader or writer in the codec layer.

## Installation

```toml
[dependencies]
qubit-codec-binary = "0.3"
qubit-codec = "0.11"
```

## Quick Start

Encode a fixed-width field and a compact integer into caller-owned buffers:

```rust
use qubit_codec::BigEndian;
use qubit_codec_binary::{BinaryCodec, Leb128Codec, NonStrict};

let mut fixed = [0_u8; BinaryCodec::<u32, BigEndian>::MAX_ENCODE_UNITS_PER_VALUE];
let fixed_len = unsafe {
    BinaryCodec::<u32, BigEndian>::encode(0x0102_0304, &mut fixed, 0)
};
assert_eq!(4, fixed_len);
assert_eq!([0x01, 0x02, 0x03, 0x04], fixed);

let mut compact = [0_u8; Leb128Codec::<u64, NonStrict>::MAX_ENCODE_UNITS_PER_VALUE];
let compact_len = unsafe {
    Leb128Codec::<u64, NonStrict>::encode(300, &mut compact, 0)
};
assert_eq!(&[0xac, 0x02], &compact[..compact_len]);
```

For ordinary one-value conversion, prefer the checked adapters from
`qubit-codec`; they own the temporary buffer and keep the unsafe boundary
inside the adapter:

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

The low-level `encode` and `decode` methods are `unsafe`: the caller must
ensure the documented readable or writable capacity before calling them.
Use them directly only when the surrounding protocol code already establishes
those bounds or when a measured hot path justifies the unchecked boundary.

## Why This Project Exists

Binary protocols often need both deterministic fixed-width fields and compact
integers, but buffering and stream ownership belong to a higher layer. This
crate keeps the wire-format layer small and explicit, while
`qubit-io-binary` provides stream-oriented adapters.

## What It Provides

| Capability | Public API | Boundary |
| --- | --- | --- |
| Fixed-width integers and floats | `BinaryCodec<T, BigEndian>` / `BinaryCodec<T, LittleEndian>` / `BinaryCodec<T, NativeEndian>` | Supports explicit-width integer types plus `f32` and `f64`. Use big- or little-endian encoding for persistent and cross-platform data; native-endian encoding is only for platform-local data. It does not define a persistent width for `usize` or `isize`. |
| Compact integers | `Leb128Codec<T, P>` | Encoding is canonical; `Strict` rejects non-canonical input and `NonStrict` accepts compatible input. |
| Compact signed integers | `ZigZagCodec<T, P>` | Maps signed values to unsigned LEB128 payloads. |
| Decode diagnostics | `Leb128DecodeError` | Distinguishes incomplete, malformed, and non-canonical input with indices and available/required counts. |

This crate does not provide `std::io` readers or writers, generic owned-value
adapters, framing, or buffering. Import shared traits and byte-order markers
from `qubit-codec`, and use `qubit-io-binary` when stream adapters are needed.

`Leb128Codec<T, P>` and `ZigZagCodec<T, P>` require an explicit decoding
policy. `P` never affects canonical encoding; encoding-only code should use
`NonStrict` as the conventional marker.

`NonStrict` accepts non-canonical representations only within the target
integer's declared maximum width. Longer unterminated or over-width payloads
remain malformed rather than being accepted as arbitrarily long encodings.

## Learn More

- [User guide](doc/user_guide.md)
- [API reference](https://docs.rs/qubit-codec-binary)
- [中文 README](README.zh_CN.md)
- [中文用户指南](doc/user_guide.zh_CN.md)

## Testing

```bash
# Run tests with the default feature set
cargo test

# Run tests with all declared features
cargo test --all-features

# Project CI checks
./ci-check.sh

# Check code coverage
./coverage.sh
```

## License

Copyright (c) 2025 - 2026. Haixing Hu. All rights reserved.

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for the
full license text.

## Contributing

Contributions are welcome. Please follow the Rust API guidelines, keep public
API documentation and tests current, and run `./align-ci.sh` to format code and
`./ci-check.sh` to satisfy CI requirements before submitting a pull request.

## Author

**Haixing Hu** - *Qubit Co. Ltd.*

Repository: [https://github.com/qubit-ltd/rs-codec-binary](https://github.com/qubit-ltd/rs-codec-binary)
