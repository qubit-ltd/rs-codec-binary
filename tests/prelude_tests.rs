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
    ByteOrder,
    ByteOrderSpec,
    DecodeErrorInfo,
    DecodeFailure,
    Leb128Codec,
    Leb128DecodeError,
    NonStrict,
    ZigZagCodec,
};

#[test]
fn test_prelude_imports_binary_codec_types_and_core_markers() {
    assert_eq!(ByteOrder::BigEndian, BigEndian::ORDER);
    let _decoder_error: Option<Leb128DecodeError> = None;

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
