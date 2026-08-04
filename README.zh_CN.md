# Qubit Binary Codec

[![Rust CI](https://github.com/qubit-ltd/rs-codec-binary/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-codec-binary/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-codec-binary/coverage-badge.json)](https://qubit-ltd.github.io/rs-codec-binary/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-codec-binary.svg?color=blue)](https://crates.io/crates/qubit-codec-binary)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

`qubit-codec-binary` 提供面向调用方缓冲区的定长标量、LEB128 整数和 ZigZag
编解码器。它适用于需要精确控制线格式字节、但不希望在编解码层引入 `std::io`
读写器的协议与文件格式实现。

## 安装

```toml
[dependencies]
qubit-codec-binary = "0.3"
qubit-codec = "0.11"
```

## 快速开始

把定长字段和紧凑整数写入调用方持有的缓冲区：

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

普通的单值转换建议优先使用 `qubit-codec` 提供的检查型 adapter；它们管理临时
缓冲区，并把 unsafe 边界封装在 adapter 内部：

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
let encoded = encoder.encode(&300).expect("u64 始终可以编码");

let mut decoder =
    CodecValueDecoder::new(Leb128Codec::<u64, NonStrict>::default());
let decoded = decoder.decode(&encoded).expect("编码结果有效");
assert_eq!(300, decoded);
```

底层 `encode` 与 `decode` 方法是 `unsafe`：调用前必须满足文档规定的可读或
可写容量条件。只有当外围协议代码已经建立这些边界，或经过测量确认热路径确有
需要时，才应直接调用它们。

## 为什么需要这个项目

二进制协议常同时需要确定宽度的字段和紧凑整数，而缓冲、流所有权应属于更高层。
本库保持线格式层小而明确；面向流的适配器由 `qubit-io-binary` 提供。

## 核心能力

| 能力 | 公开 API | 边界 |
| --- | --- | --- |
| 定长整数与浮点数 | `BinaryCodec<T, BigEndian>` / `BinaryCodec<T, LittleEndian>` / `BinaryCodec<T, NativeEndian>` | 支持明确位宽的整数及 `f32`、`f64`。持久化和跨平台数据应使用 big- 或 little-endian；native-endian 仅适用于平台本地数据。不为 `usize`、`isize` 定义持久化宽度。 |
| 紧凑整数 | `Leb128Codec<T, P>` | 编码始终规范化；`Strict` 拒绝非规范输入，`NonStrict` 接受兼容输入。 |
| 紧凑有符号整数 | `ZigZagCodec<T, P>` | 将有符号值映射为无符号 LEB128 负载。 |
| 解码诊断 | `Leb128DecodeError` | 区分不完整、畸形和非规范输入，并提供下标及可用/所需数量。 |

本库不提供 `std::io` reader/writer、通用 owned-value adapter、分帧或缓冲。共享
trait 与字节序标记请从 `qubit-codec` 导入；需要流适配器时使用
`qubit-io-binary`。

`Leb128Codec<T, P>` 与 `ZigZagCodec<T, P>` 必须显式指定解码策略。`P`
不会影响规范化编码；仅编码的代码应按约定使用 `NonStrict` 作为标记。

`NonStrict` 只接受目标整数声明的最大宽度以内的非规范表示。超过该宽度的
未终止或过宽负载仍会被视为畸形输入，不会接受任意长度的编码。

## 延伸阅读

- [用户指南](doc/user_guide.zh_CN.md)
- [API 文档](https://docs.rs/qubit-codec-binary)
- [英文 README](README.md)
- [English user guide](doc/user_guide.md)

## 测试

```bash
# 使用默认 feature 集运行测试
cargo test

# 使用项目声明的全部 feature 运行测试
cargo test --all-features

# 运行项目 CI 检查
./ci-check.sh

# 检查代码覆盖率
./coverage.sh
```

## 许可证

Copyright (c) 2025 - 2026. Haixing Hu. All rights reserved.

本项目基于 Apache License 2.0 授权。完整许可证文本请参阅
[LICENSE](LICENSE)。

## 贡献

欢迎贡献。请遵循 Rust API 指南，及时更新公共 API 文档与测试，并在提交
Pull Request 前运行 `./align-ci.sh`格式化代码，运行`./ci-check.sh`对齐CI要求。

## 作者

**Haixing Hu** - *Qubit Co. Ltd.*

仓库地址：[https://github.com/qubit-ltd/rs-codec-binary](https://github.com/qubit-ltd/rs-codec-binary)
