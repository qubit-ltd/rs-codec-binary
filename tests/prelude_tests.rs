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
    Leb128Codec,
    NonStrict,
    ZigZagCodec,
};

#[test]
fn test_prelude_imports_binary_codec_types_and_core_markers() {
    assert_eq!(ByteOrder::BigEndian, BigEndian::ORDER);

    let mut fixed = [0_u8; BinaryCodec::<u32, BigEndian>::REQUIRED_MIN_BUFFER_LEN];
    unsafe {
        BinaryCodec::<u32, BigEndian>::write_unchecked(&mut fixed, 0, 0x0102_0304);
    }
    assert_eq!([1, 2, 3, 4], fixed);

    let mut compact = [0_u8; Leb128Codec::<u64, NonStrict>::REQUIRED_MIN_BUFFER_LEN];
    let written = unsafe { Leb128Codec::<u64, NonStrict>::write_unchecked(&mut compact, 0, 300) };
    assert_eq!(2, written);

    let mut zigzag = [0_u8; ZigZagCodec::<i64, NonStrict>::REQUIRED_MIN_BUFFER_LEN];
    let written = unsafe { ZigZagCodec::<i64, NonStrict>::write_unchecked(&mut zigzag, 0, -42) };
    assert_eq!(1, written);
}
