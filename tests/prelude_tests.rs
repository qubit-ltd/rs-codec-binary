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
    CodecBufferedDecoder,
    CodecBufferedEncoder,
    CodecDecodeError,
    CodecEncodeError,
    CodecValueEncoder,
    DecodeErrorFactory,
    DecodeErrorInfo,
    DecodeFailure,
    EncodeErrorFactory,
    EncodePlan,
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
    fn _accept_codec_buffered_decoder<T: BufferedDecoder<u8, u8>>() {}
    fn _accept_codec_buffered_encoder<T: BufferedEncoder<u8, u8>>() {}
    fn _accept_buffered_decode_engine<T>() {}
    fn _accept_buffered_encode_engine<T>() {}

    assert_eq!(ByteOrder::BigEndian, BigEndian::ORDER);
    _accept_codec_value_encoder::<CodecValueEncoder<BinaryCodec<u8, BigEndian>, u8, u8>>();
    _accept_codec_buffered_decoder::<CodecBufferedDecoder<BinaryCodec<u8, BigEndian>, u8>>();
    _accept_codec_buffered_encoder::<CodecBufferedEncoder<BinaryCodec<u8, BigEndian>>>();
    _accept_buffered_decode_engine::<qubit_codec_binary::BufferedDecodeEngine<BinaryCodec<u8, BigEndian>, (), u8>>();
    _accept_buffered_encode_engine::<qubit_codec_binary::BufferedEncodeEngine<BinaryCodec<u8, BigEndian>, ()>>();

    let plan = EncodePlan::new(1, ());
    assert_eq!(1, plan.max_output_units);
    let binary_codec = BinaryCodec::<u8, BigEndian>::default();
    let encode_error = <CodecEncodeError<core::convert::Infallible> as EncodeErrorFactory<
        BinaryCodec<u8, BigEndian>,
    >>::invalid_input_index(&binary_codec, 2, 1);
    assert!(matches!(encode_error, CodecEncodeError::InvalidInputIndex { .. }));
    let decode_error = <CodecDecodeError<core::convert::Infallible> as DecodeErrorFactory<
        BinaryCodec<u8, BigEndian>,
    >>::invalid_input_index(&binary_codec, 2, 1);
    assert!(matches!(decode_error, CodecDecodeError::InvalidInputIndex { .. }));

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
    let (decoded, consumed) = unsafe { Leb128Codec::<u64, NonStrict>::decode_unchecked(&compact[..written], 0) }
        .expect("LEB128 value should decode");
    assert_eq!(300, decoded);
    assert_eq!(2, consumed.get());

    fn _accept_decode_error_info<T: DecodeErrorInfo>() {}
    _accept_decode_error_info::<Leb128DecodeError>();
    assert_eq!(Some(1), DecodeFailure::Invalid { consumed: 1 }.invalid_consumed());

    let mut zigzag = [0_u8; ZigZagCodec::<i64, NonStrict>::MAX_UNITS_PER_VALUE];
    let written = unsafe { ZigZagCodec::<i64, NonStrict>::encode_unchecked(-42, &mut zigzag, 0) };
    assert_eq!(1, written);
}
