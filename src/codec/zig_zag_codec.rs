// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use core::{
    convert::Infallible,
    marker::PhantomData,
};

use crate::{
    Leb128Codec,
    Leb128DecodeError,
    Leb128DecodePolicy,
    NonStrict,
    codec::leb128_codec::map_leb128_decode_failure,
    codec::leb128_codec::uleb_encoded_len,
};
use qubit_codec::Codec;

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
    marker: PhantomData<fn() -> (T, P)>,
}

macro_rules! impl_zig_zag_codec {
    ($signed:ty, $unsigned:ty, $shift:expr) => {
        impl<P> ZigZagCodec<$signed, P>
        where
            P: Leb128DecodePolicy,
        {
            /// Minimum number of bytes that can represent a complete value.
            pub const MIN_UNITS_PER_VALUE: usize = <Self as Codec>::MIN_UNITS_PER_VALUE;

            /// Maximum number of bytes emitted when encoding this type.
            pub const MAX_ENCODE_UNITS_PER_VALUE: usize =
                <Self as Codec>::MAX_ENCODE_UNITS_PER_VALUE;

            /// Maximum number of bytes consumed when decoding this type.
            pub const MAX_DECODE_UNITS_PER_VALUE: usize =
                <Self as Codec>::MAX_DECODE_UNITS_PER_VALUE;

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
            #[inline(always)]
            pub unsafe fn decode(
                input: &[u8],
                input_index: usize,
            ) -> Result<($signed, core::num::NonZeroUsize), Leb128DecodeError> {
                debug_assert!(input.len().saturating_sub(input_index) >= Self::MIN_UNITS_PER_VALUE);

                // SAFETY: The caller guarantees enough readable bytes for this
                // type.
                let (encoded, consumed) =
                    unsafe { Leb128Codec::<$unsigned, P>::decode(input, input_index)? };
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
            #[inline(always)]
            pub unsafe fn encode(value: $signed, output: &mut [u8], output_index: usize) -> usize {
                let encoded = ((value as $unsigned) << 1) ^ ((value >> $shift) as $unsigned);
                // SAFETY: The caller guarantees enough writable bytes for the
                // canonical representation of this value.
                unsafe {
                    Leb128Codec::<$unsigned, NonStrict>::encode(encoded, output, output_index)
                }
            }
        }

        impl<P> Codec for ZigZagCodec<$signed, P>
        where
            P: Leb128DecodePolicy,
        {
            type Value = $signed;
            type Unit = u8;
            type DecodeError = Leb128DecodeError;
            type EncodeError = Infallible;

            const MIN_UNITS_PER_VALUE: usize = 1;
            const MAX_ENCODE_UNITS_PER_VALUE: usize =
                Leb128Codec::<$unsigned, NonStrict>::MAX_ENCODE_UNITS_PER_VALUE;
            const MAX_DECODE_UNITS_PER_VALUE: usize =
                Leb128Codec::<$unsigned, NonStrict>::MAX_DECODE_UNITS_PER_VALUE;

            #[inline(always)]
            fn encode_len(&self, value: &$signed) -> usize {
                let encoded = ((*value as $unsigned) << 1) ^ ((*value >> $shift) as $unsigned);
                uleb_encoded_len(encoded as u128)
            }

            #[inline(always)]
            unsafe fn decode(
                &mut self,
                input: &[u8],
                input_index: usize,
            ) -> Result<
                ($signed, core::num::NonZeroUsize),
                qubit_codec::DecodeFailure<Self::DecodeError>,
            > {
                debug_assert!(input.len().saturating_sub(input_index) >= Self::MIN_UNITS_PER_VALUE);

                // SAFETY: The caller upholds the `Codec::decode`
                // contract.
                unsafe { Self::decode(input, input_index) }.map_err(map_leb128_decode_failure)
            }

            #[inline(always)]
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
