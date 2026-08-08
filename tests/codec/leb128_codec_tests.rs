// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use core::num::NonZeroUsize;

use qubit_codec::Codec;
use qubit_codec::DecodeFailure;
use qubit_codec_binary::Leb128Codec;
use qubit_codec_binary::Leb128DecodeError;
use qubit_codec_binary::Leb128DecodeErrorKind;
use qubit_codec_binary::NonStrict;
use qubit_codec_binary::Strict;

use super::assertions_tests::assert_decoded_eq;

fn nonzero(value: usize) -> NonZeroUsize {
    NonZeroUsize::new(value).expect("test count must be non-zero")
}

/// Checks the exact unsigned LEB128 bytes for a `u32` value.
fn assert_unsigned_u32_leb128_bytes(value: u32, expected: &[u8]) {
    let mut output =
        [0u8; Leb128Codec::<u32, NonStrict>::MAX_ENCODE_UNITS_PER_VALUE];

    let len =
        unsafe { Leb128Codec::<u32, NonStrict>::encode(value, &mut output, 0) };
    assert_eq!(expected.len(), len);
    assert_eq!(expected, &output[..len]);

    let decoded = unsafe { Leb128Codec::<u32, Strict>::decode(&output, 0) }
        .expect("canonical unsigned boundary value should decode");
    assert_decoded_eq((value, len), decoded);
}

/// Checks the exact signed LEB128 bytes for an `i32` value.
fn assert_signed_i32_leb128_bytes(value: i32, expected: &[u8]) {
    let mut output =
        [0u8; Leb128Codec::<i32, NonStrict>::MAX_ENCODE_UNITS_PER_VALUE];

    let len =
        unsafe { Leb128Codec::<i32, NonStrict>::encode(value, &mut output, 0) };
    assert_eq!(expected.len(), len);
    assert_eq!(expected, &output[..len]);

    let decoded = unsafe { Leb128Codec::<i32, Strict>::decode(&output, 0) }
        .expect("canonical signed boundary value should decode");
    assert_decoded_eq((value, len), decoded);
}

#[test]
fn test_leb128_codec_exposes_unit_bounds() {
    assert_eq!(1, Leb128Codec::<u8, NonStrict>::MIN_UNITS_PER_VALUE);
    assert_eq!(2, Leb128Codec::<u8, NonStrict>::MAX_ENCODE_UNITS_PER_VALUE);
    assert_eq!(2, Leb128Codec::<u8, NonStrict>::MAX_DECODE_UNITS_PER_VALUE);
    assert_eq!(3, Leb128Codec::<u16, NonStrict>::MAX_ENCODE_UNITS_PER_VALUE);
    assert_eq!(3, Leb128Codec::<u16, NonStrict>::MAX_DECODE_UNITS_PER_VALUE);
    assert_eq!(5, Leb128Codec::<u32, NonStrict>::MAX_ENCODE_UNITS_PER_VALUE);
    assert_eq!(5, Leb128Codec::<u32, NonStrict>::MAX_DECODE_UNITS_PER_VALUE);
    assert_eq!(
        10,
        Leb128Codec::<u64, NonStrict>::MAX_ENCODE_UNITS_PER_VALUE
    );
    assert_eq!(
        10,
        Leb128Codec::<u64, NonStrict>::MAX_DECODE_UNITS_PER_VALUE
    );
    assert_eq!(
        19,
        Leb128Codec::<u128, NonStrict>::MAX_ENCODE_UNITS_PER_VALUE
    );
    assert_eq!(
        19,
        Leb128Codec::<u128, NonStrict>::MAX_DECODE_UNITS_PER_VALUE
    );
    assert_eq!(
        (usize::BITS as usize).div_ceil(7),
        Leb128Codec::<usize, NonStrict>::MAX_ENCODE_UNITS_PER_VALUE
    );
    assert_eq!(
        (usize::BITS as usize).div_ceil(7),
        Leb128Codec::<usize, NonStrict>::MAX_DECODE_UNITS_PER_VALUE
    );
    assert_eq!(1, Leb128Codec::<i8, Strict>::MIN_UNITS_PER_VALUE);
    assert_eq!(2, Leb128Codec::<i8, Strict>::MAX_ENCODE_UNITS_PER_VALUE);
    assert_eq!(2, Leb128Codec::<i8, Strict>::MAX_DECODE_UNITS_PER_VALUE);
    assert_eq!(3, Leb128Codec::<i16, Strict>::MAX_ENCODE_UNITS_PER_VALUE);
    assert_eq!(3, Leb128Codec::<i16, Strict>::MAX_DECODE_UNITS_PER_VALUE);
}

#[test]
fn test_leb128_codec_encodes_unsigned_7_bit_boundaries() {
    let cases: &[(u32, &[u8])] = &[
        (0, &[0x00]),
        (1, &[0x01]),
        (0x7f, &[0x7f]),
        (0x80, &[0x80, 0x01]),
        (0x3fff, &[0xff, 0x7f]),
        (0x4000, &[0x80, 0x80, 0x01]),
        (u32::MAX, &[0xff, 0xff, 0xff, 0xff, 0x0f]),
    ];

    for &(value, expected) in cases {
        assert_unsigned_u32_leb128_bytes(value, expected);
    }
}

#[test]
fn test_leb128_codec_encodes_signed_7_bit_boundaries() {
    let cases: &[(i32, &[u8])] = &[
        (0, &[0x00]),
        (-1, &[0x7f]),
        (63, &[0x3f]),
        (64, &[0xc0, 0x00]),
        (-64, &[0x40]),
        (-65, &[0xbf, 0x7f]),
        (i32::MIN, &[0x80, 0x80, 0x80, 0x80, 0x78]),
        (i32::MAX, &[0xff, 0xff, 0xff, 0xff, 0x07]),
    ];

    for &(value, expected) in cases {
        assert_signed_i32_leb128_bytes(value, expected);
    }
}

#[test]
fn test_leb128_codec_non_strict_accepts_redundant_values() {
    let decoded =
        unsafe { Leb128Codec::<u16, NonStrict>::decode(&[0x80, 0x00], 0) }
            .expect("non-strict unsigned LEB128 should accept redundant zero");
    assert_decoded_eq((0, 2), decoded);

    let decoded =
        unsafe { Leb128Codec::<i16, NonStrict>::decode(&[0xff, 0x7f], 0) }
            .expect(
                "non-strict signed LEB128 should accept redundant negative one",
            );
    assert_decoded_eq((-1, 2), decoded);
}

#[test]
fn test_leb128_codec_strictly_checks_terminal_sign_extensions() {
    let decoded =
        unsafe { Leb128Codec::<u16, Strict>::decode(&[0x80, 0x01], 0) }
            .expect("canonical unsigned value should decode");
    assert_decoded_eq((128, 2), decoded);

    let error = unsafe { Leb128Codec::<u16, Strict>::decode(&[0xff, 0x00], 0) }
        .expect_err("redundant unsigned terminal byte should fail");
    assert_eq!(Leb128DecodeErrorKind::NonCanonical, error.kind());
    assert_eq!(Some(nonzero(2)), error.consumed());

    let decoded =
        unsafe { Leb128Codec::<i16, Strict>::decode(&[0xff, 0x00], 0) }
            .expect("positive signed boundary value should decode");
    assert_decoded_eq((127, 2), decoded);

    let decoded =
        unsafe { Leb128Codec::<i16, Strict>::decode(&[0x80, 0x7f], 0) }
            .expect("negative signed boundary value should decode");
    assert_decoded_eq((-128, 2), decoded);

    for bytes in [[0x80, 0x00], [0xff, 0x7f]] {
        let error = unsafe { Leb128Codec::<i16, Strict>::decode(&bytes, 0) }
            .expect_err("redundant signed terminal byte should fail");
        assert_eq!(Leb128DecodeErrorKind::NonCanonical, error.kind());
        assert_eq!(Some(nonzero(2)), error.consumed());
    }
}

#[test]
fn test_leb128_codec_roundtrips_all_u8_and_i8_values() {
    let mut unsigned_output =
        [0u8; Leb128Codec::<u8, NonStrict>::MAX_ENCODE_UNITS_PER_VALUE];
    for value in u8::MIN..=u8::MAX {
        let len = unsafe {
            Leb128Codec::<u8, NonStrict>::encode(value, &mut unsigned_output, 0)
        };
        let decoded = unsafe {
            Leb128Codec::<u8, Strict>::decode(&unsigned_output[..len], 0)
        }
        .expect("canonical u8 LEB128 should decode");
        assert_decoded_eq((value, len), decoded);
    }

    let mut signed_output =
        [0u8; Leb128Codec::<i8, NonStrict>::MAX_ENCODE_UNITS_PER_VALUE];
    for value in i8::MIN..=i8::MAX {
        let len = unsafe {
            Leb128Codec::<i8, NonStrict>::encode(value, &mut signed_output, 0)
        };
        let decoded = unsafe {
            Leb128Codec::<i8, Strict>::decode(&signed_output[..len], 0)
        }
        .expect("canonical i8 LEB128 should decode");
        assert_decoded_eq((value, len), decoded);
    }
}

#[test]
fn test_leb128_codec_reads_and_writes_unsigned_values_unchecked() {
    let mut output =
        [0u8; Leb128Codec::<u16, NonStrict>::MAX_ENCODE_UNITS_PER_VALUE + 2];
    let len =
        unsafe { Leb128Codec::<u16, NonStrict>::encode(300, &mut output, 1) };

    assert_eq!(2, len);
    assert_eq!([0x00, 0xac, 0x02, 0x00, 0x00], output);

    let decoded = unsafe { Leb128Codec::<u16, NonStrict>::decode(&output, 1) }
        .expect("valid u16 should decode");
    assert_decoded_eq((300, 2), decoded);

    let mut output =
        [0u8; Leb128Codec::<u16, NonStrict>::MAX_ENCODE_UNITS_PER_VALUE];
    let len = unsafe {
        Leb128Codec::<u16, NonStrict>::encode(u16::MAX, &mut output, 0)
    };
    let decoded = unsafe { Leb128Codec::<u16, NonStrict>::decode(&output, 0) }
        .expect("u16::MAX should decode");
    assert_decoded_eq((u16::MAX, len), decoded);
}

#[test]
fn test_leb128_codec_encodes_and_decodes_through_codec_trait() {
    let mut codec = Leb128Codec::<u16, NonStrict>::default();
    let mut output =
        [0u8; Leb128Codec::<u16, NonStrict>::MAX_ENCODE_UNITS_PER_VALUE + 2];

    assert_eq!(
        Leb128Codec::<u16, NonStrict>::MIN_UNITS_PER_VALUE,
        <Leb128Codec<u16, NonStrict> as Codec>::MIN_UNITS_PER_VALUE
    );
    assert_eq!(
        Leb128Codec::<u16, NonStrict>::MAX_ENCODE_UNITS_PER_VALUE,
        <Leb128Codec<u16, NonStrict> as Codec>::MAX_ENCODE_UNITS_PER_VALUE
    );
    assert_eq!(
        Leb128Codec::<u16, NonStrict>::MAX_DECODE_UNITS_PER_VALUE,
        <Leb128Codec<u16, NonStrict> as Codec>::MAX_DECODE_UNITS_PER_VALUE
    );

    let written = unsafe { Codec::encode(&mut codec, &300, &mut output, 1) }
        .expect("LEB128 encoding should be infallible");
    assert_eq!(2, written);
    assert_eq!([0x00, 0xac, 0x02, 0x00, 0x00], output);

    let decoded = unsafe { Codec::decode(&mut codec, &output, 1) }
        .expect("valid LEB128 value should decode");
    assert_decoded_eq((300, 2), decoded);
}

#[test]
fn test_unsigned_leb128_codec_trait_reports_exact_encoded_lengths() {
    let codec = Leb128Codec::<u32, NonStrict>::default();

    assert_eq!(1, codec.encode_len(&0));
    assert_eq!(1, codec.encode_len(&0x7f));
    assert_eq!(2, codec.encode_len(&0x80));
    assert_eq!(3, codec.encode_len(&0x4000));
    assert_eq!(5, codec.encode_len(&u32::MAX));
}

#[test]
fn test_unsigned_leb128_codec_trait_encodes_into_exact_length_buffer() {
    let mut codec = Leb128Codec::<u32, NonStrict>::default();
    let value = 0x7f;
    let mut output = vec![0_u8; codec.encode_len(&value)];

    let written = unsafe { Codec::encode(&mut codec, &value, &mut output, 0) }
        .expect("unsigned LEB128 encoding should be infallible");

    assert_eq!(output.len(), written);
    assert_eq!([0x7f], output.as_slice());
}

#[test]
fn test_unsigned_leb128_inherent_encode_accepts_exact_length_buffer() {
    let mut output = [0_u8; 1];

    let written =
        unsafe { Leb128Codec::<u64, NonStrict>::encode(0x7f, &mut output, 0) };

    assert_eq!(1, written);
    assert_eq!([0x7f], output);
}

#[test]
fn test_leb128_codec_trait_decodes_single_byte_unsigned_value() {
    let mut codec = Leb128Codec::<u64, NonStrict>::default();
    let input = [0x00u8];

    let decoded = unsafe { Codec::decode(&mut codec, &input, 0) }
        .expect("single-byte unsigned LEB128 value should decode");

    assert_decoded_eq((0, 1), decoded);
}

#[test]
fn test_leb128_codec_trait_maps_decode_failures() {
    let mut codec = Leb128Codec::<u16, NonStrict>::default();
    let incomplete = unsafe { Codec::decode(&mut codec, &[0xac], 0) }
        .expect_err("partial LEB128 value should request more input");
    assert_eq!(
        DecodeFailure::incomplete_with_source(
            Leb128DecodeError::incomplete(0, nonzero(2), 1),
            nonzero(2),
        ),
        incomplete,
    );
    assert_eq!(
        Some(&Leb128DecodeError::incomplete(0, nonzero(2), 1)),
        incomplete.incomplete_source(),
    );

    let mut codec = Leb128Codec::<u16, Strict>::default();
    let invalid = unsafe { Codec::decode(&mut codec, &[0x80, 0x00], 0) }
        .expect_err("non-canonical LEB128 value should be invalid");
    match invalid {
        DecodeFailure::Invalid {
            source, consumed, ..
        } => {
            assert_eq!(Leb128DecodeErrorKind::NonCanonical, source.kind());
            assert_eq!(Some(nonzero(2)), consumed);
        }
        other => {
            panic!("non-canonical LEB128 value must be invalid: {other:?}");
        }
    }
}

#[test]
fn test_signed_leb128_codec_encodes_and_decodes_through_codec_trait() {
    let mut codec = Leb128Codec::<i16, NonStrict>::default();
    let mut output =
        [0u8; Leb128Codec::<i16, NonStrict>::MAX_ENCODE_UNITS_PER_VALUE + 2];

    assert_eq!(
        Leb128Codec::<i16, NonStrict>::MIN_UNITS_PER_VALUE,
        <Leb128Codec<i16, NonStrict> as Codec>::MIN_UNITS_PER_VALUE
    );
    assert_eq!(
        Leb128Codec::<i16, NonStrict>::MAX_ENCODE_UNITS_PER_VALUE,
        <Leb128Codec<i16, NonStrict> as Codec>::MAX_ENCODE_UNITS_PER_VALUE
    );

    let written = unsafe { Codec::encode(&mut codec, &-300, &mut output, 1) }
        .expect("signed LEB128 encoding should be infallible");
    assert_eq!(2, written);
    assert_eq!([0x00, 0xd4, 0x7d, 0x00, 0x00], output);

    let decoded = unsafe { Codec::decode(&mut codec, &output, 1) }
        .expect("valid signed LEB128 value should decode");
    assert_decoded_eq((-300, 2), decoded);
}

#[test]
fn test_signed_leb128_codec_trait_reports_exact_encoded_lengths() {
    let codec = Leb128Codec::<i32, NonStrict>::default();

    assert_eq!(1, codec.encode_len(&0));
    assert_eq!(1, codec.encode_len(&-1));
    assert_eq!(1, codec.encode_len(&63));
    assert_eq!(1, codec.encode_len(&-64));
    assert_eq!(2, codec.encode_len(&64));
    assert_eq!(2, codec.encode_len(&-65));
    assert_eq!(5, codec.encode_len(&i32::MIN));
    assert_eq!(5, codec.encode_len(&i32::MAX));
}

#[test]
fn test_signed_leb128_codec_trait_encodes_into_exact_length_buffer() {
    let mut codec = Leb128Codec::<i32, NonStrict>::default();
    let value = -1;
    let mut output = vec![0_u8; codec.encode_len(&value)];

    let written = unsafe { Codec::encode(&mut codec, &value, &mut output, 0) }
        .expect("signed LEB128 encoding should be infallible");

    assert_eq!(output.len(), written);
    assert_eq!([0x7f], output.as_slice());
}

#[test]
fn test_signed_leb128_inherent_encode_accepts_exact_length_buffer() {
    let mut output = [0_u8; 1];

    let written =
        unsafe { Leb128Codec::<i64, NonStrict>::encode(-1, &mut output, 0) };

    assert_eq!(1, written);
    assert_eq!([0x7f], output);
}

#[test]
fn test_leb128_codec_trait_decodes_single_byte_signed_value() {
    let mut codec = Leb128Codec::<i64, NonStrict>::default();
    let input = [0x7fu8];

    let decoded = unsafe { Codec::decode(&mut codec, &input, 0) }
        .expect("single-byte signed LEB128 value should decode");

    assert_decoded_eq((-1, 1), decoded);
}

#[test]
fn test_leb128_codec_reads_and_writes_signed_values_unchecked() {
    let mut output =
        [0u8; Leb128Codec::<i16, NonStrict>::MAX_ENCODE_UNITS_PER_VALUE + 2];
    let len =
        unsafe { Leb128Codec::<i16, NonStrict>::encode(-300, &mut output, 1) };

    assert_eq!(2, len);
    assert_eq!([0x00, 0xd4, 0x7d, 0x00, 0x00], output);

    let decoded = unsafe { Leb128Codec::<i16, NonStrict>::decode(&output, 1) }
        .expect("valid i16 should decode");
    assert_decoded_eq((-300, 2), decoded);

    let mut output =
        [0u8; Leb128Codec::<i16, NonStrict>::MAX_ENCODE_UNITS_PER_VALUE];
    let len =
        unsafe { Leb128Codec::<i16, NonStrict>::encode(300, &mut output, 0) };
    let decoded = unsafe { Leb128Codec::<i16, NonStrict>::decode(&output, 0) }
        .expect("positive i16 should decode");
    assert_decoded_eq((300, len), decoded);

    let mut output =
        [0u8; Leb128Codec::<i128, NonStrict>::MAX_ENCODE_UNITS_PER_VALUE];
    let len = unsafe {
        Leb128Codec::<i128, NonStrict>::encode(i128::MIN, &mut output, 0)
    };
    let decoded = unsafe { Leb128Codec::<i128, NonStrict>::decode(&output, 0) }
        .expect("i128::MIN should decode");
    assert_decoded_eq((i128::MIN, len), decoded);

    let values: [i16; 8] = [0, -1, 63, 64, -64, -65, i16::MIN, i16::MAX];
    let mut output =
        [0u8; Leb128Codec::<i16, NonStrict>::MAX_ENCODE_UNITS_PER_VALUE];
    for value in values {
        output.fill(0);
        let len = unsafe {
            Leb128Codec::<i16, NonStrict>::encode(value, &mut output, 0)
        };
        let decoded =
            unsafe { Leb128Codec::<i16, NonStrict>::decode(&output, 0) }
                .expect("signed boundary value should decode");
        assert_decoded_eq((value, len), decoded);
    }
}

#[test]
fn test_leb128_codec_roundtrips_all_strict_and_non_strict_instantiations() {
    macro_rules! roundtrip_unsigned {
        ($ty:ty, $value:expr) => {{
            let value = $value as $ty;

            let mut output = [0u8;
                Leb128Codec::<$ty, NonStrict>::MAX_ENCODE_UNITS_PER_VALUE];
            let len = unsafe {
                Leb128Codec::<$ty, NonStrict>::encode(value, &mut output, 0)
            };
            let decoded =
                unsafe { Leb128Codec::<$ty, NonStrict>::decode(&output, 0) }
                    .expect("non-strict unsigned value should decode");
            assert_decoded_eq((value, len), decoded);

            let mut output =
                [0u8; Leb128Codec::<$ty, Strict>::MAX_ENCODE_UNITS_PER_VALUE];
            let len = unsafe {
                Leb128Codec::<$ty, Strict>::encode(value, &mut output, 0)
            };
            let decoded =
                unsafe { Leb128Codec::<$ty, Strict>::decode(&output, 0) }
                    .expect("strict unsigned value should decode");
            assert_decoded_eq((value, len), decoded);
        }};
    }

    macro_rules! roundtrip_signed {
        ($ty:ty, $value:expr) => {{
            let value = $value as $ty;

            let mut output = [0u8;
                Leb128Codec::<$ty, NonStrict>::MAX_ENCODE_UNITS_PER_VALUE];
            let len = unsafe {
                Leb128Codec::<$ty, NonStrict>::encode(value, &mut output, 0)
            };
            let decoded =
                unsafe { Leb128Codec::<$ty, NonStrict>::decode(&output, 0) }
                    .expect("non-strict signed value should decode");
            assert_decoded_eq((value, len), decoded);

            let mut output =
                [0u8; Leb128Codec::<$ty, Strict>::MAX_ENCODE_UNITS_PER_VALUE];
            let len = unsafe {
                Leb128Codec::<$ty, Strict>::encode(value, &mut output, 0)
            };
            let decoded =
                unsafe { Leb128Codec::<$ty, Strict>::decode(&output, 0) }
                    .expect("strict signed value should decode");
            assert_decoded_eq((value, len), decoded);
        }};
    }

    roundtrip_unsigned!(u8, u8::MAX);
    roundtrip_unsigned!(u16, u16::MAX);
    roundtrip_unsigned!(u32, u32::MAX);
    roundtrip_unsigned!(u64, u64::MAX);
    roundtrip_unsigned!(u128, u128::MAX);
    roundtrip_unsigned!(usize, usize::MAX);

    roundtrip_signed!(i8, i8::MIN);
    roundtrip_signed!(i16, i16::MIN);
    roundtrip_signed!(i32, i32::MIN);
    roundtrip_signed!(i64, i64::MIN);
    roundtrip_signed!(i128, i128::MIN);
    roundtrip_signed!(isize, isize::MIN);
}

#[test]
fn test_leb128_codec_reports_incomplete_unsigned_values_unchecked() {
    let input = [0x00, 0xac, 0x02, 0xff];

    let pending =
        unsafe { Leb128Codec::<u16, NonStrict>::decode(&input[..2], 1) }
            .expect_err(
                "partial unsigned LEB128 should report incomplete input",
            );
    assert_eq!(Leb128DecodeErrorKind::Incomplete, pending.kind());
    assert_eq!(1, pending.start_index());
    assert_eq!(2, pending.error_index());
    assert_eq!(Some(nonzero(2)), pending.required());
    assert_eq!(Some(1), pending.available());
    assert_eq!(Some(nonzero(1)), pending.additional());

    let decoded = unsafe { Leb128Codec::<u16, NonStrict>::decode(&input, 1) }
        .expect("complete unsigned LEB128 should decode");
    assert_decoded_eq((300, 2), decoded);

    let error = unsafe { Leb128Codec::<u16, Strict>::decode(&[0x80, 0x00], 0) }
        .expect_err("non-canonical unsigned value should fail");
    assert_eq!(Leb128DecodeErrorKind::NonCanonical, error.kind());
    assert_eq!(0, error.start_index());
    assert_eq!(1, error.error_index());
    assert_eq!(Some(nonzero(2)), error.consumed());
}

#[test]
fn test_leb128_codec_reports_incomplete_signed_values_unchecked() {
    let input = [0x00, 0xd4, 0x7d, 0xff];

    let pending =
        unsafe { Leb128Codec::<i16, NonStrict>::decode(&input[..2], 1) }
            .expect_err("partial signed LEB128 should report incomplete input");
    assert_eq!(Leb128DecodeErrorKind::Incomplete, pending.kind());
    assert_eq!(1, pending.start_index());
    assert_eq!(2, pending.error_index());
    assert_eq!(Some(nonzero(2)), pending.required());
    assert_eq!(Some(1), pending.available());
    assert_eq!(Some(nonzero(1)), pending.additional());

    let decoded = unsafe { Leb128Codec::<i16, NonStrict>::decode(&input, 1) }
        .expect("complete signed LEB128 should decode");
    assert_decoded_eq((-300, 2), decoded);

    let error = unsafe { Leb128Codec::<i16, Strict>::decode(&[0xff, 0x7f], 0) }
        .expect_err("non-canonical signed value should fail");
    assert_eq!(Leb128DecodeErrorKind::NonCanonical, error.kind());
    assert_eq!(0, error.start_index());
    assert_eq!(1, error.error_index());
    assert_eq!(Some(nonzero(2)), error.consumed());
}

#[test]
fn test_leb128_codec_rejects_all_instantiated_error_paths() {
    macro_rules! reject_unsigned {
        ($ty:ty) => {{
            let unterminated = [0x80u8;
                Leb128Codec::<$ty, NonStrict>::MAX_DECODE_UNITS_PER_VALUE];
            let error = unsafe {
                Leb128Codec::<$ty, NonStrict>::decode(&unterminated, 0)
            }
            .expect_err("unterminated unsigned value should fail");
            assert_eq!(Leb128DecodeErrorKind::Malformed, error.kind());

            let error =
                unsafe { Leb128Codec::<$ty, Strict>::decode(&unterminated, 0) }
                    .expect_err(
                        "unterminated strict unsigned value should fail",
                    );
            assert_eq!(Leb128DecodeErrorKind::Malformed, error.kind());

            let max_bytes =
                Leb128Codec::<$ty, NonStrict>::MAX_DECODE_UNITS_PER_VALUE;
            let bits = <$ty>::BITS as usize;
            let used_bits = bits - (max_bytes - 1) * 7;
            let mut malformed = [0x80u8;
                Leb128Codec::<$ty, NonStrict>::MAX_DECODE_UNITS_PER_VALUE];
            malformed[max_bytes - 1] = 1u8 << used_bits;
            let error =
                unsafe { Leb128Codec::<$ty, NonStrict>::decode(&malformed, 0) }
                    .expect_err("too-wide unsigned payload should fail");
            assert_eq!(Leb128DecodeErrorKind::Malformed, error.kind());

            let mut noncanonical =
                [0u8; Leb128Codec::<$ty, Strict>::MAX_DECODE_UNITS_PER_VALUE];
            noncanonical[0] = 0x80;
            let error =
                unsafe { Leb128Codec::<$ty, Strict>::decode(&noncanonical, 0) }
                    .expect_err("non-canonical unsigned value should fail");
            assert_eq!(Leb128DecodeErrorKind::NonCanonical, error.kind());
        }};
    }

    macro_rules! reject_signed {
        ($ty:ty) => {{
            let unterminated = [0x80u8;
                Leb128Codec::<$ty, NonStrict>::MAX_DECODE_UNITS_PER_VALUE];
            let error = unsafe {
                Leb128Codec::<$ty, NonStrict>::decode(&unterminated, 0)
            }
            .expect_err("unterminated signed value should fail");
            assert_eq!(Leb128DecodeErrorKind::Malformed, error.kind());

            let error =
                unsafe { Leb128Codec::<$ty, Strict>::decode(&unterminated, 0) }
                    .expect_err("unterminated strict signed value should fail");
            assert_eq!(Leb128DecodeErrorKind::Malformed, error.kind());

            let max_bytes =
                Leb128Codec::<$ty, NonStrict>::MAX_DECODE_UNITS_PER_VALUE;
            let bits = <$ty>::BITS as usize;
            let used_bits = bits - (max_bytes - 1) * 7;

            let mut malformed = [0x80u8;
                Leb128Codec::<$ty, NonStrict>::MAX_DECODE_UNITS_PER_VALUE];
            malformed[max_bytes - 1] = 1u8 << used_bits;
            let error =
                unsafe { Leb128Codec::<$ty, NonStrict>::decode(&malformed, 0) }
                    .expect_err("too-wide positive signed payload should fail");
            assert_eq!(Leb128DecodeErrorKind::Malformed, error.kind());

            let mut malformed = [0x80u8;
                Leb128Codec::<$ty, NonStrict>::MAX_DECODE_UNITS_PER_VALUE];
            malformed[max_bytes - 1] = 1u8 << (used_bits - 1);
            let error =
                unsafe { Leb128Codec::<$ty, NonStrict>::decode(&malformed, 0) }
                    .expect_err(
                        "too-narrow negative signed payload should fail",
                    );
            assert_eq!(Leb128DecodeErrorKind::Malformed, error.kind());

            let mut noncanonical =
                [0u8; Leb128Codec::<$ty, Strict>::MAX_DECODE_UNITS_PER_VALUE];
            noncanonical[0] = 0xff;
            noncanonical[1] = 0x7f;
            let error =
                unsafe { Leb128Codec::<$ty, Strict>::decode(&noncanonical, 0) }
                    .expect_err("non-canonical signed value should fail");
            assert_eq!(Leb128DecodeErrorKind::NonCanonical, error.kind());
        }};
    }

    reject_unsigned!(u8);
    reject_unsigned!(u16);
    reject_unsigned!(u32);
    reject_unsigned!(u64);
    reject_unsigned!(u128);
    reject_unsigned!(usize);

    reject_signed!(i8);
    reject_signed!(i16);
    reject_signed!(i32);
    reject_signed!(i64);
    reject_signed!(i128);
    reject_signed!(isize);
}

#[test]
fn test_leb128_codec_rejects_malformed_values() {
    let error = unsafe {
        Leb128Codec::<u16, NonStrict>::decode(&[0x80, 0x80, 0x04], 0)
    }
    .expect_err("too-wide unsigned payload should fail");
    assert_eq!(Leb128DecodeErrorKind::Malformed, error.kind());
    assert_eq!(0, error.start_index());
    assert_eq!(2, error.error_index());

    let error = unsafe {
        Leb128Codec::<u16, NonStrict>::decode(&[0x80, 0x80, 0x80], 0)
    }
    .expect_err("unterminated unsigned payload should fail");
    assert_eq!(Leb128DecodeErrorKind::Malformed, error.kind());
    assert_eq!(0, error.start_index());
    assert_eq!(2, error.error_index());

    let error = unsafe {
        Leb128Codec::<i16, NonStrict>::decode(&[0x80, 0x80, 0x04], 0)
    }
    .expect_err("too-wide signed payload should fail");
    assert_eq!(Leb128DecodeErrorKind::Malformed, error.kind());
    assert_eq!(0, error.start_index());
    assert_eq!(2, error.error_index());

    let error = unsafe {
        Leb128Codec::<i16, NonStrict>::decode(&[0x80, 0x80, 0x02], 0)
    }
    .expect_err("too-narrow negative signed payload should fail");
    assert_eq!(Leb128DecodeErrorKind::Malformed, error.kind());
    assert_eq!(0, error.start_index());
    assert_eq!(2, error.error_index());

    let error = unsafe {
        Leb128Codec::<i16, NonStrict>::decode(&[0x80, 0x80, 0x80], 0)
    }
    .expect_err("unterminated signed payload should fail");
    assert_eq!(Leb128DecodeErrorKind::Malformed, error.kind());
    assert_eq!(0, error.start_index());
    assert_eq!(2, error.error_index());
}

#[test]
fn test_leb128_codec_rejects_noncanonical_strict_values() {
    let decoded =
        unsafe { Leb128Codec::<u16, Strict>::decode(&[0xac, 0x02, 0x00], 0) }
            .expect("canonical unsigned value should decode");
    assert_decoded_eq((300, 2), decoded);

    let decoded =
        unsafe { Leb128Codec::<i16, Strict>::decode(&[0xd4, 0x7d, 0x00], 0) }
            .expect("canonical signed value should decode");
    assert_decoded_eq((-300, 2), decoded);

    let decoded =
        unsafe { Leb128Codec::<i16, Strict>::decode(&[0xac, 0x02, 0x00], 0) }
            .expect("canonical positive signed value should decode");
    assert_decoded_eq((300, 2), decoded);

    let error =
        unsafe { Leb128Codec::<u16, Strict>::decode(&[0x80, 0x00, 0x00], 0) }
            .expect_err("non-canonical unsigned value should fail");
    assert_eq!(Leb128DecodeErrorKind::NonCanonical, error.kind());
    assert_eq!(0, error.start_index());
    assert_eq!(1, error.error_index());

    let error =
        unsafe { Leb128Codec::<i16, Strict>::decode(&[0xff, 0x7f, 0x00], 0) }
            .expect_err("non-canonical signed value should fail");
    assert_eq!(Leb128DecodeErrorKind::NonCanonical, error.kind());
    assert_eq!(0, error.start_index());
    assert_eq!(1, error.error_index());
}
