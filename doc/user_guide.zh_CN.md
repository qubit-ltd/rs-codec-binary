# Qubit Binary Codec 用户指南

[English](user_guide.md) · [README](../README.zh_CN.md) · [API 文档](https://docs.rs/qubit-codec-binary)

本文适用于 `qubit-codec-binary` 0.3，面向实现字节缓冲区协议或文件格式的 Rust
开发者：既需要定长字段和 LEB128，又需要自行掌控缓冲与 I/O。

## 概念模型

本库只处理一个边界：一个值与调用方管理的字节 slice 之间的转换。

```text
应用缓冲区 -> codec<T, policy> -> 值或线格式字节
                     |
                     +-> 畸形、不完整、非规范输入诊断
```

`BinaryCodec<T, O>` 使用类型级字节序处理定长值；`Leb128Codec<T, P>`
处理有符号或无符号变长整数；`ZigZagCodec<T, P>` 先将有符号值做 ZigZag
映射，再按无符号 LEB128 编码。它们都实现 `qubit_codec::Codec`，且
`Unit = u8`。

多字节 `BinaryCodec` 支持 `BigEndian`、`LittleEndian` 与 `NativeEndian`。
持久化或跨平台数据应使用 big- 或 little-endian。native-endian 仅适合由同一
平台类别读回的本地数据。

## 贯穿场景：解码紧凑记录

假定一条记录由 big-endian `u32` 标识符和一个有符号 ZigZag LEB128 增量组成。
编码端在进入 unsafe 层前预留各 codec 声明的最大空间；解码端提供完整记录 slice。

## 安装与最小配置

```toml
[dependencies]
qubit-codec-binary = "0.3"
qubit-codec = "0.10"
```

字节序标记与共享 `Codec` trait 请从 `qubit-codec` 导入。

## 核心流程

```rust
use qubit_codec::BigEndian;
use qubit_codec_binary::{BinaryCodec, Strict, ZigZagCodec};

let mut record = [0_u8; 9];
let id_len = unsafe {
    BinaryCodec::<u32, BigEndian>::encode(0x0102_0304, &mut record, 0)
};
let delta_len = unsafe {
    ZigZagCodec::<i32, Strict>::encode(-42, &mut record, id_len)
};
assert_eq!(5, id_len + delta_len);

let (id, used_id) = unsafe { BinaryCodec::<u32, BigEndian>::decode(&record, 0) };
let (delta, used_delta) = unsafe {
    ZigZagCodec::<i32, Strict>::decode(&record, used_id.get())
}
.expect("编码端生成的是规范形式");

assert_eq!(0x0102_0304, id);
assert_eq!(-42, delta);
assert_eq!(id_len, used_id.get());
assert_eq!(delta_len, used_delta.get());
```

对 LEB128 与 ZigZag 而言，按 `MAX_ENCODE_UNITS_PER_VALUE` 分配是保守预留；`encode`
返回实际字节数。定长 codec 则总是读写其标量的确切宽度。
解码侧的有界缓冲区与类型宽度校验应使用 `MAX_DECODE_UNITS_PER_VALUE`。当前 binary
codec 的两个上限数值相同，但契约彼此独立。

## LEB128 策略与错误

编码始终产生规范形式。应根据线格式契约选择解码策略；每个
`Leb128Codec<T, P>` 与 `ZigZagCodec<T, P>` 实例都必须显式写出 `P`：

| 策略 | 适用场景 | 将无符号零写成 `80 00` 时 |
| --- | --- | --- |
| `Strict` | 格式要求唯一字节表示。 | 返回 `NonCanonical` 错误。 |
| `NonStrict` | 需要兼容旧数据或宽松输入。 | 解码为 `0`。 |

策略不会影响编码。仅编码的代码应按约定使用 `NonStrict` 实例化 codec。

`Leb128DecodeError` 区分 `Incomplete`、`Malformed` 与 `NonCanonical`。
`start_index()` 是尝试解码值的起点，`error_index()` 是错误可观察的位置；输入
不完整时，可使用 `required()`、`available()` 和 `additional()` 决定如何补充缓冲。

```rust
use qubit_codec_binary::{Leb128Codec, Leb128DecodeErrorKind, NonStrict};

let error = unsafe { Leb128Codec::<u16, NonStrict>::decode(&[0xac], 0) }
    .expect_err("只有续接字节时输入不完整");
assert_eq!(Leb128DecodeErrorKind::Incomplete, error.kind());
assert_eq!(Some(2), error.required().map(|count| count.get()));
assert_eq!(Some(1), error.available());
```

## Unsafe 边界与最佳实践

直接 codec 方法不会检查 slice 边界。调用前请确保：

1. 对 `BinaryCodec`，从下标起至少有 `MIN_UNITS_PER_VALUE` 可读字节，或
   `MAX_ENCODE_UNITS_PER_VALUE` 可写字节。
2. 对 LEB128 和 ZigZag 编码，预留 `MAX_ENCODE_UNITS_PER_VALUE` 个可写字节，再只保留
   返回长度所覆盖的前缀。
3. 对 LEB128 和 ZigZag 解码，至少提供一个可读字节；可以时应提供当前已缓冲的、
   不超过 `MAX_DECODE_UNITS_PER_VALUE` 的全部字节。
4. 持久化或跨平台格式不要使用 `usize`、`isize`，它们的边界依赖目标指针宽度；
   请使用固定宽度整数。

来自不可信缓冲区的数据应先经过检查型协议 reader/writer 再调用这些方法。需要面向
stream 的二进制 adapter 时使用 `qubit-io-binary`；本库不管理缓冲，也不映射到
`std::io::Error`。

## 延伸阅读

- [README](../README.zh_CN.md)
- [English user guide](user_guide.md)
- [API 文档](https://docs.rs/qubit-codec-binary)
