/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/

//! # Qubit Binary Codec
//!
//! Buffer-oriented binary codecs for Rust.
//!
//! This crate provides fixed-width scalar, LEB128, and ZigZag codecs for
//! caller-managed byte buffers. Stream-oriented readers and writers live in
//! `qubit-io-binary`.

mod codec;
pub mod prelude;

pub use codec::{
    BinaryCodec,
    DecodePolicy,
    Leb128Codec,
    Leb128DecodeError,
    Leb128DecodeErrorKind,
    NonStrict,
    Strict,
    ZigZagCodec,
};
pub use qubit_codec::{
    BigEndian,
    ByteOrder,
    ByteOrderSpec,
    Coder,
    CoderProgress,
    CoderStatus,
    LittleEndian,
};
