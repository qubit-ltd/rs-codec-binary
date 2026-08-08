// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! # Qubit Binary Codec
//!
//! Buffer-oriented binary codecs for Rust.
//!
//! This crate provides fixed-width scalar, LEB128, and ZigZag codecs for
//! caller-managed byte buffers. Stream-oriented readers and writers live in
//! `qubit-io-binary`.

mod codec;

pub use codec::BinaryCodec;
pub use codec::Leb128Codec;
pub use codec::Leb128DecodeError;
pub use codec::Leb128DecodeErrorKind;
pub use codec::Leb128DecodePolicy;
pub use codec::NonStrict;
pub use codec::Strict;
pub use codec::ZigZagCodec;
