# Qubit Binary Codec

[![Rust CI](https://github.com/qubit-ltd/rs-codec-binary/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-codec-binary/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-codec-binary/coverage-badge.json)](https://qubit-ltd.github.io/rs-codec-binary/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-codec-binary.svg?color=blue)](https://crates.io/crates/qubit-codec-binary)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

面向 Rust 的缓冲区级 binary codec。

## 概述

Qubit Binary Codec 提供基于调用方管理 byte buffer 的低层 binary codec。它不
依赖 `std::io`；面向 stream 的 reader/writer adapter 位于 `qubit-io-binary`。

本库提供：

- 用于 fixed-width 标量编码/解码的 `BinaryCodec`。
- 用于 unsigned / signed LEB128 值的 `Leb128Codec`。
- 用于 ZigZag signed integer mapping 的 `ZigZagCodec`。
- `Strict` 和 `NonStrict` 解码策略。
- `Leb128DecodeError` 和 `Leb128DecodeErrorKind`。
- 从 `qubit-codec` 重导出的 `ByteOrder`、`BigEndian`、`LittleEndian` 和
  `Coder` core primitive。

## 安装

```toml
[dependencies]
qubit-codec-binary = "0.1"
```

## 快速示例

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

## 库边界

`qubit-codec-binary` 只包含缓冲区级 binary codec。共享 core trait 使用
`qubit-codec`，通用 `std::io` helper 使用 `qubit-io`，面向 stream 的 binary
reader/writer 使用 `qubit-io-binary`。

## 开发

```bash
./align-ci.sh
RS_CI_SKIP_TOOLCHAIN_UPDATE=1 ./ci-check.sh
```

## 许可证

根据 Apache License 2.0 授权。完整许可证文本见 [LICENSE](LICENSE)。
