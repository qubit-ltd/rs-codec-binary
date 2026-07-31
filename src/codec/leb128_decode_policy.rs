// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use super::internal::sealed::Sealed;

/// Describes a type-level LEB128 decoding policy.
///
/// This trait is sealed because the current LEB128 decoder only supports the
/// two built-in canonicality modes represented by [`crate::Strict`] and
/// [`crate::NonStrict`].
///
/// ```compile_fail
/// use qubit_codec_binary::Leb128DecodePolicy;
///
/// #[derive(Clone, Copy, Default)]
/// struct ExternalPolicy;
///
/// impl Leb128DecodePolicy for ExternalPolicy {
///     const STRICT: bool = false;
/// }
/// ```
pub trait Leb128DecodePolicy: Sealed + Copy + Default {
    /// Whether this policy rejects non-canonical encodings.
    const STRICT: bool;
}
