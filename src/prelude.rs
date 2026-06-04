/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/

//! Common binary codec types and essential core primitives.
//!
//! Importing this module brings fixed-width, LEB128, ZigZag, byte-order, and
//! core codec types into scope.

pub use crate::{
    BigEndian,
    BinaryCodec,
    ByteOrder,
    ByteOrderSpec,
    Codec,
    Leb128Codec,
    Leb128DecodeError,
    Leb128DecodeErrorKind,
    Leb128DecodePolicy,
    LittleEndian,
    NonStrict,
    Strict,
    ZigZagCodec,
};
