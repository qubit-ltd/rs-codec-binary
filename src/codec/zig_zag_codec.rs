// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use core::convert::Infallible;
use core::marker::PhantomData;

use qubit_codec::Codec;
use qubit_codec::DecodeFailure;

use crate::Leb128Codec;
use crate::Leb128DecodeError;
use crate::Leb128DecodePolicy;
use crate::NonStrict;
use crate::codec::leb128_codec::map_leb128_decode_failure;
use crate::codec::leb128_codec::uleb_encoded_len;

/// Type-level unchecked ZigZag + unsigned LEB128 codec.
///
/// Encoding is always canonical; `P` only affects decoding. Encoding-only
/// callers should conventionally use [`NonStrict`] because no decoding policy
/// is applied on that path.
///
/// # Type Parameters
///
/// - `T`: Signed integer value type to decode from ZigZag-encoded LEB128 bytes
///   and encode into ZigZag-encoded LEB128 bytes.
/// - `P`: Required type-level decoding policy implementing
///   [`Leb128DecodePolicy`] for the underlying unsigned LEB128 payload. Use
///   [`crate::Strict`] to reject non-canonical inputs, or [`NonStrict`] to
///   accept non-canonical inputs. This parameter does not affect canonical
///   encoding.
///
/// The decoding policy is intentionally required so wire-format callers make
/// the canonicality contract explicit.
/// The underlying unsigned LEB128 payload remains subject to its target
/// integer's maximum decode width and terminal payload bits, including under
/// [`NonStrict`]. See that policy's documentation for the width limits.
///
/// ```compile_fail
/// use qubit_codec_binary::ZigZagCodec;
///
/// let _ = ZigZagCodec::<i64>::default();
/// ```
///
/// # Examples
///
/// ```
/// use qubit_codec_binary::{
///     NonStrict,
///     ZigZagCodec,
/// };
///
/// let mut output = [0_u8; ZigZagCodec::<i64, NonStrict>::MAX_ENCODE_UNITS_PER_VALUE];
/// let written = unsafe {
///     ZigZagCodec::<i64, NonStrict>::encode(-42, &mut output, 0)
/// };
/// assert_eq!(1, written);
///
/// let (decoded, consumed) = unsafe {
///     ZigZagCodec::<i64, NonStrict>::decode(&output[..written], 0)
/// }.expect("canonical ZigZag LEB128 value should decode");
/// assert_eq!(-42, decoded);
/// assert_eq!(1, consumed.get());
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ZigZagCodec<T, P> {
    /// Associates the value and decode-policy types without storing either.
    marker: PhantomData<fn() -> (T, P)>,
}

/// Implements the ZigZag codec for one signed and unsigned integer pair.
///
/// The expansion defines the unchecked helpers and the [`Codec`] implementation
/// while keeping their arithmetic tied to the integer type's bit width.
macro_rules! impl_zig_zag_codec {
    ($signed:ty, $unsigned:ty, $shift:expr) => {
        impl<P> ZigZagCodec<$signed, P>
        where
            P: Leb128DecodePolicy,
        {
            /// Minimum number of bytes that can represent a complete value.
            pub const MIN_UNITS_PER_VALUE: usize = <Self as Codec>::MIN_UNITS_PER_VALUE;

            /// Maximum number of bytes emitted when encoding this type.
            pub const MAX_ENCODE_UNITS_PER_VALUE: usize = <Self as Codec>::MAX_ENCODE_UNITS_PER_VALUE;

            /// Maximum number of bytes consumed when decoding this type.
            pub const MAX_DECODE_UNITS_PER_VALUE: usize = <Self as Codec>::MAX_DECODE_UNITS_PER_VALUE;

            /// Decodes a value from `input` starting at `input_index` without
            /// bounds checks.
            ///
            /// # Parameters
            ///
            /// - `input`: Source byte buffer.
            /// - `input_index`: Start index in `input`.
            ///
            /// # Returns
            ///
            /// Returns the decoded value and the non-zero number of consumed
            /// bytes.
            ///
            /// # Errors
            ///
            /// Returns [`Leb128DecodeError`] if the underlying LEB128 bytes are
            /// incomplete, malformed, or non-canonical under strict policy.
            ///
            /// # Safety
            ///
            /// The caller must guarantee that `input_index` is a valid boundary
            /// and at least [`Self::MIN_UNITS_PER_VALUE`] byte is readable
            /// from `input_index`.
            #[inline]
            pub unsafe fn decode(
                input: &[u8],
                input_index: usize,
            ) -> Result<($signed, core::num::NonZeroUsize), Leb128DecodeError> {
                debug_assert!(input.len().saturating_sub(input_index) >= Self::MIN_UNITS_PER_VALUE);

                // SAFETY: The caller guarantees enough readable bytes for this
                // type.
                let (encoded, consumed) = unsafe { Leb128Codec::<$unsigned, P>::decode(input, input_index)? };
                let value = ((encoded >> 1) as $signed) ^ (-((encoded & 1) as $signed));
                Ok((value, consumed))
            }

            /// Encodes `value` into `output` starting at `output_index` without
            /// bounds checks.
            ///
            /// # Parameters
            ///
            /// - `value`: Value to encode.
            /// - `output`: Destination byte buffer.
            /// - `output_index`: Start index in `output`.
            ///
            /// # Returns
            ///
            /// Returns the number of written bytes.
            ///
            /// # Safety
            ///
            /// The caller must guarantee that the canonical ZigZag LEB128 byte
            /// width of `value` is writable starting at `output_index`.
            /// Reserving [`Self::MAX_ENCODE_UNITS_PER_VALUE`] bytes always satisfies
            /// this requirement; for a known value, the exact
            /// [`Codec::encode_len`] is sufficient.
            #[must_use = "the returned byte count determines the encoded payload range"]
            #[inline]
            pub unsafe fn encode(value: $signed, output: &mut [u8], output_index: usize) -> usize {
                let encoded = ((value as $unsigned) << 1) ^ ((value >> $shift) as $unsigned);
                // SAFETY: The caller guarantees enough writable bytes for the
                // canonical representation of this value.
                unsafe { Leb128Codec::<$unsigned, NonStrict>::encode(encoded, output, output_index) }
            }
        }

        impl<P> Codec for ZigZagCodec<$signed, P>
        where
            P: Leb128DecodePolicy,
        {
            /// The signed integer represented by this codec.
            type Value = $signed;
            /// One byte of the ZigZag LEB128 representation.
            type Unit = u8;
            /// Detailed errors for incomplete, malformed, or policy-rejected input.
            type DecodeError = Leb128DecodeError;
            /// Encoding is infallible for every value of this signed integer type.
            type EncodeError = Infallible;

            /// Minimum bytes readable before decoding can be attempted.
            const MIN_UNITS_PER_VALUE: usize = 1;
            /// Maximum bytes emitted by canonical ZigZag plus unsigned LEB128 encoding.
            const MAX_ENCODE_UNITS_PER_VALUE: usize = Leb128Codec::<$unsigned, NonStrict>::MAX_ENCODE_UNITS_PER_VALUE;
            /// Maximum bytes consumed by decoding this signed integer type.
            const MAX_DECODE_UNITS_PER_VALUE: usize = Leb128Codec::<$unsigned, NonStrict>::MAX_DECODE_UNITS_PER_VALUE;

            /// Returns the exact width of the canonical unsigned LEB128 encoding
            /// of `value` after ZigZag mapping.
            ///
            /// # Parameters
            ///
            /// - `value`: Signed integer value whose encoded width is queried.
            ///
            /// # Returns
            ///
            /// Returns a width of at least one byte and no more than
            /// [`Self::MAX_ENCODE_UNITS_PER_VALUE`]. The decoding policy does not
            /// affect this length.
            #[inline]
            fn encode_len(&self, value: &$signed) -> usize {
                let encoded = ((*value as $unsigned) << 1) ^ ((*value >> $shift) as $unsigned);
                uleb_encoded_len(encoded as u128)
            }

            /// Decodes one ZigZag-mapped unsigned LEB128 value and reports its
            /// byte consumption.
            ///
            /// # Parameters
            ///
            /// - `input`: Source bytes.
            /// - `input_index`: Start index of the encoded value.
            ///
            /// # Returns
            ///
            /// Returns the signed value and the non-zero number of bytes consumed.
            ///
            /// # Errors
            ///
            /// Returns [`DecodeFailure::Incomplete`] with the detailed error
            /// retained as its source and a minimum total byte requirement when
            /// a valid prefix needs more input. Returns [`DecodeFailure::Invalid`]
            /// with the detailed error and consumed-byte count for malformed
            /// input or, under [`crate::Strict`], a non-canonical underlying
            /// unsigned LEB128 representation. The required count is a lower
            /// bound, not a guarantee that decoding will succeed at that length.
            /// The consumed count is not a guaranteed protocol-field recovery
            /// boundary. Non-strict decoding still enforces the target integer's
            /// maximum width and terminal payload-bit limits.
            ///
            /// # Safety
            ///
            /// `input_index` must be a valid boundary and at least
            /// [`Self::MIN_UNITS_PER_VALUE`] bytes must be readable there.
            #[inline]
            unsafe fn decode(
                &mut self,
                input: &[u8],
                input_index: usize,
            ) -> Result<($signed, core::num::NonZeroUsize), DecodeFailure<Self::DecodeError>> {
                debug_assert!(input.len().saturating_sub(input_index) >= Self::MIN_UNITS_PER_VALUE);

                // SAFETY: The caller upholds the `Codec::decode`
                // contract.
                unsafe { Self::decode(input, input_index) }.map_err(map_leb128_decode_failure)
            }

            /// Encodes one value using canonical ZigZag mapping followed by
            /// unsigned LEB128 encoding.
            ///
            /// # Parameters
            ///
            /// - `value`: Signed integer value to encode.
            /// - `output`: Destination byte buffer.
            /// - `output_index`: Start index in `output`.
            ///
            /// # Returns
            ///
            /// Returns the number of bytes written, equal to
            /// [`Self::encode_len`] for `value`.
            ///
            /// # Errors
            ///
            /// This implementation uses [`Infallible`]; every value of the
            /// signed integer type can be encoded.
            ///
            /// # Safety
            ///
            /// The caller must ensure [`Codec::can_encode_value`] returned
            /// `true` for `value`. Starting at `output_index`, provide either
            /// the exact [`Self::encode_len`] capacity for `value` or at least
            /// [`Self::MAX_ENCODE_UNITS_PER_VALUE`] writable bytes.
            #[inline]
            unsafe fn encode(
                &mut self,
                value: &$signed,
                output: &mut [u8],
                output_index: usize,
            ) -> Result<usize, Self::EncodeError> {
                let required = self.encode_len(value);
                debug_assert!(output.len().saturating_sub(output_index) >= required);

                // SAFETY: The `Codec::encode` contract provides either the
                // exact canonical width or this type's maximum width.
                let written = unsafe { Self::encode(*value, output, output_index) };
                Ok(written)
            }
        }
    };
}

impl_zig_zag_codec!(i8, u8, 7);
impl_zig_zag_codec!(i16, u16, 15);
impl_zig_zag_codec!(i32, u32, 31);
impl_zig_zag_codec!(i64, u64, 63);
impl_zig_zag_codec!(i128, u128, 127);
impl_zig_zag_codec!(isize, usize, isize::BITS - 1);
