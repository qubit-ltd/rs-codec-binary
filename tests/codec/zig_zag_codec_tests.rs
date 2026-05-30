use qubit_codec::{
    Codec,
    DecodeErrorInfo,
    DecodeFailure,
};
use qubit_codec_binary::{
    Leb128DecodeErrorKind,
    NonStrict,
    Strict,
    ZigZagCodec,
};

use super::assert_decoded_eq;

#[test]
fn test_zig_zag_codec_exposes_unit_bounds() {
    assert_eq!(1, ZigZagCodec::<i8, NonStrict>::MIN_UNITS_PER_VALUE);
    assert_eq!(2, ZigZagCodec::<i8, NonStrict>::MAX_UNITS_PER_VALUE);
    assert_eq!(3, ZigZagCodec::<i16, NonStrict>::MAX_UNITS_PER_VALUE);
    assert_eq!(5, ZigZagCodec::<i32, NonStrict>::MAX_UNITS_PER_VALUE);
    assert_eq!(10, ZigZagCodec::<i64, NonStrict>::MAX_UNITS_PER_VALUE);
    assert_eq!(19, ZigZagCodec::<i128, NonStrict>::MAX_UNITS_PER_VALUE);
    assert_eq!(
        (isize::BITS as usize).div_ceil(7),
        ZigZagCodec::<isize, Strict>::MAX_UNITS_PER_VALUE
    );
}

#[test]
fn test_zig_zag_codec_reads_and_writes_values_unchecked() {
    let mut output = [0u8; ZigZagCodec::<i16, NonStrict>::MAX_UNITS_PER_VALUE + 2];
    let len = unsafe { ZigZagCodec::<i16, NonStrict>::encode_unchecked(-300, &mut output, 1) };

    assert_eq!(2, len);
    assert_eq!([0x00, 0xd7, 0x04, 0x00, 0x00], output);

    let decoded =
        unsafe { ZigZagCodec::<i16, NonStrict>::decode_unchecked(&output, 1) }.expect("valid i16 should decode");
    assert_decoded_eq((-300, 2), decoded);
}

#[test]
fn test_zig_zag_codec_encodes_and_decodes_through_codec_trait() {
    let codec = ZigZagCodec::<i16, NonStrict>::default();
    let mut output = [0u8; ZigZagCodec::<i16, NonStrict>::MAX_UNITS_PER_VALUE + 2];

    assert_eq!(
        ZigZagCodec::<i16, NonStrict>::MIN_UNITS_PER_VALUE,
        codec.min_units_per_value().get()
    );
    assert_eq!(
        ZigZagCodec::<i16, NonStrict>::MAX_UNITS_PER_VALUE,
        codec.max_units_per_value().get()
    );

    let written = unsafe { Codec::encode_unchecked(&codec, &-300, &mut output, 1) }
        .expect("ZigZag encoding should be infallible");
    assert_eq!(2, written);
    assert_eq!([0x00, 0xd7, 0x04, 0x00, 0x00], output);

    let decoded = unsafe { Codec::decode_unchecked(&codec, &output, 1) }.expect("valid ZigZag value should decode");
    assert_decoded_eq((-300, 2), decoded);
}

#[test]
fn test_zig_zag_codec_trait_decodes_single_byte_value() {
    let codec = ZigZagCodec::<i64, NonStrict>::default();
    let input = [0x01u8];

    let decoded =
        unsafe { Codec::decode_unchecked(&codec, &input, 0) }.expect("single-byte ZigZag value should decode");

    assert_decoded_eq((-1, 1), decoded);
}

#[test]
fn test_zig_zag_codec_handles_signed_extremes() {
    let mut output = [0u8; ZigZagCodec::<i128, NonStrict>::MAX_UNITS_PER_VALUE];
    let len = unsafe { ZigZagCodec::<i128, NonStrict>::encode_unchecked(i128::MIN, &mut output, 0) };

    let decoded =
        unsafe { ZigZagCodec::<i128, NonStrict>::decode_unchecked(&output, 0) }.expect("valid i128 should decode");
    assert_decoded_eq((i128::MIN, len), decoded);
}

#[test]
fn test_zig_zag_codec_reports_incomplete_values_unchecked() {
    let input = [0x00, 0xd7, 0x04, 0xff];

    let pending = unsafe { ZigZagCodec::<i16, NonStrict>::decode_unchecked(&input[..2], 1) }
        .expect_err("partial ZigZag value should report incomplete input");
    assert_eq!(Leb128DecodeErrorKind::Incomplete, pending.kind());
    assert_eq!(Some(2), pending.required());
    assert_eq!(Some(1), pending.available());
    assert_eq!(
        DecodeFailure::Incomplete {
            required_total: 2,
            available: 1,
        },
        pending.failure(),
    );

    let decoded = unsafe { ZigZagCodec::<i16, NonStrict>::decode_unchecked(&input, 1) }
        .expect("complete ZigZag value should decode");
    assert_decoded_eq((-300, 2), decoded);

    let error = unsafe { ZigZagCodec::<i16, Strict>::decode_unchecked(&[0x80, 0x00], 0) }
        .expect_err("non-canonical ZigZag value should fail");
    assert_eq!(Leb128DecodeErrorKind::NonCanonical, error.kind());
    assert_eq!(Some(2), error.consumed());
    assert_eq!(DecodeFailure::Invalid { consumed: 2 }, error.failure());
}

#[test]
fn test_zig_zag_codec_rejects_noncanonical_strict_values() {
    let error = unsafe { ZigZagCodec::<i16, Strict>::decode_unchecked(&[0x80, 0x00, 0x00], 0) }
        .expect_err("non-canonical value should fail");

    assert_eq!(Leb128DecodeErrorKind::NonCanonical, error.kind());
    assert_eq!(0, error.index());
}
