use qubit_codec::Codec;
use qubit_codec_binary::{
    Leb128DecodeErrorKind,
    NonStrict,
    Strict,
    ZigZagCodec,
};

#[test]
fn test_zig_zag_codec_exposes_required_min_buffer_len() {
    assert_eq!(2, ZigZagCodec::<i8, NonStrict>::REQUIRED_MIN_BUFFER_LEN);
    assert_eq!(3, ZigZagCodec::<i16, NonStrict>::REQUIRED_MIN_BUFFER_LEN);
    assert_eq!(5, ZigZagCodec::<i32, NonStrict>::REQUIRED_MIN_BUFFER_LEN);
    assert_eq!(10, ZigZagCodec::<i64, NonStrict>::REQUIRED_MIN_BUFFER_LEN);
    assert_eq!(19, ZigZagCodec::<i128, NonStrict>::REQUIRED_MIN_BUFFER_LEN);
    assert_eq!(
        (isize::BITS as usize).div_ceil(7),
        ZigZagCodec::<isize, Strict>::REQUIRED_MIN_BUFFER_LEN
    );
}

#[test]
fn test_zig_zag_codec_reads_and_writes_values_unchecked() {
    let mut output = [0u8; ZigZagCodec::<i16, NonStrict>::REQUIRED_MIN_BUFFER_LEN + 2];
    let len = unsafe { ZigZagCodec::<i16, NonStrict>::encode_unchecked(-300, &mut output, 1) };

    assert_eq!(2, len);
    assert_eq!([0x00, 0xd7, 0x04, 0x00, 0x00], output);

    let decoded =
        unsafe { ZigZagCodec::<i16, NonStrict>::decode_unchecked(&output, 1) }.expect("valid i16 should decode");
    assert_eq!((-300, 2), decoded);
}

#[test]
fn test_zig_zag_codec_encodes_and_decodes_through_codec_trait() {
    let codec = ZigZagCodec::<i16, NonStrict>::default();
    let mut output = [0u8; ZigZagCodec::<i16, NonStrict>::REQUIRED_MIN_BUFFER_LEN + 2];

    assert_eq!(1, codec.min_units_per_value());
    assert_eq!(
        ZigZagCodec::<i16, NonStrict>::REQUIRED_MIN_BUFFER_LEN,
        codec.max_units_per_value()
    );

    let written =
        unsafe { Codec::encode_unchecked(&codec, -300, &mut output, 1) }.expect("ZigZag encoding should be infallible");
    assert_eq!(2, written);
    assert_eq!([0x00, 0xd7, 0x04, 0x00, 0x00], output);

    let decoded = unsafe { Codec::decode_unchecked(&codec, &output, 1) }.expect("valid ZigZag value should decode");
    assert_eq!((-300, 2), decoded);
}

#[test]
fn test_zig_zag_codec_handles_signed_extremes() {
    let mut output = [0u8; ZigZagCodec::<i128, NonStrict>::REQUIRED_MIN_BUFFER_LEN];
    let len = unsafe { ZigZagCodec::<i128, NonStrict>::encode_unchecked(i128::MIN, &mut output, 0) };

    let decoded =
        unsafe { ZigZagCodec::<i128, NonStrict>::decode_unchecked(&output, 0) }.expect("valid i128 should decode");
    assert_eq!((i128::MIN, len), decoded);
}

#[test]
fn test_zig_zag_codec_reads_available_values_unchecked() {
    let input = [0x00, 0xd7, 0x04, 0xff];

    let pending = unsafe { ZigZagCodec::<i16, NonStrict>::decode_available_unchecked(&input, 1, 1) }
        .expect("partial ZigZag value should not fail");
    assert_eq!(None, pending);

    let decoded = unsafe { ZigZagCodec::<i16, NonStrict>::decode_available_unchecked(&input, 1, 2) }
        .expect("complete ZigZag value should decode");
    assert_eq!(Some((-300, 2)), decoded);

    let error = unsafe { ZigZagCodec::<i16, Strict>::decode_available_unchecked(&[0x80, 0x00], 0, 2) }
        .expect_err("non-canonical ZigZag value should fail");
    assert_eq!(Leb128DecodeErrorKind::NonCanonical, error.0.kind());
    assert_eq!(2, error.1);
}

#[test]
fn test_zig_zag_codec_rejects_noncanonical_strict_values() {
    let error = unsafe { ZigZagCodec::<i16, Strict>::decode_unchecked(&[0x80, 0x00, 0x00], 0) }
        .expect_err("non-canonical value should fail");

    assert_eq!(Leb128DecodeErrorKind::NonCanonical, error.kind());
    assert_eq!(0, error.index());
}
