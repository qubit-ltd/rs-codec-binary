// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::NativeEndian;
use qubit_codec::{
    BigEndian,
    Codec,
    LittleEndian,
};
use qubit_codec_binary::BinaryCodec;

#[test]
fn test_native_endian_round_trip_matches_platform_order() {
    let value = 0x1234_5678_u32;
    let mut output =
        [0_u8; BinaryCodec::<u32, NativeEndian>::MAX_ENCODE_UNITS_PER_VALUE];
    let written = unsafe {
        BinaryCodec::<u32, NativeEndian>::encode(value, &mut output, 0)
    };
    assert_eq!(4, written);
    assert_eq!(value.to_ne_bytes(), output);
    let (decoded, consumed) =
        unsafe { BinaryCodec::<u32, NativeEndian>::decode(&output, 0) };
    assert_eq!(value, decoded);
    assert_eq!(4, consumed.get());
}

#[test]
fn test_native_endian_round_trip_covers_all_supported_scalar_types() {
    macro_rules! assert_integer {
        ($ty:ty, $value:expr) => {{
            let value: $ty = $value;
            let mut output = [0_u8;
                BinaryCodec::<$ty, NativeEndian>::MAX_ENCODE_UNITS_PER_VALUE];
            let written = unsafe {
                BinaryCodec::<$ty, NativeEndian>::encode(value, &mut output, 0)
            };
            assert_eq!(value.to_ne_bytes(), output);
            assert_eq!(output.len(), written);

            let (decoded, consumed) =
                unsafe { BinaryCodec::<$ty, NativeEndian>::decode(&output, 0) };
            assert_eq!(value, decoded);
            assert_eq!(written, consumed.get());
        }};
    }

    macro_rules! assert_float {
        ($ty:ty, $value:expr) => {{
            let value: $ty = $value;
            let mut output = [0_u8;
                BinaryCodec::<$ty, NativeEndian>::MAX_ENCODE_UNITS_PER_VALUE];
            let written = unsafe {
                BinaryCodec::<$ty, NativeEndian>::encode(value, &mut output, 0)
            };
            assert_eq!(value.to_bits().to_ne_bytes(), output);
            assert_eq!(output.len(), written);

            let (decoded, consumed) =
                unsafe { BinaryCodec::<$ty, NativeEndian>::decode(&output, 0) };
            assert_eq!(value.to_bits(), decoded.to_bits());
            assert_eq!(written, consumed.get());
        }};
    }

    assert_integer!(u8, 0xa5);
    assert_integer!(i8, -37);
    assert_integer!(u16, 0xa5b6);
    assert_integer!(u32, 0xa5b6_c7d8);
    assert_integer!(u64, 0xa5b6_c7d8_e9fa_0b1c);
    assert_integer!(u128, 0xa5b6_c7d8_e9fa_0b1c_2d3e_4f50_6172_8394);
    assert_integer!(i16, -0x1234);
    assert_integer!(i32, -0x1234_5678);
    assert_integer!(i64, -0x1234_5678_9abc_def0);
    assert_integer!(i128, -0x1234_5678_9abc_def0_1234_5678_9abc_def0);
    assert_float!(f32, f32::from_bits(0x7fc0_0123));
    assert_float!(f64, f64::from_bits(0x7ff8_0000_0000_0123));
}

use super::assertions_tests::assert_decoded_eq;

#[test]
fn test_binary_codec_exposes_unit_bounds() {
    macro_rules! assert_bounds {
        ($ty:ty, $order:ty, $width:expr) => {
            assert_eq!($width, BinaryCodec::<$ty, $order>::MIN_UNITS_PER_VALUE);
            assert_eq!(
                $width,
                BinaryCodec::<$ty, $order>::MAX_ENCODE_UNITS_PER_VALUE
            );
            assert_eq!(
                $width,
                BinaryCodec::<$ty, $order>::MAX_DECODE_UNITS_PER_VALUE
            );
        };
    }

    assert_bounds!(u8, BigEndian, 1);
    assert_bounds!(i8, LittleEndian, 1);
    assert_bounds!(u16, BigEndian, 2);
    assert_bounds!(u32, LittleEndian, 4);
    assert_bounds!(u64, BigEndian, 8);
    assert_bounds!(u128, LittleEndian, 16);
    assert_bounds!(f32, BigEndian, 4);
    assert_bounds!(f64, LittleEndian, 8);
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

#[cfg(debug_assertions)]
#[test]
#[should_panic]
fn test_binary_codec_big_endian_decode_checks_readable_width() {
    unsafe {
        let _ = BinaryCodec::<u32, BigEndian>::decode(&[0; 3], 0);
    }
}

#[cfg(debug_assertions)]
#[test]
#[should_panic]
fn test_binary_codec_little_endian_encode_checks_writable_width() {
    unsafe {
        BinaryCodec::<u32, LittleEndian>::encode(0, &mut [0; 3], 0);
    }
}

#[cfg(debug_assertions)]
#[test]
#[should_panic]
fn test_binary_codec_big_endian_float_decode_checks_readable_width() {
    unsafe {
        let _ = BinaryCodec::<f32, BigEndian>::decode(&[0; 3], 0);
    }
}

#[cfg(debug_assertions)]
#[test]
#[should_panic]
fn test_binary_codec_little_endian_float_encode_checks_writable_width() {
    unsafe {
        BinaryCodec::<f64, LittleEndian>::encode(0.0, &mut [0; 7], 0);
    }
}

#[cfg(debug_assertions)]
#[test]
#[should_panic]
fn test_binary_codec_native_endian_decode_checks_readable_width() {
    unsafe {
        let _ = BinaryCodec::<u64, NativeEndian>::decode(&[0; 7], 0);
    }
}

#[cfg(debug_assertions)]
#[test]
#[should_panic]
fn test_binary_codec_native_endian_float_encode_checks_writable_width() {
    unsafe {
        BinaryCodec::<f64, NativeEndian>::encode(0.0, &mut [0; 7], 0);
    }
}

#[test]
fn test_binary_codec_roundtrips_integer_extremes_for_all_fixed_width_types() {
    macro_rules! assert_extreme_roundtrip {
        ($ty:ty, $value:expr) => {{
            let value: $ty = $value;

            let mut output = [0u8;
                BinaryCodec::<$ty, BigEndian>::MAX_ENCODE_UNITS_PER_VALUE];
            let written = unsafe {
                BinaryCodec::<$ty, BigEndian>::encode(value, &mut output, 0)
            };
            assert_eq!(output.len(), written);
            assert_eq!(value.to_be_bytes(), output);
            let decoded =
                unsafe { BinaryCodec::<$ty, BigEndian>::decode(&output, 0) };
            assert_decoded_eq((value, output.len()), decoded);

            let mut output = [0u8;
                BinaryCodec::<$ty, LittleEndian>::MAX_ENCODE_UNITS_PER_VALUE];
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

            let mut output = [0u8;
                BinaryCodec::<f32, BigEndian>::MAX_ENCODE_UNITS_PER_VALUE];
            let written = unsafe {
                BinaryCodec::<f32, BigEndian>::encode(value, &mut output, 0)
            };
            assert_eq!(output.len(), written);
            assert_eq!(bits.to_be_bytes(), output);
            let (decoded, consumed) =
                unsafe { BinaryCodec::<f32, BigEndian>::decode(&output, 0) };
            assert_eq!(bits, decoded.to_bits());
            assert_eq!(output.len(), consumed.get());

            let mut output = [0u8;
                BinaryCodec::<f32, LittleEndian>::MAX_ENCODE_UNITS_PER_VALUE];
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

            let mut output = [0u8;
                BinaryCodec::<f64, BigEndian>::MAX_ENCODE_UNITS_PER_VALUE];
            let written = unsafe {
                BinaryCodec::<f64, BigEndian>::encode(value, &mut output, 0)
            };
            assert_eq!(output.len(), written);
            assert_eq!(bits.to_be_bytes(), output);
            let (decoded, consumed) =
                unsafe { BinaryCodec::<f64, BigEndian>::decode(&output, 0) };
            assert_eq!(bits, decoded.to_bits());
            assert_eq!(output.len(), consumed.get());

            let mut output = [0u8;
                BinaryCodec::<f64, LittleEndian>::MAX_ENCODE_UNITS_PER_VALUE];
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
        <BinaryCodec<u32, BigEndian> as Codec>::MAX_ENCODE_UNITS_PER_VALUE,
    );
    assert_eq!(
        4,
        <BinaryCodec<u32, BigEndian> as Codec>::MAX_DECODE_UNITS_PER_VALUE,
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
        <BinaryCodec<u8, BigEndian> as Codec>::MAX_ENCODE_UNITS_PER_VALUE,
    );
    assert_eq!(
        1,
        <BinaryCodec<i8, LittleEndian> as Codec>::MIN_UNITS_PER_VALUE,
    );
    assert_eq!(
        1,
        <BinaryCodec<i8, LittleEndian> as Codec>::MAX_ENCODE_UNITS_PER_VALUE,
    );
    assert_eq!(
        2,
        <BinaryCodec<u16, LittleEndian> as Codec>::MIN_UNITS_PER_VALUE,
    );
    assert_eq!(
        2,
        <BinaryCodec<u16, LittleEndian> as Codec>::MAX_ENCODE_UNITS_PER_VALUE,
    );
    assert_eq!(
        4,
        <BinaryCodec<f32, BigEndian> as Codec>::MIN_UNITS_PER_VALUE,
    );
    assert_eq!(
        4,
        <BinaryCodec<f32, BigEndian> as Codec>::MAX_ENCODE_UNITS_PER_VALUE,
    );
    assert_eq!(
        8,
        <BinaryCodec<f64, LittleEndian> as Codec>::MIN_UNITS_PER_VALUE,
    );
    assert_eq!(
        8,
        <BinaryCodec<f64, LittleEndian> as Codec>::MAX_ENCODE_UNITS_PER_VALUE,
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
fn test_binary_codec_native_endian_trait_covers_all_supported_scalar_types() {
    macro_rules! assert_integer {
        ($ty:ty, $value:expr) => {{
            let value: $ty = $value;
            let mut codec = BinaryCodec::<$ty, NativeEndian>::default();
            let mut output = [0xaa_u8;
                BinaryCodec::<$ty, NativeEndian>::MAX_ENCODE_UNITS_PER_VALUE
                    + 2];

            let written =
                unsafe { Codec::encode(&mut codec, &value, &mut output, 1) }
                    .expect(
                        "native-endian integer encoding should be infallible",
                    );
            assert_eq!(value.to_ne_bytes(), output[1..=written]);
            assert_eq!(output[0], 0xaa);
            assert_eq!(output[written + 1], 0xaa);

            let (decoded, consumed) =
                unsafe { Codec::decode(&mut codec, &output, 1) }.expect(
                    "native-endian integer decoding should be infallible",
                );
            assert_eq!(value, decoded);
            assert_eq!(written, consumed.get());
        }};
    }

    macro_rules! assert_float {
        ($ty:ty, $value:expr) => {{
            let value: $ty = $value;
            let mut codec = BinaryCodec::<$ty, NativeEndian>::default();
            let mut output = [0xaa_u8;
                BinaryCodec::<$ty, NativeEndian>::MAX_ENCODE_UNITS_PER_VALUE
                    + 2];

            let written =
                unsafe { Codec::encode(&mut codec, &value, &mut output, 1) }
                    .expect(
                        "native-endian float encoding should be infallible",
                    );
            assert_eq!(value.to_bits().to_ne_bytes(), output[1..=written]);
            assert_eq!(output[0], 0xaa);
            assert_eq!(output[written + 1], 0xaa);

            let (decoded, consumed) =
                unsafe { Codec::decode(&mut codec, &output, 1) }.expect(
                    "native-endian float decoding should be infallible",
                );
            assert_eq!(value.to_bits(), decoded.to_bits());
            assert_eq!(written, consumed.get());
        }};
    }

    assert_integer!(u8, 0xa5);
    assert_integer!(i8, -37);
    assert_integer!(u16, 0xa5b6);
    assert_integer!(i16, -0x1234);
    assert_integer!(u32, 0xa5b6_c7d8);
    assert_integer!(i32, -0x1234_5678);
    assert_integer!(u64, 0xa5b6_c7d8_e9fa_0b1c);
    assert_integer!(i64, -0x1234_5678_9abc_def0);
    assert_integer!(u128, 0xa5b6_c7d8_e9fa_0b1c_2d3e_4f50_6172_8394);
    assert_integer!(i128, -0x1234_5678_9abc_def0_1234_5678_9abc_def0);
    assert_float!(f32, f32::from_bits(0x7fc0_0123));
    assert_float!(f64, f64::from_bits(0x7ff8_0000_0000_0123));
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
