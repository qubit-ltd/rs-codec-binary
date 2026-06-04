/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/

/// Describes a type-level LEB128 decoding policy.
///
/// This trait is sealed because the current LEB128 decoder only supports the
/// two built-in canonicality modes represented by [`crate::Strict`] and
/// [`crate::NonStrict`].
pub trait Leb128DecodePolicy: sealed::Sealed + Copy + Default {
    /// Whether this policy rejects non-canonical encodings.
    const STRICT: bool;
}

pub(crate) mod sealed {
    /// Marker trait preventing external LEB128 decode policy implementations.
    pub trait Sealed {}
}
