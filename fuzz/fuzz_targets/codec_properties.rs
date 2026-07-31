// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![no_main]

use libfuzzer_sys::fuzz_target;
use qubit_codec::{
    BigEndian,
    LittleEndian,
};
use qubit_codec_binary::{
    BinaryCodec,
    Leb128Codec,
    Leb128DecodeErrorKind,
    NonStrict,
    Strict,
    ZigZagCodec,
};

/// Bounds each invocation independently of the fuzzer configuration.
const MAX_FUZZ_INPUT_LEN: usize = 19;

macro_rules! assert_decode_policies {
    ($codec:ident, $ty:ty, $input:expr) => {{
        let non_strict = unsafe { $codec::<$ty, NonStrict>::decode($input, 0) };
        let strict = unsafe { $codec::<$ty, Strict>::decode($input, 0) };
        assert_strict_success_is_non_strict_success(strict, non_strict);
    }};
}

macro_rules! assert_leb128_roundtrip {
    ($codec:ident, $ty:ty, $value:expr) => {{
        let expected: $ty = $value;
        let mut output = [0_u8; $codec::<$ty, NonStrict>::MAX_UNITS_PER_VALUE];
        let written = unsafe {
            $codec::<$ty, NonStrict>::encode(expected, &mut output, 0)
        };
        let strict = unsafe {
            $codec::<$ty, Strict>::decode(&output[..written], 0)
        }
        .expect("canonical LEB128-family encoding must pass strict decoding");
        let non_strict = unsafe {
            $codec::<$ty, NonStrict>::decode(&output[..written], 0)
        }
        .expect(
            "canonical LEB128-family encoding must pass non-strict decoding",
        );

        let (strict_value, strict_consumed) = strict;
        let (non_strict_value, non_strict_consumed) = non_strict;
        assert_eq!((expected, written), (strict_value, strict_consumed.get()));
        assert_eq!(
            (expected, written),
            (non_strict_value, non_strict_consumed.get())
        );
    }};
}

macro_rules! assert_binary_roundtrip {
    ($ty:ty, $order:ty, $value:expr) => {{
        let expected: $ty = $value;
        let mut output =
            [0xA5_u8; BinaryCodec::<$ty, $order>::MAX_UNITS_PER_VALUE + 2];
        let written = unsafe {
            BinaryCodec::<$ty, $order>::encode(expected, &mut output, 1)
        };
        let (actual, consumed) =
            unsafe { BinaryCodec::<$ty, $order>::decode(&output, 1) };
        assert_eq!(BinaryCodec::<$ty, $order>::MAX_UNITS_PER_VALUE, written);
        assert_eq!(expected, actual);
        assert_eq!(written, consumed.get());
        assert_eq!(0xA5, output[0]);
        assert_eq!(0xA5, output[written + 1]);
    }};
}

macro_rules! assert_binary_float_roundtrip {
    ($ty:ty, $order:ty, $value:expr) => {{
        let expected: $ty = $value;
        let mut output =
            [0xA5_u8; BinaryCodec::<$ty, $order>::MAX_UNITS_PER_VALUE + 2];
        let written = unsafe {
            BinaryCodec::<$ty, $order>::encode(expected, &mut output, 1)
        };
        let (actual, consumed) =
            unsafe { BinaryCodec::<$ty, $order>::decode(&output, 1) };
        assert_eq!(BinaryCodec::<$ty, $order>::MAX_UNITS_PER_VALUE, written);
        assert_eq!(expected.to_bits(), actual.to_bits());
        assert_eq!(written, consumed.get());
        assert_eq!(0xA5, output[0]);
        assert_eq!(0xA5, output[written + 1]);
    }};
}

fuzz_target!(|data: &[u8]| {
    let input = &data[..data.len().min(MAX_FUZZ_INPUT_LEN)];

    decode_arbitrary_input(input);
    assert_noncanonical_policy_behavior();

    let bits = fuzz_u128(input);
    assert_leb128_roundtrips(bits);
    assert_binary_roundtrips(bits);
});

/// Builds a deterministic integer from at most sixteen fuzz input bytes.
fn fuzz_u128(input: &[u8]) -> u128 {
    let mut bytes = [0_u8; size_of::<u128>()];
    for (output, source) in bytes.iter_mut().zip(input.iter().copied()) {
        *output = source;
    }
    u128::from_le_bytes(bytes)
}

/// Exercises both policies on arbitrary non-empty input without violating the
/// unchecked decoder precondition.
fn decode_arbitrary_input(input: &[u8]) {
    if input.is_empty() {
        return;
    }

    assert_decode_policies!(Leb128Codec, u8, input);
    assert_decode_policies!(Leb128Codec, u16, input);
    assert_decode_policies!(Leb128Codec, u32, input);
    assert_decode_policies!(Leb128Codec, u64, input);
    assert_decode_policies!(Leb128Codec, u128, input);
    assert_decode_policies!(Leb128Codec, usize, input);
    assert_decode_policies!(Leb128Codec, i8, input);
    assert_decode_policies!(Leb128Codec, i16, input);
    assert_decode_policies!(Leb128Codec, i32, input);
    assert_decode_policies!(Leb128Codec, i64, input);
    assert_decode_policies!(Leb128Codec, i128, input);
    assert_decode_policies!(Leb128Codec, isize, input);
    assert_decode_policies!(ZigZagCodec, i8, input);
    assert_decode_policies!(ZigZagCodec, i16, input);
    assert_decode_policies!(ZigZagCodec, i32, input);
    assert_decode_policies!(ZigZagCodec, i64, input);
    assert_decode_policies!(ZigZagCodec, i128, input);
    assert_decode_policies!(ZigZagCodec, isize, input);
}

/// Verifies that strict acceptance implies the same non-strict value and
/// consumed byte count.
fn assert_strict_success_is_non_strict_success<T, E>(
    strict: Result<(T, core::num::NonZeroUsize), E>,
    non_strict: Result<(T, core::num::NonZeroUsize), E>,
) where
    T: core::fmt::Debug + PartialEq,
    E: core::fmt::Debug,
{
    if let Ok((expected_value, expected_consumed)) = strict {
        let (actual_value, actual_consumed) = non_strict
            .expect("non-strict decoding must accept strict-valid input");
        assert_eq!(expected_value, actual_value);
        assert_eq!(expected_consumed, actual_consumed);
    }
}

/// Verifies roundtrips for every LEB128 and ZigZag supported width.
fn assert_leb128_roundtrips(bits: u128) {
    assert_leb128_roundtrip!(Leb128Codec, u8, bits as u8);
    assert_leb128_roundtrip!(Leb128Codec, u16, bits as u16);
    assert_leb128_roundtrip!(Leb128Codec, u32, bits as u32);
    assert_leb128_roundtrip!(Leb128Codec, u64, bits as u64);
    assert_leb128_roundtrip!(Leb128Codec, u128, bits);
    assert_leb128_roundtrip!(Leb128Codec, usize, bits as usize);
    assert_leb128_roundtrip!(Leb128Codec, i8, bits as i8);
    assert_leb128_roundtrip!(Leb128Codec, i16, bits as i16);
    assert_leb128_roundtrip!(Leb128Codec, i32, bits as i32);
    assert_leb128_roundtrip!(Leb128Codec, i64, bits as i64);
    assert_leb128_roundtrip!(Leb128Codec, i128, bits as i128);
    assert_leb128_roundtrip!(Leb128Codec, isize, bits as isize);
    assert_leb128_roundtrip!(ZigZagCodec, i8, bits as i8);
    assert_leb128_roundtrip!(ZigZagCodec, i16, bits as i16);
    assert_leb128_roundtrip!(ZigZagCodec, i32, bits as i32);
    assert_leb128_roundtrip!(ZigZagCodec, i64, bits as i64);
    assert_leb128_roundtrip!(ZigZagCodec, i128, bits as i128);
    assert_leb128_roundtrip!(ZigZagCodec, isize, bits as isize);
}

/// Verifies binary codec roundtrips at non-zero offsets for every wire type.
fn assert_binary_roundtrips(bits: u128) {
    assert_binary_roundtrip!(u8, BigEndian, bits as u8);
    assert_binary_roundtrip!(i8, LittleEndian, bits as i8);
    assert_binary_roundtrip!(u16, BigEndian, bits as u16);
    assert_binary_roundtrip!(u32, LittleEndian, bits as u32);
    assert_binary_roundtrip!(u64, BigEndian, bits as u64);
    assert_binary_roundtrip!(u128, LittleEndian, bits);
    assert_binary_roundtrip!(i16, LittleEndian, bits as i16);
    assert_binary_roundtrip!(i32, BigEndian, bits as i32);
    assert_binary_roundtrip!(i64, LittleEndian, bits as i64);
    assert_binary_roundtrip!(i128, BigEndian, bits as i128);
    assert_binary_float_roundtrip!(f32, BigEndian, f32::from_bits(bits as u32));
    assert_binary_float_roundtrip!(
        f64,
        LittleEndian,
        f64::from_bits(bits as u64)
    );
}

/// Verifies that only strict decoding rejects redundant valid encodings.
fn assert_noncanonical_policy_behavior() {
    let unsigned_non_strict =
        unsafe { Leb128Codec::<u64, NonStrict>::decode(&[0x80, 0x00], 0) }
            .expect("non-strict unsigned decoding must accept redundant zero");
    let (unsigned_value, unsigned_consumed) = unsigned_non_strict;
    assert_eq!((0, 2), (unsigned_value, unsigned_consumed.get()));
    let unsigned_strict =
        unsafe { Leb128Codec::<u64, Strict>::decode(&[0x80, 0x00], 0) }
            .expect_err("strict unsigned decoding must reject redundant zero");
    assert_eq!(Leb128DecodeErrorKind::NonCanonical, unsigned_strict.kind());

    let signed_non_strict =
        unsafe { Leb128Codec::<i64, NonStrict>::decode(&[0xff, 0x7f], 0) }
            .expect(
                "non-strict signed decoding must accept redundant negative one",
            );
    let (signed_value, signed_consumed) = signed_non_strict;
    assert_eq!((-1, 2), (signed_value, signed_consumed.get()));
    let signed_strict =
        unsafe { Leb128Codec::<i64, Strict>::decode(&[0xff, 0x7f], 0) }
            .expect_err(
                "strict signed decoding must reject redundant negative one",
            );
    assert_eq!(Leb128DecodeErrorKind::NonCanonical, signed_strict.kind());

    let zig_zag_non_strict =
        unsafe { ZigZagCodec::<i64, NonStrict>::decode(&[0x80, 0x00], 0) }
            .expect("non-strict ZigZag decoding must accept redundant zero");
    let (zig_zag_value, zig_zag_consumed) = zig_zag_non_strict;
    assert_eq!((0, 2), (zig_zag_value, zig_zag_consumed.get()));
    let zig_zag_strict =
        unsafe { ZigZagCodec::<i64, Strict>::decode(&[0x80, 0x00], 0) }
            .expect_err("strict ZigZag decoding must reject redundant zero");
    assert_eq!(Leb128DecodeErrorKind::NonCanonical, zig_zag_strict.kind());
}
