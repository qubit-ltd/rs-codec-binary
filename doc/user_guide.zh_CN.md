# Qubit Binary Codec 用户指南

`qubit-codec-binary` 提供缓冲区级 binary codec，适合 parser、二进制格式和
已经自行管理 buffer 的 stream adapter。

## 层次

- 使用 `BinaryCodec<T, O>` 处理 fixed-width 整数和浮点数。
- 使用 `Leb128Codec<T, P>` 处理 unsigned / signed LEB128 值。
- 当有符号值通常接近零、包括负数也要保持紧凑时，使用 `ZigZagCodec<T, P>`。
- 使用 `Strict` 拒绝非 canonical LEB128 payload，使用 `NonStrict` 做宽松解码。

本库从 `qubit-codec` 重导出 `Codec`、`CodecValueEncoder`、
`CodecBufferedEncoder`、`ValueEncoder`、`ValueDecoder`、`ByteOrder`、
`BigEndian`、`LittleEndian` 和 `Transcoder`。

## Fixed-Width 值

```rust
use qubit_codec_binary::{
    BigEndian,
    BinaryCodec,
};

let mut output = [0_u8; BinaryCodec::<u32, BigEndian>::REQUIRED_MIN_BUFFER_LEN];
unsafe {
    BinaryCodec::<u32, BigEndian>::encode_unchecked(0x0102_0304, &mut output, 0);
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
let written = unsafe { Leb128Codec::<u64, NonStrict>::encode_unchecked(300, &mut unsigned, 0) };
assert_eq!(2, written);

let mut signed = [0_u8; ZigZagCodec::<i64, NonStrict>::MAX_UNITS_PER_VALUE];
let written = unsafe { ZigZagCodec::<i64, NonStrict>::encode_unchecked(-42, &mut signed, 0) };
assert_eq!(1, written);
```

`MIN_UNITS_PER_VALUE` 适合用于判断是否可以开始解码。
`MAX_UNITS_PER_VALUE` 是容量上界，适合用于分配输出 buffer，或者在调用方无法
证明终止字节位置时保证最大可读范围。

## Unsafe 边界

这些 codec 是低层构件。unchecked 方法不负责发现 buffer 空间是否足够：

- Fixed-width `BinaryCodec` 的 decode 和 encode 调用要求从给定 index 开始有
  `REQUIRED_MIN_BUFFER_LEN` 个可读或可写字节。
- LEB128 和 ZigZag 的 encode 调用要求从给定 index 开始有
  `MAX_UNITS_PER_VALUE` 个可写字节。
- LEB128 和 ZigZag 的 decode 调用要求从给定 index 开始至少有
  `MIN_UNITS_PER_VALUE` 个可读字节。调用方还必须确保在不可读内存之前存在终止字节，
  或者直接提供 `MAX_UNITS_PER_VALUE` 个可读字节。
- 如果 EOF 导致调用方无法提供足够字节来完成一个变长值，上层应把该值作为
  malformed input 处理。

对外暴露安全 API 时，应先验证这些条件，再跨过 unsafe 边界。

## 用 ValueEncoder 包装

当安全 API 需要把一个借用值编码为 owned 输出对象时，使用 `ValueEncoder`。

```rust
use core::convert::Infallible;

use qubit_codec_binary::{
    Leb128Codec,
    NonStrict,
    ValueEncoder,
};

struct U64Leb128Encoder;

impl ValueEncoder<u64> for U64Leb128Encoder {
    type Output = Vec<u8>;
    type Error = Infallible;

    fn encode(&self, input: &u64) -> Result<Self::Output, Self::Error> {
        let mut output = vec![0_u8; Leb128Codec::<u64, NonStrict>::MAX_UNITS_PER_VALUE];
        let written = unsafe {
            Leb128Codec::<u64, NonStrict>::encode_unchecked(*input, &mut output, 0)
        };
        output.truncate(written);
        Ok(output)
    }
}
```

这个 wrapper 在调用 `encode_unchecked` 前分配最大可能输出长度，然后把结果截断到
实际写入长度。

## 用 ValueDecoder 包装

当安全 API 需要把一个借用输入对象解码为 owned 值时，使用 `ValueDecoder`。

```rust
use qubit_codec_binary::{
    Leb128Codec,
    Leb128DecodeError,
    NonStrict,
    ValueDecoder,
};

struct U64Leb128Decoder;

impl ValueDecoder<[u8]> for U64Leb128Decoder {
    type Output = u64;
    type Error = Leb128DecodeError;

    fn decode(&self, input: &[u8]) -> Result<Self::Output, Self::Error> {
        let (value, _consumed) = unsafe {
            Leb128Codec::<u64, NonStrict>::decode_unchecked(input, 0)?
        };
        Ok(value)
    }
}
```

这个 wrapper 直接调用 `decode_unchecked`，因为 `Leb128DecodeError` 自己会表达
不完整、畸形和非 canonical 输入。

## 用 Transcoder 包装

当安全 API 需要在调用方提供的 buffer 上批量处理一系列值，并在输出空间不足时返回
progress，使用 `Transcoder`。

```rust
use core::convert::Infallible;

use qubit_codec_binary::{
    Leb128Codec,
    NonStrict,
    TranscodeProgress,
    TranscodeStatus,
    Transcoder,
};

struct U64Leb128Transcoder;

impl Transcoder<u64, u8> for U64Leb128Transcoder {
    type Error = Infallible;

    fn max_output_len(&self, input_len: usize) -> Option<usize> {
        Some(input_len.saturating_mul(
            Leb128Codec::<u64, NonStrict>::MAX_UNITS_PER_VALUE,
        ))
    }

    fn transcode(
        &mut self,
        input: &[u64],
        input_index: usize,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        let mut read = 0;
        let mut written = 0;
        while input_index + read < input.len() {
            let cursor = output_index + written;
            let available = output.len().saturating_sub(cursor);
            let required = Leb128Codec::<u64, NonStrict>::MAX_UNITS_PER_VALUE;
            if available < required {
                return Ok(TranscodeProgress::new(
                    TranscodeStatus::NeedOutput {
                        output_index: cursor,
                        required,
                        available,
                    },
                    read,
                    written,
                ));
            }
            let value = input[input_index + read];
            let len = unsafe {
                Leb128Codec::<u64, NonStrict>::encode_unchecked(value, output, cursor)
            };
            read += 1;
            written += len;
        }
        Ok(TranscodeProgress::complete(read, written))
    }
}
```

这个 transcoder 在每次 unsafe encode 前检查输出容量；空间不足时返回
`NeedOutput`，而不是写入短 buffer。

如果需要围绕这些 codec 的 `std::io::Read` / `Write` adapter，请使用
`qubit-io-binary`。
