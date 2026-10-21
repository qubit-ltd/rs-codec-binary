// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use super::Leb128DecodePolicy;

/// Selects LEB128 decoding that rejects non-canonical integer encodings.
///
/// Use this zero-sized policy as the second type parameter of
/// [`crate::Leb128Codec`] or [`crate::ZigZagCodec`] when input must use its
/// shortest valid representation. Encoding remains canonical regardless of
/// the selected policy.
///
/// # Examples
///
/// ```
/// use qubit_codec_binary::{Leb128Codec, Strict};
///
/// let input = [0x80, 0x00];
/// let result = unsafe { Leb128Codec::<u16, Strict>::decode(&input, 0) };
/// assert!(result.is_err());
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Strict;

impl super::internal::sealed::Sealed for Strict {}

impl Leb128DecodePolicy for Strict {
    /// Whether this policy rejects non-canonical encodings.
    const STRICT: bool = true;
}
