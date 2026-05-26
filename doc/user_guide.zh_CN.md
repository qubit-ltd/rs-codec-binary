# Qubit Binary Codec 用户指南

`qubit-codec-binary` 提供缓冲区级 binary codec，适合 parser、二进制格式和
已经自行管理 buffer 的 stream adapter。

## 层次

- 使用 `BinaryCodec<T, O>` 处理 fixed-width 整数和浮点数。
- 使用 `Leb128Codec<T, P>` 处理 unsigned / signed LEB128 值。
- 当有符号值通常接近零、包括负数也要保持紧凑时，使用 `ZigZagCodec<T, P>`。
- 使用 `Strict` 拒绝非 canonical LEB128 payload，使用 `NonStrict` 做宽松解码。

本库从 `qubit-codec` 重导出 `ByteOrder`、`BigEndian`、`LittleEndian` 和
`Coder`。

## Fixed-Width 值

```rust
use qubit_codec_binary::{
    BigEndian,
    BinaryCodec,
};

let mut output = [0_u8; BinaryCodec::<u32, BigEndian>::REQUIRED_MIN_BUFFER_LEN];
unsafe {
    BinaryCodec::<u32, BigEndian>::write_unchecked(&mut output, 0, 0x0102_0304);
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

let mut unsigned = [0_u8; Leb128Codec::<u64, NonStrict>::REQUIRED_MIN_BUFFER_LEN];
let written = unsafe { Leb128Codec::<u64, NonStrict>::write_unchecked(&mut unsigned, 0, 300) };
assert_eq!(2, written);

let mut signed = [0_u8; ZigZagCodec::<i64, NonStrict>::REQUIRED_MIN_BUFFER_LEN];
let written = unsafe { ZigZagCodec::<i64, NonStrict>::write_unchecked(&mut signed, 0, -42) };
assert_eq!(1, written);
```

如果需要围绕这些 codec 的 `std::io::Read` / `Write` adapter，请使用
`qubit-io-binary`。
