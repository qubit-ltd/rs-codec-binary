// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::{
    BigEndian,
    Codec,
    LittleEndian,
};
use qubit_codec_binary::BinaryCodec;

use super::assertions_tests::assert_decoded_eq;

#[test]
fn test_binary_codec_exposes_unit_bounds() {
    assert_eq!(1, BinaryCodec::<u8, BigEndian>::MIN_UNITS_PER_VALUE);
    assert_eq!(1, BinaryCodec::<u8, BigEndian>::MAX_UNITS_PER_VALUE);
    assert_eq!(1, BinaryCodec::<i8, LittleEndian>::MIN_UNITS_PER_VALUE);
    assert_eq!(1, BinaryCodec::<i8, LittleEndian>::MAX_UNITS_PER_VALUE);
    assert_eq!(2, BinaryCodec::<u16, BigEndian>::MIN_UNITS_PER_VALUE);
    assert_eq!(2, BinaryCodec::<u16, BigEndian>::MAX_UNITS_PER_VALUE);
    assert_eq!(4, BinaryCodec::<u32, LittleEndian>::MIN_UNITS_PER_VALUE);
    assert_eq!(4, BinaryCodec::<u32, LittleEndian>::MAX_UNITS_PER_VALUE);
    assert_eq!(8, BinaryCodec::<u64, BigEndian>::MIN_UNITS_PER_VALUE);
    assert_eq!(8, BinaryCodec::<u64, BigEndian>::MAX_UNITS_PER_VALUE);
    assert_eq!(16, BinaryCodec::<u128, LittleEndian>::MIN_UNITS_PER_VALUE);
    assert_eq!(16, BinaryCodec::<u128, LittleEndian>::MAX_UNITS_PER_VALUE);
    assert_eq!(4, BinaryCodec::<f32, BigEndian>::MIN_UNITS_PER_VALUE);
    assert_eq!(4, BinaryCodec::<f32, BigEndian>::MAX_UNITS_PER_VALUE);
    assert_eq!(8, BinaryCodec::<f64, LittleEndian>::MIN_UNITS_PER_VALUE);
    assert_eq!(8, BinaryCodec::<f64, LittleEndian>::MAX_UNITS_PER_VALUE);
}

#[test]
fn test_binary_codec_reads_from_explicit_index_unchecked() {
    let input = [0xaa, 0x12, 0x34, 0x56, 0x78, 0xbb];

    let decoded = unsafe { BinaryCodec::<u32, BigEndian>::decode(&input, 1) };
    assert_decoded_eq((0x1234_5678, 4), decoded);

    let decoded =
        unsafe { BinaryCodec::<u32, LittleEndian>::decode(&input, 1) };
    assert_decoded_eq((0x7856_3412, 4), decoded);
}

#[test]
fn test_binary_codec_writes_to_explicit_index_unchecked() {
    let mut output = [0xaa, 0, 0, 0, 0, 0xbb];

    unsafe {
        assert_eq!(
            4,
            BinaryCodec::<u32, BigEndian>::encode(0x1234_5678, &mut output, 1)
        );
    }
    assert_eq!([0xaa, 0x12, 0x34, 0x56, 0x78, 0xbb], output);

    unsafe {
        assert_eq!(
            4,
            BinaryCodec::<u32, LittleEndian>::encode(
                0x1234_5678,
                &mut output,
                1
            )
        );
    }
    assert_eq!([0xaa, 0x78, 0x56, 0x34, 0x12, 0xbb], output);
}

#[test]
fn test_binary_codec_roundtrips_integer_extremes_for_all_fixed_width_types() {
    macro_rules! assert_extreme_roundtrip {
        ($ty:ty, $value:expr) => {{
            let value: $ty = $value;

            let mut output =
                [0u8; BinaryCodec::<$ty, BigEndian>::MAX_UNITS_PER_VALUE];
            let written = unsafe {
                BinaryCodec::<$ty, BigEndian>::encode(value, &mut output, 0)
            };
            assert_eq!(output.len(), written);
            assert_eq!(value.to_be_bytes(), output);
            let decoded =
                unsafe { BinaryCodec::<$ty, BigEndian>::decode(&output, 0) };
            assert_decoded_eq((value, output.len()), decoded);

            let mut output =
                [0u8; BinaryCodec::<$ty, LittleEndian>::MAX_UNITS_PER_VALUE];
            let written = unsafe {
                BinaryCodec::<$ty, LittleEndian>::encode(value, &mut output, 0)
            };
            assert_eq!(output.len(), written);
            assert_eq!(value.to_le_bytes(), output);
            let decoded =
                unsafe { BinaryCodec::<$ty, LittleEndian>::decode(&output, 0) };
            assert_decoded_eq((value, output.len()), decoded);
        }};
    }

    assert_extreme_roundtrip!(u8, u8::MIN);
    assert_extreme_roundtrip!(u8, u8::MAX);
    assert_extreme_roundtrip!(i8, i8::MIN);
    assert_extreme_roundtrip!(i8, i8::MAX);
    assert_extreme_roundtrip!(u16, u16::MIN);
    assert_extreme_roundtrip!(u16, u16::MAX);
    assert_extreme_roundtrip!(i16, i16::MIN);
    assert_extreme_roundtrip!(i16, i16::MAX);
    assert_extreme_roundtrip!(u32, u32::MIN);
    assert_extreme_roundtrip!(u32, u32::MAX);
    assert_extreme_roundtrip!(i32, i32::MIN);
    assert_extreme_roundtrip!(i32, i32::MAX);
    assert_extreme_roundtrip!(u64, u64::MIN);
    assert_extreme_roundtrip!(u64, u64::MAX);
    assert_extreme_roundtrip!(i64, i64::MIN);
    assert_extreme_roundtrip!(i64, i64::MAX);
    assert_extreme_roundtrip!(u128, u128::MIN);
    assert_extreme_roundtrip!(u128, u128::MAX);
    assert_extreme_roundtrip!(i128, i128::MIN);
    assert_extreme_roundtrip!(i128, i128::MAX);
}

#[test]
fn test_binary_codec_preserves_f32_bit_patterns() {
    macro_rules! assert_f32_bit_roundtrip {
        ($bits:expr) => {{
            let bits: u32 = $bits;
            let value = f32::from_bits(bits);

            let mut output =
                [0u8; BinaryCodec::<f32, BigEndian>::MAX_UNITS_PER_VALUE];
            let written = unsafe {
                BinaryCodec::<f32, BigEndian>::encode(value, &mut output, 0)
            };
            assert_eq!(output.len(), written);
            assert_eq!(bits.to_be_bytes(), output);
            let (decoded, consumed) =
                unsafe { BinaryCodec::<f32, BigEndian>::decode(&output, 0) };
            assert_eq!(bits, decoded.to_bits());
            assert_eq!(output.len(), consumed.get());

            let mut output =
                [0u8; BinaryCodec::<f32, LittleEndian>::MAX_UNITS_PER_VALUE];
            let written = unsafe {
                BinaryCodec::<f32, LittleEndian>::encode(value, &mut output, 0)
            };
            assert_eq!(output.len(), written);
            assert_eq!(bits.to_le_bytes(), output);
            let (decoded, consumed) =
                unsafe { BinaryCodec::<f32, LittleEndian>::decode(&output, 0) };
            assert_eq!(bits, decoded.to_bits());
            assert_eq!(output.len(), consumed.get());
        }};
    }

    assert_f32_bit_roundtrip!(0x8000_0000);
    assert_f32_bit_roundtrip!(0x7f80_0000);
    assert_f32_bit_roundtrip!(0xff80_0000);
    assert_f32_bit_roundtrip!(0x7fc0_0123);
}

#[test]
fn test_binary_codec_preserves_f64_bit_patterns() {
    macro_rules! assert_f64_bit_roundtrip {
        ($bits:expr) => {{
            let bits: u64 = $bits;
            let value = f64::from_bits(bits);

            let mut output =
                [0u8; BinaryCodec::<f64, BigEndian>::MAX_UNITS_PER_VALUE];
            let written = unsafe {
                BinaryCodec::<f64, BigEndian>::encode(value, &mut output, 0)
            };
            assert_eq!(output.len(), written);
            assert_eq!(bits.to_be_bytes(), output);
            let (decoded, consumed) =
                unsafe { BinaryCodec::<f64, BigEndian>::decode(&output, 0) };
            assert_eq!(bits, decoded.to_bits());
            assert_eq!(output.len(), consumed.get());

            let mut output =
                [0u8; BinaryCodec::<f64, LittleEndian>::MAX_UNITS_PER_VALUE];
            let written = unsafe {
                BinaryCodec::<f64, LittleEndian>::encode(value, &mut output, 0)
            };
            assert_eq!(output.len(), written);
            assert_eq!(bits.to_le_bytes(), output);
            let (decoded, consumed) =
                unsafe { BinaryCodec::<f64, LittleEndian>::decode(&output, 0) };
            assert_eq!(bits, decoded.to_bits());
            assert_eq!(output.len(), consumed.get());
        }};
    }

    assert_f64_bit_roundtrip!(0x8000_0000_0000_0000);
    assert_f64_bit_roundtrip!(0x7ff0_0000_0000_0000);
    assert_f64_bit_roundtrip!(0xfff0_0000_0000_0000);
    assert_f64_bit_roundtrip!(0x7ff8_0000_0000_1234);
}

#[test]
fn test_binary_codec_encodes_and_decodes_through_codec_trait() {
    let mut codec = BinaryCodec::<u32, BigEndian>::default();
    let mut output = [0xaa, 0, 0, 0, 0, 0xbb];

    assert_eq!(
        4,
        <BinaryCodec<u32, BigEndian> as Codec>::MIN_UNITS_PER_VALUE,
    );
    assert_eq!(
        4,
        <BinaryCodec<u32, BigEndian> as Codec>::MAX_UNITS_PER_VALUE,
    );

    let written =
        unsafe { Codec::encode(&mut codec, &0x1234_5678, &mut output, 1) }
            .expect("fixed-width encoding should be infallible");
    assert_eq!(4, written);
    assert_eq!([0xaa, 0x12, 0x34, 0x56, 0x78, 0xbb], output);

    let (decoded, consumed) = unsafe { Codec::decode(&mut codec, &output, 1) }
        .expect("fixed-width decoding should be infallible");
    assert_eq!(0x1234_5678, decoded);
    assert_eq!(4, consumed.get());
}

#[test]
fn test_binary_codec_trait_covers_byte_and_little_endian_groups() {
    let mut unsigned_byte = BinaryCodec::<u8, BigEndian>::default();
    let mut signed_byte = BinaryCodec::<i8, LittleEndian>::default();
    let mut little_integer = BinaryCodec::<u16, LittleEndian>::default();
    let mut big_float = BinaryCodec::<f32, BigEndian>::default();
    let mut little_float = BinaryCodec::<f64, LittleEndian>::default();
    let mut output = [0u8; 24];

    assert_eq!(
        1,
        <BinaryCodec<u8, BigEndian> as Codec>::MIN_UNITS_PER_VALUE,
    );
    assert_eq!(
        1,
        <BinaryCodec<u8, BigEndian> as Codec>::MAX_UNITS_PER_VALUE,
    );
    assert_eq!(
        1,
        <BinaryCodec<i8, LittleEndian> as Codec>::MIN_UNITS_PER_VALUE,
    );
    assert_eq!(
        1,
        <BinaryCodec<i8, LittleEndian> as Codec>::MAX_UNITS_PER_VALUE,
    );
    assert_eq!(
        2,
        <BinaryCodec<u16, LittleEndian> as Codec>::MIN_UNITS_PER_VALUE,
    );
    assert_eq!(
        2,
        <BinaryCodec<u16, LittleEndian> as Codec>::MAX_UNITS_PER_VALUE,
    );
    assert_eq!(
        4,
        <BinaryCodec<f32, BigEndian> as Codec>::MIN_UNITS_PER_VALUE,
    );
    assert_eq!(
        4,
        <BinaryCodec<f32, BigEndian> as Codec>::MAX_UNITS_PER_VALUE,
    );
    assert_eq!(
        8,
        <BinaryCodec<f64, LittleEndian> as Codec>::MIN_UNITS_PER_VALUE,
    );
    assert_eq!(
        8,
        <BinaryCodec<f64, LittleEndian> as Codec>::MAX_UNITS_PER_VALUE,
    );

    assert_eq!(
        1,
        unsafe { Codec::encode(&mut unsigned_byte, &0x7f, &mut output, 0) }
            .expect("u8 encoding should be infallible")
    );
    assert_eq!(
        1,
        unsafe { Codec::encode(&mut signed_byte, &-1, &mut output, 1) }
            .expect("i8 encoding should be infallible")
    );
    assert_eq!(
        2,
        unsafe { Codec::encode(&mut little_integer, &0x1234, &mut output, 2) }
            .expect("little-endian integer encoding should be infallible")
    );
    assert_eq!(
        4,
        unsafe { Codec::encode(&mut big_float, &12.5, &mut output, 4) }
            .expect("big-endian float encoding should be infallible")
    );
    assert_eq!(
        8,
        unsafe { Codec::encode(&mut little_float, &-25.25, &mut output, 8) }
            .expect("little-endian float encoding should be infallible")
    );

    let (decoded, consumed) =
        unsafe { Codec::decode(&mut unsigned_byte, &output, 0) }
            .expect("u8 decoding should be infallible");
    assert_eq!(0x7f, decoded);
    assert_eq!(1, consumed.get());
    let (decoded, consumed) =
        unsafe { Codec::decode(&mut signed_byte, &output, 1) }
            .expect("i8 decoding should be infallible");
    assert_eq!(-1, decoded);
    assert_eq!(1, consumed.get());
    let (decoded, consumed) =
        unsafe { Codec::decode(&mut little_integer, &output, 2) }
            .expect("little-endian integer decoding should be infallible");
    assert_eq!(0x1234, decoded);
    assert_eq!(2, consumed.get());
    let (decoded, consumed) =
        unsafe { Codec::decode(&mut big_float, &output, 4) }
            .expect("big-endian float decoding should be infallible");
    assert_eq!(12.5, decoded);
    assert_eq!(4, consumed.get());
    let (decoded, consumed) =
        unsafe { Codec::decode(&mut little_float, &output, 8) }
            .expect("little-endian float decoding should be infallible");
    assert_eq!(-25.25, decoded);
    assert_eq!(8, consumed.get());
}

#[test]
fn test_binary_codec_handles_byte_signed_and_float_values() {
    let mut output = [0u8; 16];

    unsafe {
        assert_eq!(
            1,
            BinaryCodec::<u8, BigEndian>::encode(0x7f, &mut output, 0)
        );
        assert_eq!(
            1,
            BinaryCodec::<i8, LittleEndian>::encode(-1, &mut output, 1)
        );
        assert_eq!(
            4,
            BinaryCodec::<f32, BigEndian>::encode(12.5, &mut output, 2)
        );
        assert_eq!(
            8,
            BinaryCodec::<f64, LittleEndian>::encode(-25.25, &mut output, 6)
        );
    }

    assert_decoded_eq((0x7f, 1), unsafe {
        BinaryCodec::<u8, LittleEndian>::decode(&output, 0)
    });
    assert_decoded_eq((-1, 1), unsafe {
        BinaryCodec::<i8, BigEndian>::decode(&output, 1)
    });
    assert_decoded_eq((12.5, 4), unsafe {
        BinaryCodec::<f32, BigEndian>::decode(&output, 2)
    });
    assert_decoded_eq((-25.25, 8), unsafe {
        BinaryCodec::<f64, LittleEndian>::decode(&output, 6)
    });
}
