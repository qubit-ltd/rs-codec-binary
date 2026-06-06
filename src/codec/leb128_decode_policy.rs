// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use super::sealed::Sealed;

/// Describes a type-level LEB128 decoding policy.
///
/// This trait is sealed because the current LEB128 decoder only supports the
/// two built-in canonicality modes represented by [`crate::Strict`] and
/// [`crate::NonStrict`].
pub trait Leb128DecodePolicy: Sealed + Copy + Default {
    /// Whether this policy rejects non-canonical encodings.
    const STRICT: bool;
}
