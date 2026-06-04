/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/

use core::{convert::Infallible, marker::PhantomData};

use qubit_codec::Codec;

use crate::{Leb128Codec, Leb128DecodeError, Leb128DecodePolicy, NonStrict};

/// Type-level unchecked ZigZag + unsigned LEB128 codec.
///
/// # Type Parameters
///
/// - `T`: Signed integer value type to decode from ZigZag-encoded LEB128 bytes
///   and encode into ZigZag-encoded LEB128 bytes.
/// - `P`: Type-level decoding policy implementing [`Leb128DecodePolicy`] for the
///   underlying unsigned LEB128 payload. Use [`crate::Strict`] to reject
///   non-canonical inputs, or [`NonStrict`] to accept non-canonical inputs.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ZigZagCodec<T, P = NonStrict> {
    marker: PhantomData<fn() -> (T, P)>,
}

macro_rules! impl_zig_zag_codec {
    ($signed:ty, $unsigned:ty, $shift:expr) => {
        impl<P> ZigZagCodec<$signed, P>
        where
            P: Leb128DecodePolicy,
        {
            /// Minimum number of bytes that can represent a complete value.
            pub const MIN_UNITS_PER_VALUE: usize = 1;

            /// Maximum number of bytes required to encode or decode this type.
            pub const MAX_UNITS_PER_VALUE: usize =
                Leb128Codec::<$unsigned, NonStrict>::MAX_UNITS_PER_VALUE;

            /// Decodes a value from `input` starting at `index` without bounds checks.
            ///
            /// # Parameters
            ///
            /// - `input`: Source byte buffer.
            /// - `index`: Start index in `input`.
            ///
            /// # Returns
            ///
            /// Returns the decoded value and the non-zero number of consumed bytes.
            ///
            /// # Errors
            ///
            /// Returns [`Leb128DecodeError`] if the underlying LEB128 bytes are
            /// incomplete, malformed, or non-canonical under strict policy.
            ///
            /// # Safety
            ///
            /// The caller must guarantee that `index` is a valid boundary and
            /// at least [`Self::MIN_UNITS_PER_VALUE`] byte is readable from
            /// `index`.
            #[inline(always)]
            pub unsafe fn decode_unchecked(
                input: &[u8],
                index: usize,
            ) -> Result<($signed, core::num::NonZeroUsize), Leb128DecodeError> {
                debug_assert!(input.len().saturating_sub(index) >= Self::MIN_UNITS_PER_VALUE);

                // SAFETY: The caller guarantees enough readable bytes for this type.
                let (encoded, consumed) =
                    unsafe { Leb128Codec::<$unsigned, P>::decode_unchecked(input, index)? };
                let value = ((encoded >> 1) as $signed) ^ (-((encoded & 1) as $signed));
                Ok((value, consumed))
            }

            /// Encodes `value` into `output` starting at `index` without bounds checks.
            ///
            /// # Parameters
            ///
            /// - `value`: Value to encode.
            /// - `output`: Destination byte buffer.
            /// - `index`: Start index in `output`.
            ///
            /// # Returns
            ///
            /// Returns the number of written bytes.
            ///
            /// # Safety
            ///
            /// The caller must guarantee that `output.as_mut_ptr().add(index)` is valid
            /// to write [`Self::MAX_UNITS_PER_VALUE`] bytes.
            #[inline(always)]
            pub unsafe fn encode_unchecked(
                value: $signed,
                output: &mut [u8],
                index: usize,
            ) -> usize {
                let encoded = ((value as $unsigned) << 1) ^ ((value >> $shift) as $unsigned);
                // SAFETY: The caller guarantees enough writable bytes for this type.
                unsafe {
                    Leb128Codec::<$unsigned, NonStrict>::encode_unchecked(encoded, output, index)
                }
            }
        }

        unsafe impl<P> Codec for ZigZagCodec<$signed, P>
        where
            P: Leb128DecodePolicy,
        {
            type Value = $signed;
            type Unit = u8;
            type DecodeError = Leb128DecodeError;
            type EncodeError = Infallible;

            #[inline(always)]
            fn min_units_per_value(&self) -> core::num::NonZeroUsize {
                core::num::NonZeroUsize::MIN
            }

            #[inline(always)]
            fn max_units_per_value(&self) -> core::num::NonZeroUsize {
                // SAFETY: ZigZag LEB128 has a non-zero maximum encoded width.
                unsafe { core::num::NonZeroUsize::new_unchecked(Self::MAX_UNITS_PER_VALUE) }
            }

            #[inline(always)]
            unsafe fn decode_unchecked(
                &self,
                input: &[u8],
                index: usize,
            ) -> Result<($signed, core::num::NonZeroUsize), Self::DecodeError> {
                debug_assert!(input.len().saturating_sub(index) >= Self::MIN_UNITS_PER_VALUE);

                // SAFETY: The caller upholds the `Codec::decode_unchecked` contract.
                unsafe { Self::decode_unchecked(input, index) }
            }

            #[inline(always)]
            unsafe fn encode_unchecked(
                &self,
                value: &$signed,
                output: &mut [u8],
                index: usize,
            ) -> Result<usize, Self::EncodeError> {
                debug_assert!(output.len().saturating_sub(index) >= Self::MAX_UNITS_PER_VALUE);

                // SAFETY: The caller upholds the `Codec::encode_unchecked` contract.
                Ok(unsafe { Self::encode_unchecked(*value, output, index) })
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
