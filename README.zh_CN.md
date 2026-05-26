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
- 从 `qubit-codec` 重导出的 `Codec`、`ByteOrder`、`BigEndian`、
  `LittleEndian` 和 `Coder` core primitive。

## 设计目标

- **缓冲区优先**：直接操作调用方持有的 byte slice，不要求 `Read` 或 `Write`。
- **热路径效率**：为已经验证过边界的调用方提供 unchecked 静态 codec 方法，并
  实现 `Codec<Value, u8>` 以接入通用 codec pipeline。
- **分层清晰**：只依赖 `qubit-codec`，stream adapter 交给 `qubit-io-binary`。
- **规范编码**：始终写出 canonical LEB128，同时允许配置 decode strictness。
- **强类型字节序**：同时支持运行时和类型级 endian 选择。
- **依赖图小**：让低层二进制线格式代码不必拉入通用 I/O 工具。

## 特性

### Fixed-Width Binary Scalar

- **整数覆盖**：支持基础 signed / unsigned integer 类型的编码和解码。
- **字节序支持**：支持 `BigEndian` 与 `LittleEndian` 类型标记。
- **Unchecked 热路径**：调用方验证容量后，可用 `read_unchecked` 和
  `write_unchecked` 避免重复边界检查。

### LEB128 值

- **无符号值**：支持 `u8`、`u16`、`u32`、`u64`、`u128` 和 `usize`。
- **有符号值**：支持 `i8`、`i16`、`i32`、`i64`、`i128` 和 `isize`。
- **Strict Decode Policy**：`Strict` 拒绝非 canonical payload。
- **Non-Strict Decode Policy**：`NonStrict` 在 decoded value 可表达时接受兼容 payload。

### ZigZag 值

- **有符号整数映射**：把 signed integer 映射到 unsigned LEB128 payload。
- **Buffered Decode 支持**：提供 stream reader 使用的 partial-buffer decode 入口。

### 聚焦的公开 API

- **`prelude` 模块**：导入 binary codec 类型和核心字节序标记。
- **核心 codec trait**：`BinaryCodec`、`Leb128Codec` 和 `ZigZagCodec`
  实现 `qubit_codec::Codec<Value, u8>`。
- **不包含 `std::io` adapter**：stream helper 位于 `qubit-io-binary`。

## 文档

- [用户指南](doc/user_guide.zh_CN.md)
- [API 文档](https://docs.rs/qubit-codec-binary)
- [英文 README](README.md)

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
qubit-codec-binary = "0.1"
```

## 快速开始

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

## API 参考

### `BinaryCodec` 操作

| 条目 | 描述 |
|------|------|
| `Codec<Value, u8>` | 通过 core trait 解码和编码一个 fixed-width scalar |
| `REQUIRED_MIN_BUFFER_LEN` | 当前标量类型所需的最少字节数 |
| `read_unchecked(input, index)` | 无边界检查解码一个 fixed-width scalar |
| `write_unchecked(output, index, value)` | 无边界检查编码一个 fixed-width scalar |

### `Leb128Codec` 操作

| 条目 | 描述 |
|------|------|
| `Codec<Value, u8>` | 通过 core trait 解码和编码一个 LEB128 值 |
| `REQUIRED_MIN_BUFFER_LEN` | 当前整数类型最多需要的字节数 |
| `read_unchecked(input, index)` | 解码一个完整 LEB128 值 |
| `read_available_unchecked(input, index, available)` | 从当前可用的部分缓冲区尝试解码 |
| `write_unchecked(output, index, value)` | 编码一个 canonical LEB128 值 |

### `ZigZagCodec` 操作

| 条目 | 描述 |
|------|------|
| `Codec<Value, u8>` | 通过 core trait 解码和编码一个 ZigZag LEB128 值 |
| `REQUIRED_MIN_BUFFER_LEN` | 当前 signed integer 类型最多需要的字节数 |
| `read_unchecked(input, index)` | 解码 ZigZag over unsigned LEB128 |
| `read_available_unchecked(input, index, available)` | 从当前可用的部分缓冲区尝试解码 |
| `write_unchecked(output, index, value)` | 把 signed integer 编码为 ZigZag + unsigned LEB128 |

### Decode Policy

| Policy | 含义 |
|--------|------|
| `Strict` | 拒绝非 canonical LEB128 编码 |
| `NonStrict` | 在 decoded value 可表达时接受兼容编码 |

## 库边界

`qubit-codec-binary` 只包含缓冲区级 binary codec。共享 core trait 使用
`qubit-codec`，通用 `std::io` helper 使用 `qubit-io`，面向 stream 的 binary
reader/writer 使用 `qubit-io-binary`。

## 性能考虑

`BinaryCodec`、`Leb128Codec` 和 `ZigZagCodec` 都是零大小 codec 类型，不产生运行时分配。
它们的 unchecked 方法和 `Codec<Value, u8>` 实现面向已经验证缓冲区容量的热路径，或被
buffered stream adapter 在内部使用。

## 测试与代码覆盖率

本项目通过 `tests/` 下的集成测试覆盖二进制线格式行为。

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行覆盖率报告
./coverage.sh

# 生成文本格式报告
./coverage.sh text

# 对齐 CI 要求
./align-ci.sh

# 运行 CI 检查（格式化、clippy、测试、覆盖率、安全审计）
RS_CI_SKIP_TOOLCHAIN_UPDATE=1 ./ci-check.sh
```

## 依赖项

运行时依赖保持很少：

- `qubit-codec` 提供共享字节序和 coder 原语。
- `thiserror` 提供公共 LEB128 错误类型实现。

## 许可证

Copyright (c) 2026. Haixing Hu.

根据 Apache 许可证 2.0 版（"许可证"）授权；
除非遵守许可证，否则您不得使用此文件。
您可以在以下位置获取许可证副本：

    http://www.apache.org/licenses/LICENSE-2.0

除非适用法律要求或书面同意，否则根据许可证分发的软件
按"原样"分发，不附带任何明示或暗示的担保或条件。
有关许可证下的特定语言管理权限和限制，请参阅许可证。

完整的许可证文本请参阅 [LICENSE](LICENSE)。

## 贡献

欢迎贡献！请随时提交 Pull Request。

### 开发指南

- 保持本 crate 聚焦缓冲区级 binary codec。
- 为 canonical 和 non-canonical 线格式场景补测试。
- 为公开 API 和 safety contract 编写文档。
- 提交 PR 前确保所有检查通过。

## 作者

**胡海星**

## 相关项目

- [qubit-codec](https://github.com/qubit-ltd/rs-codec)：共享核心 codec trait 与字节序标记。
- [qubit-io-binary](https://github.com/qubit-ltd/rs-io-binary)：这些 binary codec 的 stream adapter。
- [qubit-io](https://github.com/qubit-ltd/rs-io)：通用 `std::io` helper。
- Qubit 旗下的更多 Rust 库发布在 GitHub 组织
  [qubit-ltd](https://github.com/qubit-ltd)。

---

仓库地址：[https://github.com/qubit-ltd/rs-codec-binary](https://github.com/qubit-ltd/rs-codec-binary)
