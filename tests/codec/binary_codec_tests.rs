use qubit_codec::Codec;
use qubit_codec_binary::{
    BigEndian,
    BinaryCodec,
    LittleEndian,
};

#[test]
fn test_binary_codec_exposes_required_min_buffer_len() {
    assert_eq!(1, BinaryCodec::<u8, BigEndian>::REQUIRED_MIN_BUFFER_LEN);
    assert_eq!(1, BinaryCodec::<i8, LittleEndian>::REQUIRED_MIN_BUFFER_LEN);
    assert_eq!(2, BinaryCodec::<u16, BigEndian>::REQUIRED_MIN_BUFFER_LEN);
    assert_eq!(4, BinaryCodec::<u32, LittleEndian>::REQUIRED_MIN_BUFFER_LEN);
    assert_eq!(8, BinaryCodec::<u64, BigEndian>::REQUIRED_MIN_BUFFER_LEN);
    assert_eq!(16, BinaryCodec::<u128, LittleEndian>::REQUIRED_MIN_BUFFER_LEN);
    assert_eq!(4, BinaryCodec::<f32, BigEndian>::REQUIRED_MIN_BUFFER_LEN);
    assert_eq!(8, BinaryCodec::<f64, LittleEndian>::REQUIRED_MIN_BUFFER_LEN);
}

#[test]
fn test_binary_codec_reads_from_explicit_index_unchecked() {
    let input = [0xaa, 0x12, 0x34, 0x56, 0x78, 0xbb];

    let decoded = unsafe { BinaryCodec::<u32, BigEndian>::decode_unchecked(&input, 1) };
    assert_eq!((0x1234_5678, 4), decoded);

    let decoded = unsafe { BinaryCodec::<u32, LittleEndian>::decode_unchecked(&input, 1) };
    assert_eq!((0x7856_3412, 4), decoded);
}

#[test]
fn test_binary_codec_writes_to_explicit_index_unchecked() {
    let mut output = [0xaa, 0, 0, 0, 0, 0xbb];

    unsafe {
        assert_eq!(
            4,
            BinaryCodec::<u32, BigEndian>::encode_unchecked(0x1234_5678, &mut output, 1)
        );
    }
    assert_eq!([0xaa, 0x12, 0x34, 0x56, 0x78, 0xbb], output);

    unsafe {
        assert_eq!(
            4,
            BinaryCodec::<u32, LittleEndian>::encode_unchecked(0x1234_5678, &mut output, 1)
        );
    }
    assert_eq!([0xaa, 0x78, 0x56, 0x34, 0x12, 0xbb], output);
}

#[test]
fn test_binary_codec_encodes_and_decodes_through_codec_trait() {
    let codec = BinaryCodec::<u32, BigEndian>::default();
    let mut output = [0xaa, 0, 0, 0, 0, 0xbb];

    assert_eq!(4, codec.min_units_per_value());
    assert_eq!(4, codec.max_units_per_value());

    let written = unsafe { Codec::encode_unchecked(&codec, 0x1234_5678, &mut output, 1) }
        .expect("fixed-width encoding should be infallible");
    assert_eq!(4, written);
    assert_eq!([0xaa, 0x12, 0x34, 0x56, 0x78, 0xbb], output);

    let decoded =
        unsafe { Codec::decode_unchecked(&codec, &output, 1) }.expect("fixed-width decoding should be infallible");
    assert_eq!((0x1234_5678, 4), decoded);
}

#[test]
fn test_binary_codec_trait_covers_byte_and_little_endian_groups() {
    let unsigned_byte = BinaryCodec::<u8, BigEndian>::default();
    let signed_byte = BinaryCodec::<i8, LittleEndian>::default();
    let little_integer = BinaryCodec::<u16, LittleEndian>::default();
    let big_float = BinaryCodec::<f32, BigEndian>::default();
    let little_float = BinaryCodec::<f64, LittleEndian>::default();
    let mut output = [0u8; 24];

    assert_eq!(1, unsigned_byte.min_units_per_value());
    assert_eq!(1, unsigned_byte.max_units_per_value());
    assert_eq!(1, signed_byte.min_units_per_value());
    assert_eq!(1, signed_byte.max_units_per_value());
    assert_eq!(2, little_integer.min_units_per_value());
    assert_eq!(2, little_integer.max_units_per_value());
    assert_eq!(4, big_float.min_units_per_value());
    assert_eq!(4, big_float.max_units_per_value());
    assert_eq!(8, little_float.min_units_per_value());
    assert_eq!(8, little_float.max_units_per_value());

    assert_eq!(
        1,
        unsafe { Codec::encode_unchecked(&unsigned_byte, 0x7f, &mut output, 0) }
            .expect("u8 encoding should be infallible")
    );
    assert_eq!(
        1,
        unsafe { Codec::encode_unchecked(&signed_byte, -1, &mut output, 1) }.expect("i8 encoding should be infallible")
    );
    assert_eq!(
        2,
        unsafe { Codec::encode_unchecked(&little_integer, 0x1234, &mut output, 2) }
            .expect("little-endian integer encoding should be infallible")
    );
    assert_eq!(
        4,
        unsafe { Codec::encode_unchecked(&big_float, 12.5, &mut output, 4) }
            .expect("big-endian float encoding should be infallible")
    );
    assert_eq!(
        8,
        unsafe { Codec::encode_unchecked(&little_float, -25.25, &mut output, 8) }
            .expect("little-endian float encoding should be infallible")
    );

    assert_eq!(
        (0x7f, 1),
        unsafe { Codec::decode_unchecked(&unsigned_byte, &output, 0) }.expect("u8 decoding should be infallible")
    );
    assert_eq!(
        (-1, 1),
        unsafe { Codec::decode_unchecked(&signed_byte, &output, 1) }.expect("i8 decoding should be infallible")
    );
    assert_eq!(
        (0x1234, 2),
        unsafe { Codec::decode_unchecked(&little_integer, &output, 2) }
            .expect("little-endian integer decoding should be infallible")
    );
    assert_eq!(
        (12.5, 4),
        unsafe { Codec::decode_unchecked(&big_float, &output, 4) }
            .expect("big-endian float decoding should be infallible")
    );
    assert_eq!(
        (-25.25, 8),
        unsafe { Codec::decode_unchecked(&little_float, &output, 8) }
            .expect("little-endian float decoding should be infallible")
    );
}

#[test]
fn test_binary_codec_handles_byte_signed_and_float_values() {
    let mut output = [0u8; 16];

    unsafe {
        assert_eq!(1, BinaryCodec::<u8, BigEndian>::encode_unchecked(0x7f, &mut output, 0));
        assert_eq!(1, BinaryCodec::<i8, LittleEndian>::encode_unchecked(-1, &mut output, 1));
        assert_eq!(4, BinaryCodec::<f32, BigEndian>::encode_unchecked(12.5, &mut output, 2));
        assert_eq!(
            8,
            BinaryCodec::<f64, LittleEndian>::encode_unchecked(-25.25, &mut output, 6)
        );
    }

    assert_eq!((0x7f, 1), unsafe {
        BinaryCodec::<u8, LittleEndian>::decode_unchecked(&output, 0)
    });
    assert_eq!((-1, 1), unsafe {
        BinaryCodec::<i8, BigEndian>::decode_unchecked(&output, 1)
    });
    assert_eq!((12.5, 4), unsafe {
        BinaryCodec::<f32, BigEndian>::decode_unchecked(&output, 2)
    });
    assert_eq!((-25.25, 8), unsafe {
        BinaryCodec::<f64, LittleEndian>::decode_unchecked(&output, 6)
    });
}
