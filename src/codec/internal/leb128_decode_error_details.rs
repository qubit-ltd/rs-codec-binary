// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use core::num::NonZeroUsize;

use crate::Leb128DecodeErrorKind;

/// Stores the additional context associated with a LEB128 decode error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::codec) enum Leb128DecodeErrorDetails {
    /// Describes an input prefix that needs more bytes before decoding can
    /// continue.
    Incomplete {
        /// Minimum total bytes required from the value start.
        required: NonZeroUsize,
        /// Bytes currently available from the value start.
        available: usize,
    },
    /// Describes an input sequence that cannot be accepted by the decoder.
    Invalid {
        /// Specific reason why the input was rejected.
        kind: Leb128DecodeErrorKind,
        /// Non-zero bytes that the caller may consume to make progress.
        consumed: NonZeroUsize,
    },
}
