/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/

//! Common binary codec types and shared core primitives.
//!
//! Importing this module brings fixed-width, LEB128, ZigZag, byte-order, and
//! buffer conversion types into scope.

pub use crate::{
    BigEndian,
    BinaryCodec,
    BufferedConverter,
    BufferedDecoder,
    BufferedEncoder,
    ByteOrder,
    ByteOrderSpec,
    Codec,
    CodecBufferedDecoder,
    CodecBufferedEncoder,
    CodecDecodeError,
    CodecEncodeError,
    CodecValueEncoder,
    DecodeErrorFactory,
    DecodeErrorInfo,
    DecodeFailure,
    DecodePolicy,
    EncodeErrorFactory,
    EncodePlan,
    Leb128Codec,
    Leb128DecodeError,
    Leb128DecodeErrorKind,
    LittleEndian,
    NonStrict,
    Strict,
    TranscodeProgress,
    TranscodeStatus,
    Transcoder,
    ValueDecoder,
    ValueEncoder,
    ZigZagCodec,
};
