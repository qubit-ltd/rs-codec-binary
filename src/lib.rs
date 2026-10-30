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
//!
//! # Checked single-value conversion
//!
//! For ordinary one-value conversion, prefer [`qubit_codec::CodecValueEncoder`]
//! and [`qubit_codec::CodecValueDecoder`]. These checked adapters own the
//! temporary buffer and keep the unsafe boundary inside the adapter:
//!
//! ```
//! use qubit_codec::{CodecValueDecoder, CodecValueEncoder};
//! use qubit_codec_binary::{Leb128Codec, NonStrict, Strict};
//!
//! let mut encoder = CodecValueEncoder::new(Leb128Codec::<u64, NonStrict>::default());
//! let encoded = encoder.encode(&300).expect("u64 is always encodable");
//!
//! let mut decoder = CodecValueDecoder::new(Leb128Codec::<u64, Strict>::default());
//! let decoded = decoder.decode(&encoded).expect("the encoding is canonical");
//! assert_eq!(300, decoded);
//! ```
//!
//! Each adapter handles exactly one value. The decoder rejects empty input and
//! trailing bytes, and does not return a consumed prefix. Encoding is always
//! canonical; `NonStrict` is the conventional policy marker for this
//! encode-only path, while `Strict` verifies the canonical encoded value.
//!
//! # API navigation
//!
//! For fixed-width fields, see [`BinaryCodec`] and
//! [`qubit_codec::BigEndian`], [`qubit_codec::LittleEndian`], and
//! [`qubit_codec::NativeEndian`]. For variable-width fields, see
//! [`Leb128Codec`], [`ZigZagCodec`], [`Strict`], and [`NonStrict`].
//!
//! Direct calls to the low-level `unsafe` methods require the surrounding code
//! to guarantee valid input/output indices and sufficient readable/writable
//! capacity. See [`qubit_codec::Codec`] for the low-level contract.
//!
//! Read the [English user guide](https://github.com/qubit-ltd/rs-codec-binary/blob/main/doc/user_guide.md)
//! or the [中文用户指南](https://github.com/qubit-ltd/rs-codec-binary/blob/main/doc/user_guide.zh_CN.md).

mod codec;

#[doc = include_str!("../README.md")]
#[cfg(doctest)]
mod readme_examples {}

#[doc = include_str!("../README.zh_CN.md")]
#[cfg(doctest)]
mod readme_zh_cn_examples {}

#[doc = include_str!("../doc/user_guide.md")]
#[cfg(doctest)]
mod user_guide_examples {}

#[doc = include_str!("../doc/user_guide.zh_CN.md")]
#[cfg(doctest)]
mod user_guide_zh_cn_examples {}

pub use codec::BinaryCodec;
pub use codec::Leb128Codec;
pub use codec::Leb128DecodeError;
pub use codec::Leb128DecodeErrorKind;
pub use codec::Leb128DecodePolicy;
pub use codec::NonStrict;
pub use codec::Strict;
pub use codec::ZigZagCodec;
