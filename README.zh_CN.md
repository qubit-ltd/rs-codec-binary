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
- `Strict` 和 `NonStrict` sealed LEB128 解码策略。
- 内置 LEB128 policy marker 使用的 `Leb128DecodePolicy`。
- `Leb128DecodeError` 和 `Leb128DecodeErrorKind`。
- binary codec 表面使用的必要 `qubit-codec` 原语：`Codec`、`ByteOrder`、
  `ByteOrderSpec`、`BigEndian` 和 `LittleEndian`。

## 设计目标

- **缓冲区优先**：直接操作调用方持有的 byte slice，不要求 `Read` 或 `Write`。
- **热路径效率**：为已经验证过边界的调用方提供 unchecked 静态 codec 方法，并
  实现 `Unit = u8` 的 `Codec` 以接入通用 codec pipeline。
- **分层清晰**：只依赖 `qubit-codec`，stream adapter 交给 `qubit-io-binary`。
- **规范编码**：始终写出 canonical LEB128，同时允许配置 decode strictness。
- **强类型字节序**：通过类型级 byte-order marker 选择 endian 行为。
- **依赖图小**：让低层二进制线格式代码不必拉入通用 I/O 工具。

## 特性

### Fixed-Width Binary Scalar

- **整数覆盖**：支持明确位宽的 primitive integer 类型：`u8`、`i8`、
  `u16`、`i16`、`u32`、`i32`、`u64`、`i64`、`u128` 和 `i128`。
  平台相关位宽的 `usize` 和 `isize` 不作为 fixed-width binary scalar 支持。
- **浮点覆盖**：支持 `f32` 和 `f64`，并保留 IEEE 754 bit pattern。
- **字节序支持**：支持 `BigEndian` 与 `LittleEndian` 类型标记。
- **Unchecked 热路径**：调用方验证容量后，可用 `decode_unchecked` 和
  `encode_unchecked` 避免重复边界检查。

### LEB128 值

- **无符号值**：支持 `u8`、`u16`、`u32`、`u64`、`u128` 和 `usize`。
- **有符号值**：支持 `i8`、`i16`、`i32`、`i64`、`i128` 和 `isize`。
- **Strict Decode Policy**：`Strict` 拒绝非 canonical payload。
- **Non-Strict Decode Policy**：`NonStrict` 在 decoded value 可表达时接受兼容 payload。
- **Sealed Policy Trait**：`Leb128DecodePolicy` 刻意封闭，只使用内置
  `Strict` 或 `NonStrict` marker。

### ZigZag 值

- **有符号整数映射**：把 signed integer 映射到 unsigned LEB128 payload。
- **不完整输入报告**：通过 `Leb128DecodeError` 表达 partial LEB128 payload。

### 聚焦的公开 API

- **`prelude` 模块**：导入 binary codec 类型和核心字节序标记。
- **核心 codec trait**：`BinaryCodec`、`Leb128Codec` 和 `ZigZagCodec`
  实现 `qubit_codec::Codec`，其中 `Unit = u8`，`Value` 由具体 codec 决定。
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

let mut fixed = [0_u8; BinaryCodec::<u32, BigEndian>::MAX_UNITS_PER_VALUE];
unsafe {
    BinaryCodec::<u32, BigEndian>::encode_unchecked(0x0102_0304, &mut fixed, 0);
}
assert_eq!([1, 2, 3, 4], fixed);

let mut compact = [0_u8; Leb128Codec::<u64, NonStrict>::MAX_UNITS_PER_VALUE];
let written = unsafe { Leb128Codec::<u64, NonStrict>::encode_unchecked(300, &mut compact, 0) };
assert_eq!(2, written);
```

## Unchecked API 契约

底层 codec 方法刻意设计为 unsafe。调用方必须先验证 buffer 边界：

- `BinaryCodec::decode_unchecked` 和 `BinaryCodec::encode_unchecked` 要求从
  `index` 开始分别有 `MIN_UNITS_PER_VALUE` 个可读字节或
  `MAX_UNITS_PER_VALUE` 个可写字节。对于 fixed-width 值，这两个边界相等。
- `Leb128Codec` 和 `ZigZagCodec` 暴露 `MIN_UNITS_PER_VALUE` 和
  `MAX_UNITS_PER_VALUE`。它们的 `encode_unchecked` 要求从 `index` 开始至少有
  `MAX_UNITS_PER_VALUE` 个可写字节，即使实际编码结果可能更短。
- `Leb128Codec::decode_unchecked` 和 `ZigZagCodec::decode_unchecked` 要求从
  `index` 开始至少有 `MIN_UNITS_PER_VALUE` 个可读字节。调用方通常应尽量提供到
  `MAX_UNITS_PER_VALUE`，除非 EOF 已经无法继续读取。不完整、畸形和非 canonical
  输入都通过 `Leb128DecodeError` 表达。
- `Leb128DecodeError::start_index()` 表示当前尝试解码的值从哪里开始；
  `error_index()` 表示错误在哪里变得可观察。对于 incomplete input，
  `error_index()` 是当前可用输入之后的边界，`additional()` 表示至少还需要多少
  字节才能继续推进解码。

上层代码应先完成这些边界检查，再把 unsafe 调用封装到安全 API 中。owned-value
adapter 和 buffered engine 请直接从 `qubit-codec` 引入；本 crate 不重导出通用
codec adapter。binary codec 示例见[用户指南](doc/user_guide.zh_CN.md)。

## API 参考

### `BinaryCodec` 操作

| 条目 | 描述 |
|------|------|
| `Codec` (`Unit = u8`) | 通过 core trait 解码和编码一个 fixed-width scalar |
| `MIN_UNITS_PER_VALUE` | 当前标量类型所需的最少字节数 |
| `MAX_UNITS_PER_VALUE` | 当前标量类型所需的最多字节数 |
| `decode_unchecked(input, index)` | 无边界检查解码一个 fixed-width scalar |
| `encode_unchecked(value, output, index)` | 无边界检查编码一个 fixed-width scalar |

### `Leb128Codec` 操作

| 条目 | 描述 |
|------|------|
| `Codec` (`Unit = u8`) | 通过 core trait 解码和编码一个 LEB128 值 |
| `MIN_UNITS_PER_VALUE` | 可能构成完整值的最少可读字节数 |
| `MAX_UNITS_PER_VALUE` | 当前整数类型最多需要的字节数 |
| `decode_unchecked(input, index)` | 解码一个完整 LEB128 值 |
| `encode_unchecked(value, output, index)` | 编码一个 canonical LEB128 值 |

### `ZigZagCodec` 操作

| 条目 | 描述 |
|------|------|
| `Codec` (`Unit = u8`) | 通过 core trait 解码和编码一个 ZigZag LEB128 值 |
| `MIN_UNITS_PER_VALUE` | 可能构成完整值的最少可读字节数 |
| `MAX_UNITS_PER_VALUE` | 当前 signed integer 类型最多需要的字节数 |
| `decode_unchecked(input, index)` | 解码 ZigZag over unsigned LEB128 |
| `encode_unchecked(value, output, index)` | 把 signed integer 编码为 ZigZag + unsigned LEB128 |

### LEB128 Decode Policy

| Policy | 含义 |
|--------|------|
| `Strict` | 拒绝非 canonical LEB128 编码 |
| `NonStrict` | 在 decoded value 可表达时接受兼容编码 |
| `Leb128DecodePolicy` | sealed policy trait，由内置 policy marker 实现 |

## 库边界

`qubit-codec-binary` 只包含缓冲区级 binary codec。共享 core trait 使用
`qubit-codec`，通用 `std::io` helper 使用 `qubit-io`，面向 stream 的 binary
reader/writer 使用 `qubit-io-binary`。

## 性能考虑

`BinaryCodec`、`Leb128Codec` 和 `ZigZagCodec` 都是零大小 codec 类型，不产生运行时分配。
它们的 unchecked 方法和 `Codec` 实现（`Unit = u8`）面向已经验证缓冲区容量的热路径，或被
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

- `qubit-codec` 提供共享 codec 和字节序原语。
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
