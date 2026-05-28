/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/

use qubit_codec_binary::prelude::{
    BigEndian,
    BinaryCodec,
    BufferedConverter,
    BufferedDecoder,
    BufferedEncoder,
    ByteOrder,
    ByteOrderSpec,
    CodecBufferedEncoder,
    CodecValueEncoder,
    DecodeErrorInfo,
    DecodeFailure,
    Leb128Codec,
    Leb128DecodeError,
    NonStrict,
    ValueDecoder,
    ValueEncoder,
    ZigZagCodec,
};

#[test]
fn test_prelude_imports_binary_codec_types_and_core_markers() {
    fn _accept_buffered_encoder<T: BufferedEncoder<u64, u8>>() {}
    fn _accept_buffered_decoder<T: BufferedDecoder<u8, u64>>() {}
    fn _accept_buffered_converter<T: BufferedConverter<u8, u8>>() {}
    fn _accept_codec_value_encoder<T: ValueEncoder<u8, Output = Vec<u8>>>() {}
    fn _accept_codec_buffered_encoder<T: BufferedEncoder<u8, u8>>() {}

    assert_eq!(ByteOrder::BigEndian, BigEndian::ORDER);
    _accept_codec_value_encoder::<CodecValueEncoder<BinaryCodec<u8, BigEndian>, u8, u8>>();
    _accept_codec_buffered_encoder::<CodecBufferedEncoder<BinaryCodec<u8, BigEndian>>>();

    let _encoder_trait: Option<&dyn ValueEncoder<u64, Output = Vec<u8>, Error = core::convert::Infallible>> = None;
    let _decoder_trait: Option<&dyn ValueDecoder<[u8], Output = u64, Error = Leb128DecodeError>> = None;

    let mut fixed = [0_u8; BinaryCodec::<u32, BigEndian>::REQUIRED_MIN_BUFFER_LEN];
    unsafe {
        BinaryCodec::<u32, BigEndian>::encode_unchecked(0x0102_0304, &mut fixed, 0);
    }
    assert_eq!([1, 2, 3, 4], fixed);

    let mut compact = [0_u8; Leb128Codec::<u64, NonStrict>::MAX_UNITS_PER_VALUE];
    let written = unsafe { Leb128Codec::<u64, NonStrict>::encode_unchecked(300, &mut compact, 0) };
    assert_eq!(2, written);
    let decoded = unsafe { Leb128Codec::<u64, NonStrict>::decode_unchecked(&compact[..written], 0) }
        .expect("LEB128 value should decode");
    assert_eq!((300, 2), decoded);

    fn _accept_decode_error_info<T: DecodeErrorInfo>() {}
    _accept_decode_error_info::<Leb128DecodeError>();
    assert_eq!(Some(1), DecodeFailure::Invalid { consumed: 1 }.invalid_consumed());

    let mut zigzag = [0_u8; ZigZagCodec::<i64, NonStrict>::MAX_UNITS_PER_VALUE];
    let written = unsafe { ZigZagCodec::<i64, NonStrict>::encode_unchecked(-42, &mut zigzag, 0) };
    assert_eq!(1, written);
}
