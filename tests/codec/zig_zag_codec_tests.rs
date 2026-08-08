// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use core::num::NonZeroUsize;

use qubit_codec::Codec;
use qubit_codec_binary::Leb128DecodeErrorKind;
use qubit_codec_binary::NonStrict;
use qubit_codec_binary::Strict;
use qubit_codec_binary::ZigZagCodec;

use super::assertions_tests::assert_decoded_eq;

fn nonzero(value: usize) -> NonZeroUsize {
    NonZeroUsize::new(value).expect("test count must be non-zero")
}

/// Checks the exact ZigZag LEB128 bytes for an `i16` value.
fn assert_i16_zig_zag_bytes(value: i16, expected: &[u8]) {
    let mut output = [0u8; ZigZagCodec::<i16, NonStrict>::MAX_ENCODE_UNITS_PER_VALUE];

    let len = unsafe { ZigZagCodec::<i16, NonStrict>::encode(value, &mut output, 0) };
    assert_eq!(expected.len(), len);
    assert_eq!(expected, &output[..len]);

    let decoded = unsafe { ZigZagCodec::<i16, Strict>::decode(&output, 0) }
        .expect("canonical ZigZag boundary value should decode");
    assert_decoded_eq((value, len), decoded);
}

#[test]
fn test_zig_zag_codec_exposes_unit_bounds() {
    assert_eq!(1, ZigZagCodec::<i8, NonStrict>::MIN_UNITS_PER_VALUE);
    assert_eq!(2, ZigZagCodec::<i8, NonStrict>::MAX_ENCODE_UNITS_PER_VALUE);
    assert_eq!(2, ZigZagCodec::<i8, NonStrict>::MAX_DECODE_UNITS_PER_VALUE);
    assert_eq!(3, ZigZagCodec::<i16, NonStrict>::MAX_ENCODE_UNITS_PER_VALUE);
    assert_eq!(3, ZigZagCodec::<i16, NonStrict>::MAX_DECODE_UNITS_PER_VALUE);
    assert_eq!(5, ZigZagCodec::<i32, NonStrict>::MAX_ENCODE_UNITS_PER_VALUE);
    assert_eq!(5, ZigZagCodec::<i32, NonStrict>::MAX_DECODE_UNITS_PER_VALUE);
    assert_eq!(
        10,
        ZigZagCodec::<i64, NonStrict>::MAX_ENCODE_UNITS_PER_VALUE
    );
    assert_eq!(
        10,
        ZigZagCodec::<i64, NonStrict>::MAX_DECODE_UNITS_PER_VALUE
    );
    assert_eq!(
        19,
        ZigZagCodec::<i128, NonStrict>::MAX_ENCODE_UNITS_PER_VALUE
    );
    assert_eq!(
        19,
        ZigZagCodec::<i128, NonStrict>::MAX_DECODE_UNITS_PER_VALUE
    );
    assert_eq!(
        (isize::BITS as usize).div_ceil(7),
        ZigZagCodec::<isize, Strict>::MAX_ENCODE_UNITS_PER_VALUE
    );
    assert_eq!(
        (isize::BITS as usize).div_ceil(7),
        ZigZagCodec::<isize, Strict>::MAX_DECODE_UNITS_PER_VALUE
    );
}

#[test]
fn test_zig_zag_codec_encodes_7_bit_boundaries() {
    let cases: &[(i16, &[u8])] = &[
        (0, &[0x00]),
        (-1, &[0x01]),
        (1, &[0x02]),
        (63, &[0x7e]),
        (-64, &[0x7f]),
        (64, &[0x80, 0x01]),
        (-65, &[0x81, 0x01]),
        (i16::MAX, &[0xfe, 0xff, 0x03]),
        (i16::MIN, &[0xff, 0xff, 0x03]),
    ];

    for &(value, expected) in cases {
        assert_i16_zig_zag_bytes(value, expected);
    }
}

#[test]
fn test_zig_zag_codec_non_strict_accepts_redundant_values() {
    let decoded = unsafe { ZigZagCodec::<i16, NonStrict>::decode(&[0x80, 0x00], 0) }
        .expect("non-strict ZigZag should accept redundant zero");
    assert_decoded_eq((0, 2), decoded);

    let decoded = unsafe { ZigZagCodec::<i16, NonStrict>::decode(&[0x81, 0x00], 0) }
        .expect("non-strict ZigZag should accept redundant negative one");
    assert_decoded_eq((-1, 2), decoded);
}

#[test]
fn test_zig_zag_codec_roundtrips_all_i8_values() {
    let mut output = [0u8; ZigZagCodec::<i8, NonStrict>::MAX_ENCODE_UNITS_PER_VALUE];
    for value in i8::MIN..=i8::MAX {
        let len = unsafe { ZigZagCodec::<i8, NonStrict>::encode(value, &mut output, 0) };
        let decoded = unsafe { ZigZagCodec::<i8, Strict>::decode(&output[..len], 0) }
            .expect("canonical i8 ZigZag should decode");
        assert_decoded_eq((value, len), decoded);
    }
}

#[test]
fn test_zig_zag_codec_reads_and_writes_values_unchecked() {
    let mut output = [0u8; ZigZagCodec::<i16, NonStrict>::MAX_ENCODE_UNITS_PER_VALUE + 2];
    let len = unsafe { ZigZagCodec::<i16, NonStrict>::encode(-300, &mut output, 1) };

    assert_eq!(2, len);
    assert_eq!([0x00, 0xd7, 0x04, 0x00, 0x00], output);

    let decoded = unsafe { ZigZagCodec::<i16, NonStrict>::decode(&output, 1) }
        .expect("valid i16 should decode");
    assert_decoded_eq((-300, 2), decoded);
}

#[test]
fn test_zig_zag_codec_encodes_and_decodes_through_codec_trait() {
    let mut codec = ZigZagCodec::<i16, NonStrict>::default();
    let mut output = [0u8; ZigZagCodec::<i16, NonStrict>::MAX_ENCODE_UNITS_PER_VALUE + 2];

    assert_eq!(
        ZigZagCodec::<i16, NonStrict>::MIN_UNITS_PER_VALUE,
        <ZigZagCodec<i16, NonStrict> as Codec>::MIN_UNITS_PER_VALUE
    );
    assert_eq!(
        ZigZagCodec::<i16, NonStrict>::MAX_ENCODE_UNITS_PER_VALUE,
        <ZigZagCodec<i16, NonStrict> as Codec>::MAX_ENCODE_UNITS_PER_VALUE
    );
    assert_eq!(
        ZigZagCodec::<i16, NonStrict>::MAX_DECODE_UNITS_PER_VALUE,
        <ZigZagCodec<i16, NonStrict> as Codec>::MAX_DECODE_UNITS_PER_VALUE
    );

    let written = unsafe { Codec::encode(&mut codec, &-300, &mut output, 1) }
        .expect("ZigZag encoding should be infallible");
    assert_eq!(2, written);
    assert_eq!([0x00, 0xd7, 0x04, 0x00, 0x00], output);

    let decoded =
        unsafe { Codec::decode(&mut codec, &output, 1) }.expect("valid ZigZag value should decode");
    assert_decoded_eq((-300, 2), decoded);
}

#[test]
fn test_zig_zag_codec_trait_reports_exact_encoded_lengths() {
    let codec = ZigZagCodec::<i16, NonStrict>::default();

    assert_eq!(1, codec.encode_len(&0));
    assert_eq!(1, codec.encode_len(&-1));
    assert_eq!(1, codec.encode_len(&63));
    assert_eq!(1, codec.encode_len(&-64));
    assert_eq!(2, codec.encode_len(&64));
    assert_eq!(2, codec.encode_len(&-65));
    assert_eq!(3, codec.encode_len(&i16::MIN));
    assert_eq!(3, codec.encode_len(&i16::MAX));
}

#[test]
fn test_zig_zag_codec_trait_encodes_into_exact_length_buffer() {
    let mut codec = ZigZagCodec::<i16, NonStrict>::default();
    let value = -1;
    let mut output = vec![0_u8; codec.encode_len(&value)];

    let written = unsafe { Codec::encode(&mut codec, &value, &mut output, 0) }
        .expect("ZigZag encoding should be infallible");

    assert_eq!(output.len(), written);
    assert_eq!([0x01], output.as_slice());
}

#[test]
fn test_zig_zag_inherent_encode_accepts_exact_length_buffer() {
    let mut output = [0_u8; 1];

    let written = unsafe { ZigZagCodec::<i64, NonStrict>::encode(-1, &mut output, 0) };

    assert_eq!(1, written);
    assert_eq!([0x01], output);
}

#[test]
fn test_zig_zag_codec_trait_decodes_single_byte_value() {
    let mut codec = ZigZagCodec::<i64, NonStrict>::default();
    let input = [0x01u8];

    let decoded = unsafe { Codec::decode(&mut codec, &input, 0) }
        .expect("single-byte ZigZag value should decode");

    assert_decoded_eq((-1, 1), decoded);
}

#[test]
fn test_zig_zag_codec_handles_signed_extremes() {
    let mut output = [0u8; ZigZagCodec::<i128, NonStrict>::MAX_ENCODE_UNITS_PER_VALUE];
    let len = unsafe { ZigZagCodec::<i128, NonStrict>::encode(i128::MIN, &mut output, 0) };

    let decoded = unsafe { ZigZagCodec::<i128, NonStrict>::decode(&output, 0) }
        .expect("valid i128 should decode");
    assert_decoded_eq((i128::MIN, len), decoded);
}

#[test]
fn test_zig_zag_codec_reports_incomplete_values_unchecked() {
    let input = [0x00, 0xd7, 0x04, 0xff];

    let pending = unsafe { ZigZagCodec::<i16, NonStrict>::decode(&input[..2], 1) }
        .expect_err("partial ZigZag value should report incomplete input");
    assert_eq!(Leb128DecodeErrorKind::Incomplete, pending.kind());
    assert_eq!(1, pending.start_index());
    assert_eq!(2, pending.error_index());
    assert_eq!(Some(nonzero(2)), pending.required());
    assert_eq!(Some(1), pending.available());
    assert_eq!(Some(nonzero(1)), pending.additional());

    let decoded = unsafe { ZigZagCodec::<i16, NonStrict>::decode(&input, 1) }
        .expect("complete ZigZag value should decode");
    assert_decoded_eq((-300, 2), decoded);

    let error = unsafe { ZigZagCodec::<i16, Strict>::decode(&[0x80, 0x00], 0) }
        .expect_err("non-canonical ZigZag value should fail");
    assert_eq!(Leb128DecodeErrorKind::NonCanonical, error.kind());
    assert_eq!(0, error.start_index());
    assert_eq!(1, error.error_index());
    assert_eq!(Some(nonzero(2)), error.consumed());
}

#[test]
fn test_zig_zag_codec_rejects_noncanonical_strict_values() {
    let error = unsafe { ZigZagCodec::<i16, Strict>::decode(&[0x80, 0x00, 0x00], 0) }
        .expect_err("non-canonical value should fail");

    assert_eq!(Leb128DecodeErrorKind::NonCanonical, error.kind());
    assert_eq!(0, error.start_index());
    assert_eq!(1, error.error_index());
}
