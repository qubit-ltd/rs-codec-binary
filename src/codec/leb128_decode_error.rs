/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
******************************************************************************/
use core::num::NonZeroUsize;

use thiserror::Error;

use crate::Leb128DecodeErrorKind;

/// Error reported while decoding a LEB128 integer from a byte buffer.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[error("{kind}")]
pub struct Leb128DecodeError {
    kind: Leb128DecodeErrorKind,
    start_index: usize,
    error_index: usize,
    consumed: Option<NonZeroUsize>,
    required: Option<NonZeroUsize>,
    available: Option<usize>,
}

impl Leb128DecodeError {
    /// Creates an incomplete-input decoding error.
    ///
    /// # Parameters
    ///
    /// - `start_index`: Byte index where the incomplete value starts.
    /// - `required`: Non-zero lower bound for total bytes that must be
    ///   readable from `start_index` before decoding can make progress.
    /// - `available`: Bytes currently available from `start_index`.
    ///
    /// # Returns
    ///
    /// Returns an error carrying incomplete-input context.
    ///
    /// # Panics
    ///
    /// Panics when `required <= available`, or when the one-past-available
    /// error boundary overflows `usize`.
    pub const fn incomplete(start_index: usize, required: NonZeroUsize, available: usize) -> Self {
        assert!(
            required.get() > available,
            "incomplete LEB128 required bytes must exceed available bytes",
        );
        Self {
            kind: Leb128DecodeErrorKind::Incomplete,
            start_index,
            error_index: add_offset(start_index, available),
            consumed: None,
            required: Some(required),
            available: Some(available),
        }
    }

    /// Creates a malformed-input decoding error.
    ///
    /// # Parameters
    ///
    /// - `start_index`: Byte index where the malformed value starts.
    /// - `error_index`: Byte index at which the malformed input was detected.
    /// - `consumed`: Non-zero bytes the caller may consume to make progress.
    ///
    /// # Returns
    ///
    /// Returns an error carrying malformed-input context.
    ///
    /// # Panics
    ///
    /// Panics when `error_index` is outside the consumed span.
    pub const fn malformed(start_index: usize, error_index: usize, consumed: NonZeroUsize) -> Self {
        assert_error_index_in_consumed_span(start_index, error_index, consumed);
        Self {
            kind: Leb128DecodeErrorKind::Malformed,
            start_index,
            error_index,
            consumed: Some(consumed),
            required: None,
            available: None,
        }
    }

    /// Creates a non-canonical-input decoding error.
    ///
    /// # Parameters
    ///
    /// - `start_index`: Byte index where the non-canonical value starts.
    /// - `consumed`: Non-zero bytes the caller may consume to make progress.
    ///
    /// # Returns
    ///
    /// Returns an error carrying non-canonical-input context.
    ///
    /// # Panics
    ///
    /// Panics when the last consumed byte index overflows `usize`.
    pub const fn noncanonical(start_index: usize, consumed: NonZeroUsize) -> Self {
        Self {
            kind: Leb128DecodeErrorKind::NonCanonical,
            start_index,
            error_index: last_consumed_index(start_index, consumed),
            consumed: Some(consumed),
            required: None,
            available: None,
        }
    }

    /// Returns the decoding error kind.
    #[must_use]
    pub const fn kind(self) -> Leb128DecodeErrorKind {
        self.kind
    }

    /// Returns the absolute byte index where the attempted value starts.
    #[must_use]
    pub const fn start_index(self) -> usize {
        self.start_index
    }

    /// Returns the absolute byte index where the error was detected.
    ///
    /// # Returns
    ///
    /// Returns the byte index that made the error observable. For incomplete
    /// input this is the one-past-available boundary where the next byte would
    /// be required, so it may be equal to the input length visible to the
    /// caller.
    #[must_use]
    pub const fn error_index(self) -> usize {
        self.error_index
    }

    /// Returns bytes that may be consumed after an invalid-input error.
    ///
    /// # Returns
    ///
    /// Returns `Some(consumed)` for invalid input, or `None` for incomplete
    /// input.
    #[must_use]
    pub const fn consumed(self) -> Option<NonZeroUsize> {
        self.consumed
    }

    /// Returns whether this error reports an incomplete input prefix.
    #[must_use]
    pub const fn is_incomplete(self) -> bool {
        matches!(self.kind, Leb128DecodeErrorKind::Incomplete)
    }

    /// Returns whether this error reports malformed input.
    #[must_use]
    pub const fn is_malformed(self) -> bool {
        matches!(self.kind, Leb128DecodeErrorKind::Malformed)
    }

    /// Returns whether this error reports a non-canonical representation.
    #[must_use]
    pub const fn is_noncanonical(self) -> bool {
        matches!(self.kind, Leb128DecodeErrorKind::NonCanonical)
    }

    /// Returns a lower bound for bytes required to continue incomplete decoding.
    ///
    /// # Returns
    ///
    /// Returns `Some(required)` for incomplete input, or `None` otherwise. The
    /// returned value is a non-zero lower bound for the total readable bytes
    /// required from [`Self::start_index`] before decoding can make progress;
    /// it does not guarantee that the value will be complete at exactly that
    /// length.
    #[must_use]
    pub const fn required(self) -> Option<NonZeroUsize> {
        self.required
    }

    /// Returns additional bytes required to continue incomplete decoding.
    ///
    /// # Returns
    ///
    /// Returns `Some(additional)` for incomplete input, or `None` otherwise.
    /// The value is `required - available` and is guaranteed to be non-zero.
    #[must_use]
    pub const fn additional(self) -> Option<NonZeroUsize> {
        match (self.required, self.available) {
            (Some(required), Some(available)) => {
                let additional = required.get() - available;
                // SAFETY: `incomplete` enforces `required > available`.
                Some(unsafe { NonZeroUsize::new_unchecked(additional) })
            }
            _ => None,
        }
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

/// Adds an absolute byte offset to an index.
///
/// # Parameters
///
/// - `index`: Base byte index.
/// - `offset`: Byte offset to add.
///
/// # Returns
///
/// Returns `index + offset`.
///
/// # Panics
///
/// Panics when the sum overflows `usize`.
const fn add_offset(index: usize, offset: usize) -> usize {
    match index.checked_add(offset) {
        Some(value) => value,
        None => panic!("LEB128 byte index overflow"),
    }
}

/// Returns the absolute index of the last consumed byte.
///
/// # Parameters
///
/// - `start_index`: Byte index where the value starts.
/// - `consumed`: Non-zero number of consumed bytes.
///
/// # Returns
///
/// Returns `start_index + consumed - 1`.
///
/// # Panics
///
/// Panics when the index overflows `usize`.
const fn last_consumed_index(start_index: usize, consumed: NonZeroUsize) -> usize {
    add_offset(start_index, consumed.get() - 1)
}

/// Validates that an error index lies inside a consumed byte span.
///
/// # Parameters
///
/// - `start_index`: Byte index where the value starts.
/// - `error_index`: Byte index where the error was detected.
/// - `consumed`: Non-zero number of consumed bytes.
///
/// # Panics
///
/// Panics when `error_index` is before `start_index` or after the last consumed
/// byte.
const fn assert_error_index_in_consumed_span(
    start_index: usize,
    error_index: usize,
    consumed: NonZeroUsize,
) {
    let last_index = last_consumed_index(start_index, consumed);
    assert!(
        error_index >= start_index,
        "LEB128 error index must not precede value start",
    );
    assert!(
        error_index <= last_index,
        "LEB128 error index must lie inside consumed input",
    );
}
