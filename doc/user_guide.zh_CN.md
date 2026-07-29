# Qubit Binary Codec 用户指南

`qubit-codec-binary` 提供缓冲区级 binary codec，适合 parser、二进制格式和
已经自行管理 buffer 的 stream adapter。

## 层次

- 使用 `BinaryCodec<T, O>` 处理明确位宽的 primitive integer 和浮点数。
- 使用 `Leb128Codec<T, P>` 处理 unsigned / signed LEB128 值。
- 当有符号值通常接近零、包括负数也要保持紧凑时，使用 `ZigZagCodec<T, P>`。
- 使用 `Strict` 拒绝非 canonical LEB128 payload，使用 `NonStrict` 做宽松解码。

本库暴露 sealed `Leb128DecodePolicy` trait，用于内置 LEB128 policy marker。
byte-order marker、通用 adapter、engine、hook 和 value trait 请直接从
`qubit-codec` 引入。

## Fixed-Width 值

`BinaryCodec` 支持 `u8`、`i8`、`u16`、`i16`、`u32`、`i32`、`u64`、`i64`、
`u128`、`i128`、`f32` 和 `f64`。它不实现 `usize` 或 `isize`，因为它们的
字节宽度依赖目标平台。

```rust
use qubit_codec::BigEndian;
use qubit_codec_binary::BinaryCodec;

let mut output = [0_u8; BinaryCodec::<u32, BigEndian>::MAX_UNITS_PER_VALUE];
unsafe {
    BinaryCodec::<u32, BigEndian>::encode(0x0102_0304, &mut output, 0);
}
assert_eq!([1, 2, 3, 4], output);
```

这些 unchecked API 面向调用方已经验证过 buffer 容量的热路径。

## LEB128 与 ZigZag

```rust
use qubit_codec_binary::{
    Leb128Codec,
    NonStrict,
    ZigZagCodec,
};

let mut unsigned = [0_u8; Leb128Codec::<u64, NonStrict>::MAX_UNITS_PER_VALUE];
let written = unsafe { Leb128Codec::<u64, NonStrict>::encode(300, &mut unsigned, 0) };
assert_eq!(2, written);

let mut signed = [0_u8; ZigZagCodec::<i64, NonStrict>::MAX_UNITS_PER_VALUE];
let written = unsafe { ZigZagCodec::<i64, NonStrict>::encode(-42, &mut signed, 0) };
assert_eq!(1, written);
```

`MIN_UNITS_PER_VALUE` 适合用于判断是否可以开始解码。
`MAX_UNITS_PER_VALUE` 是容量上界，适合用于分配输出 buffer，或者在调用方无法
证明终止字节位置时保证最大可读范围。

LEB128 的 `usize` 和 `isize` 使用当前 Rust target 的 pointer width。持久化文件和
跨平台协议应使用 `u64`、`i64` 等 fixed-width 类型。

## LEB128 解码错误

`Leb128DecodeError` 区分当前值的起点和错误被发现的位置：

- `start_index()` 返回当前尝试解码的值从哪个字节开始。
- `error_index()` 返回解码发现错误的位置。对于 incomplete input，它是当前可用
  输入之后、下一字节应出现的边界。
- `required()`、`available()` 和 `additional()` 描述 incomplete input。
- `consumed()` 描述 malformed 或 non-canonical input 后调用方可以消费多少
  无效字节。

## Unsafe 边界

这些 codec 是低层构件。unchecked 方法不负责发现 buffer 空间是否足够：

- Fixed-width `BinaryCodec` 的 decode 和 encode 调用要求从给定 index 开始有
  `MIN_UNITS_PER_VALUE` 个可读字节或 `MAX_UNITS_PER_VALUE` 个可写字节。
  对于 fixed-width 值，这两个边界相等。
- LEB128 和 ZigZag 的 encode 调用要求从给定 index 开始有
  `MAX_UNITS_PER_VALUE` 个可写字节。
- LEB128 和 ZigZag 的 decode 调用要求从给定 index 开始至少有
  `MIN_UNITS_PER_VALUE` 个可读字节。调用方通常应尽量提供到
  `MAX_UNITS_PER_VALUE`，除非 EOF 已经无法继续读取。
- 如果 EOF 导致调用方无法提供足够字节来完成一个变长值，上层应把该值作为
  `Leb128DecodeError` 中的不完整值处理。

对外暴露安全 API 时，应先验证这些条件，再跨过 unsafe 边界。

## 安全 Adapter 与 Stream

本 crate 刻意停留在 buffer-level binary codec 层。owned single-value adapter、
通用 buffered engine 或 conversion trait 请直接从 `qubit-codec` 引入，并把这些
binary codec 作为低层 codec 实现使用。

如果自行封装 `decode`，调用 unsafe 方法前要先检查起始 index 后至少有
`MIN_UNITS_PER_VALUE` 个可读字节。对于 single-value decoder，还需要由你的格式决定
返回的 `consumed` 之后是否允许 trailing bytes。

如果需要围绕这些 codec 的 `std::io::Read` / `Write` adapter，请使用
`qubit-io-binary`。
