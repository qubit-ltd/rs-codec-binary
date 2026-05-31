/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
******************************************************************************/
use thiserror::Error;

use qubit_codec::{
    DecodeErrorInfo,
    DecodeFailure,
};

use crate::Leb128DecodeErrorKind;

/// Error reported while decoding a LEB128 integer from a byte buffer.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[error("{kind}")]
pub struct Leb128DecodeError {
    kind: Leb128DecodeErrorKind,
    index: usize,
    consumed: usize,
    required: Option<usize>,
    available: Option<usize>,
}

impl Leb128DecodeError {
    /// Creates a LEB128 decoding error.
    ///
    /// # Parameters
    ///
    /// - `kind`: Failure category.
    /// - `index`: Absolute byte index at which the failure was detected.
    ///
    /// # Returns
    ///
    /// Returns a decoding error carrying the supplied context.
    pub const fn new(kind: Leb128DecodeErrorKind, index: usize) -> Self {
        Self {
            kind,
            index,
            consumed: 1,
            required: None,
            available: None,
        }
    }

    /// Creates an incomplete-input decoding error.
    ///
    /// # Parameters
    ///
    /// - `index`: Byte index where the incomplete value starts.
    /// - `required`: Total bytes required from `index`.
    /// - `available`: Bytes currently available from `index`.
    ///
    /// # Returns
    ///
    /// Returns an error carrying incomplete-input context.
    pub const fn incomplete(index: usize, required: usize, available: usize) -> Self {
        Self {
            kind: Leb128DecodeErrorKind::Incomplete,
            index,
            consumed: 0,
            required: Some(required),
            available: Some(available),
        }
    }

    /// Creates a malformed-input decoding error.
    ///
    /// # Parameters
    ///
    /// - `index`: Byte index at which the malformed input was detected.
    /// - `consumed`: Bytes the caller may consume to make progress.
    ///
    /// # Returns
    ///
    /// Returns an error carrying malformed-input context.
    pub const fn malformed(index: usize, consumed: usize) -> Self {
        Self {
            kind: Leb128DecodeErrorKind::Malformed,
            index,
            consumed,
            required: None,
            available: None,
        }
    }

    /// Creates a non-canonical-input decoding error.
    ///
    /// # Parameters
    ///
    /// - `index`: Byte index where the non-canonical value starts.
    /// - `consumed`: Bytes the caller may consume to make progress.
    ///
    /// # Returns
    ///
    /// Returns an error carrying non-canonical-input context.
    pub const fn noncanonical(index: usize, consumed: usize) -> Self {
        Self {
            kind: Leb128DecodeErrorKind::NonCanonical,
            index,
            consumed,
            required: None,
            available: None,
        }
    }

    /// Returns the decoding error kind.
    #[must_use]
    pub const fn kind(self) -> Leb128DecodeErrorKind {
        self.kind
    }

    /// Returns the absolute byte index associated with this error.
    #[must_use]
    pub const fn index(self) -> usize {
        self.index
    }

    /// Returns bytes that may be consumed after an invalid-input error.
    ///
    /// # Returns
    ///
    /// Returns `Some(consumed)` for invalid input, or `None` for incomplete
    /// input.
    #[must_use]
    pub const fn consumed(self) -> Option<usize> {
        match self.kind {
            Leb128DecodeErrorKind::Incomplete => None,
            Leb128DecodeErrorKind::Malformed | Leb128DecodeErrorKind::NonCanonical => Some(self.consumed),
        }
    }

    /// Returns total bytes required to finish an incomplete value.
    ///
    /// # Returns
    ///
    /// Returns `Some(required)` for incomplete input, or `None` otherwise.
    #[must_use]
    pub const fn required(self) -> Option<usize> {
        self.required
    }

    /// Returns bytes available for an incomplete value.
    ///
    /// # Returns
    ///
    /// Returns `Some(available)` for incomplete input, or `None` otherwise.
    #[must_use]
    pub const fn available(self) -> Option<usize> {
        self.available
    }
}

impl DecodeErrorInfo for Leb128DecodeError {
    /// Returns buffered-decode metadata for this LEB128 error.
    fn failure(&self) -> DecodeFailure {
        match self.kind {
            Leb128DecodeErrorKind::Incomplete => DecodeFailure::Incomplete {
                required_total: self.required.unwrap_or(0),
                available: self.available.unwrap_or(0),
            },
            Leb128DecodeErrorKind::Malformed | Leb128DecodeErrorKind::NonCanonical => DecodeFailure::Invalid {
                consumed: self.consumed.max(1),
            },
        }
    }
}
