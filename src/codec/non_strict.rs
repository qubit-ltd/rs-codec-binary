// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use super::Leb128DecodePolicy;

/// Selects LEB128 decoding that accepts non-canonical integer encodings.
///
/// Use this zero-sized policy as the second type parameter of
/// [`crate::Leb128Codec`] or [`crate::ZigZagCodec`] when decoding
/// compatibility-sensitive input. Encoding remains canonical regardless of the
/// selected policy.
///
/// # Examples
///
/// ```
/// use qubit_codec_binary::{Leb128Codec, NonStrict};
///
/// let input = [0x80, 0x00];
/// let (value, consumed) = unsafe {
///     Leb128Codec::<u16, NonStrict>::decode(&input, 0)
/// }
/// .expect("non-canonical zero should decode with the non-strict policy");
/// assert_eq!(0, value);
/// assert_eq!(2, consumed.get());
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct NonStrict;

impl super::internal::sealed::Sealed for NonStrict {}

impl Leb128DecodePolicy for NonStrict {
    /// Whether this policy rejects non-canonical encodings.
    const STRICT: bool = false;
}
