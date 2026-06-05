/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/

use core::{
    convert::Infallible,
    marker::PhantomData,
    ptr,
};

use qubit_codec::Codec;

use crate::{
    BigEndian,
    LittleEndian,
};

/// Type-level unchecked binary codec for one scalar type and one byte order.
///
/// `BinaryCodec` is intentionally a zero-sized codec type. It exposes type-level
/// unchecked helpers for direct hot-path use and also implements [`Codec`] for
/// generic codec pipelines. Callers must validate buffer lengths before entering
/// the hot path.
///
/// # Type Parameters
///
/// - `T`: Scalar value type to decode from bytes and encode into bytes.
/// - `O`: Type-level byte order marker. Multi-byte scalar implementations use
///   [`BigEndian`] or [`LittleEndian`]. Single-byte scalar implementations
///   accept any marker because byte order does not affect one-byte values.
///
/// # Examples
///
/// ```
/// use qubit_codec_binary::{
///     BigEndian,
///     BinaryCodec,
/// };
///
/// let mut output = [0_u8; BinaryCodec::<u32, BigEndian>::MAX_UNITS_PER_VALUE];
/// let written = unsafe {
///     BinaryCodec::<u32, BigEndian>::encode_unchecked(0x0102_0304, &mut output, 0)
/// };
/// assert_eq!(4, written);
/// assert_eq!([1, 2, 3, 4], output);
///
/// let (decoded, consumed) = unsafe {
///     BinaryCodec::<u32, BigEndian>::decode_unchecked(&output, 0)
/// };
/// assert_eq!(0x0102_0304, decoded);
/// assert_eq!(4, consumed.get());
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct BinaryCodec<T, O> {
    marker: PhantomData<fn() -> (T, O)>,
}

impl<O> BinaryCodec<u8, O> {
    /// Minimum number of bytes required to encode or decode this type.
    pub const MIN_UNITS_PER_VALUE: usize = 1;

    /// Maximum number of bytes required to encode or decode this type.
    pub const MAX_UNITS_PER_VALUE: usize = Self::MIN_UNITS_PER_VALUE;

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
    /// # Safety
    ///
    /// The caller must guarantee that `input.as_ptr().add(index)` is valid to
    /// read [`Self::MIN_UNITS_PER_VALUE`] bytes.
    #[must_use]
    #[inline(always)]
    pub unsafe fn decode_unchecked(input: &[u8], index: usize) -> (u8, core::num::NonZeroUsize) {
        debug_assert!(index + Self::MIN_UNITS_PER_VALUE <= input.len());

        // SAFETY: The caller guarantees that the indexed byte is readable.
        (
            unsafe { *input.as_ptr().add(index) },
            // SAFETY: `u8` is one byte wide.
            unsafe { core::num::NonZeroUsize::new_unchecked(Self::MIN_UNITS_PER_VALUE) },
        )
    }

    /// Encodes `value` into `output` starting at `index` without bounds checks.
    ///
    /// # Parameters
    ///
    /// - `value`: Value to encode.
    /// - `output`: Destination byte buffer.
    /// - `index`: Start index in `output`.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that `output.as_mut_ptr().add(index)` is valid
    /// to write [`Self::MAX_UNITS_PER_VALUE`] bytes.
    #[inline(always)]
    pub unsafe fn encode_unchecked(value: u8, output: &mut [u8], index: usize) -> usize {
        debug_assert!(index + Self::MAX_UNITS_PER_VALUE <= output.len());

        // SAFETY: The caller guarantees that the indexed byte is writable.
        unsafe {
            *output.as_mut_ptr().add(index) = value;
        }
        Self::MAX_UNITS_PER_VALUE
    }
}

unsafe impl<O> Codec for BinaryCodec<u8, O> {
    type Value = u8;
    type Unit = u8;
    type DecodeError = Infallible;
    type EncodeError = Infallible;

    #[inline(always)]
    fn min_units_per_value(&self) -> core::num::NonZeroUsize {
        // SAFETY: `u8` is one byte wide.
        unsafe { core::num::NonZeroUsize::new_unchecked(Self::MIN_UNITS_PER_VALUE) }
    }

    #[inline(always)]
    fn max_units_per_value(&self) -> core::num::NonZeroUsize {
        // SAFETY: `u8` is one byte wide.
        unsafe { core::num::NonZeroUsize::new_unchecked(Self::MAX_UNITS_PER_VALUE) }
    }

    #[inline(always)]
    unsafe fn decode_unchecked(
        &self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), Self::DecodeError> {
        // SAFETY: The caller upholds the `Codec::decode_unchecked` contract.
        Ok(unsafe { Self::decode_unchecked(input, index) })
    }

    #[inline(always)]
    unsafe fn encode_unchecked(&self, value: &u8, output: &mut [u8], index: usize) -> Result<usize, Self::EncodeError> {
        // SAFETY: The caller upholds the `Codec::encode_unchecked` contract.
        Ok(unsafe { Self::encode_unchecked(*value, output, index) })
    }
}

impl<O> BinaryCodec<i8, O> {
    /// Minimum number of bytes required to encode or decode this type.
    pub const MIN_UNITS_PER_VALUE: usize = 1;

    /// Maximum number of bytes required to encode or decode this type.
    pub const MAX_UNITS_PER_VALUE: usize = Self::MIN_UNITS_PER_VALUE;

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
    /// # Safety
    ///
    /// The caller must guarantee that `input.as_ptr().add(index)` is valid to
    /// read [`Self::MIN_UNITS_PER_VALUE`] bytes.
    #[must_use]
    #[inline(always)]
    pub unsafe fn decode_unchecked(input: &[u8], index: usize) -> (i8, core::num::NonZeroUsize) {
        debug_assert!(index + Self::MIN_UNITS_PER_VALUE <= input.len());

        // SAFETY: The caller guarantees that the indexed byte is readable.
        (
            unsafe { *input.as_ptr().add(index) as i8 },
            // SAFETY: `i8` is one byte wide.
            unsafe { core::num::NonZeroUsize::new_unchecked(Self::MIN_UNITS_PER_VALUE) },
        )
    }

    /// Encodes `value` into `output` starting at `index` without bounds checks.
    ///
    /// # Parameters
    ///
    /// - `value`: Value to encode.
    /// - `output`: Destination byte buffer.
    /// - `index`: Start index in `output`.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that `output.as_mut_ptr().add(index)` is valid
    /// to write [`Self::MAX_UNITS_PER_VALUE`] bytes.
    #[inline(always)]
    pub unsafe fn encode_unchecked(value: i8, output: &mut [u8], index: usize) -> usize {
        debug_assert!(index + Self::MAX_UNITS_PER_VALUE <= output.len());

        // SAFETY: The caller guarantees that the indexed byte is writable.
        unsafe {
            *output.as_mut_ptr().add(index) = value as u8;
        }
        Self::MAX_UNITS_PER_VALUE
    }
}

unsafe impl<O> Codec for BinaryCodec<i8, O> {
    type Value = i8;
    type Unit = u8;
    type DecodeError = Infallible;
    type EncodeError = Infallible;

    #[inline(always)]
    fn min_units_per_value(&self) -> core::num::NonZeroUsize {
        // SAFETY: `i8` is one byte wide.
        unsafe { core::num::NonZeroUsize::new_unchecked(Self::MIN_UNITS_PER_VALUE) }
    }

    #[inline(always)]
    fn max_units_per_value(&self) -> core::num::NonZeroUsize {
        // SAFETY: `i8` is one byte wide.
        unsafe { core::num::NonZeroUsize::new_unchecked(Self::MAX_UNITS_PER_VALUE) }
    }

    #[inline(always)]
    unsafe fn decode_unchecked(
        &self,
        input: &[u8],
        index: usize,
    ) -> Result<(i8, core::num::NonZeroUsize), Self::DecodeError> {
        // SAFETY: The caller upholds the `Codec::decode_unchecked` contract.
        Ok(unsafe { Self::decode_unchecked(input, index) })
    }

    #[inline(always)]
    unsafe fn encode_unchecked(&self, value: &i8, output: &mut [u8], index: usize) -> Result<usize, Self::EncodeError> {
        // SAFETY: The caller upholds the `Codec::encode_unchecked` contract.
        Ok(unsafe { Self::encode_unchecked(*value, output, index) })
    }
}

macro_rules! impl_integer_binary_codec {
    ($ty:ty, $len:expr) => {
        impl BinaryCodec<$ty, BigEndian> {
            /// Minimum number of bytes required to encode or decode this type.
            pub const MIN_UNITS_PER_VALUE: usize = $len;

            /// Maximum number of bytes required to encode or decode this type.
            pub const MAX_UNITS_PER_VALUE: usize = Self::MIN_UNITS_PER_VALUE;

            /// Decodes a value from `input` starting at `index` without bounds checks.
            ///
            /// This function is intended for hot binary codec paths where the
            /// caller has already validated the buffer length externally.
            ///
            /// # Parameters
            ///
            /// - `input`: Source byte buffer.
            /// - `index`: Start byte index in `input`.
            ///
            /// # Returns
            ///
            /// Returns the decoded value and the non-zero number of consumed bytes.
            ///
            /// # Safety
            ///
            /// The caller must guarantee that:
            ///
            /// - `index + Self::MIN_UNITS_PER_VALUE <= input.len()`
            /// - `input[index..index + Self::MIN_UNITS_PER_VALUE]`
            ///   is valid for reading.
            #[must_use]
            #[inline(always)]
            pub unsafe fn decode_unchecked(input: &[u8], index: usize) -> ($ty, core::num::NonZeroUsize) {
                debug_assert!(index + Self::MIN_UNITS_PER_VALUE <= input.len());

                // SAFETY:
                // The caller guarantees that the readable range is fully in-bounds.
                // `read_unaligned` permits unaligned memory access.
                let pointer = unsafe { input.as_ptr().add(index).cast::<$ty>() };

                // SAFETY:
                // The pointer is valid for an unaligned integer load.
                let raw = unsafe { ptr::read_unaligned(pointer) };

                (
                    <$ty>::from_be(raw),
                    // SAFETY: `$ty` is a concrete primitive integer type with non-zero size.
                    unsafe { core::num::NonZeroUsize::new_unchecked(Self::MIN_UNITS_PER_VALUE) },
                )
            }

            /// Encodes `value` into `output` starting at `index`
            /// without bounds checks.
            ///
            /// This function is intended for hot binary codec paths where the
            /// caller has already validated the buffer length externally.
            ///
            /// # Parameters
            ///
            /// - `value`: Value to encode.
            /// - `output`: Destination byte buffer.
            /// - `index`: Start byte index in `output`.
            ///
            /// # Safety
            ///
            /// The caller must guarantee that:
            ///
            /// - `index + Self::MAX_UNITS_PER_VALUE <= output.len()`
            /// - `output[index..index + Self::MAX_UNITS_PER_VALUE]`
            ///   is valid for writing.
            #[inline(always)]
            pub unsafe fn encode_unchecked(value: $ty, output: &mut [u8], index: usize) -> usize {
                debug_assert!(index + Self::MAX_UNITS_PER_VALUE <= output.len());

                let raw = value.to_be();

                // SAFETY:
                // The caller guarantees that the writable range is fully in-bounds.
                // `write_unaligned` permits unaligned memory access.
                let pointer = unsafe { output.as_mut_ptr().add(index).cast::<$ty>() };

                // SAFETY:
                // The pointer is valid for an unaligned integer store.
                unsafe {
                    ptr::write_unaligned(pointer, raw);
                }
                Self::MAX_UNITS_PER_VALUE
            }
        }

        unsafe impl Codec for BinaryCodec<$ty, BigEndian> {
            type Value = $ty;
            type Unit = u8;
            type DecodeError = Infallible;
            type EncodeError = Infallible;

            #[inline(always)]
            fn min_units_per_value(&self) -> core::num::NonZeroUsize {
                // SAFETY: `$ty` is a concrete primitive integer type with non-zero size.
                unsafe { core::num::NonZeroUsize::new_unchecked(Self::MIN_UNITS_PER_VALUE) }
            }

            #[inline(always)]
            fn max_units_per_value(&self) -> core::num::NonZeroUsize {
                // SAFETY: `$ty` is a concrete primitive integer type with non-zero size.
                unsafe { core::num::NonZeroUsize::new_unchecked(Self::MAX_UNITS_PER_VALUE) }
            }

            #[inline(always)]
            unsafe fn decode_unchecked(
                &self,
                input: &[u8],
                index: usize,
            ) -> Result<($ty, core::num::NonZeroUsize), Self::DecodeError> {
                // SAFETY: The caller upholds the `Codec::decode_unchecked` contract.
                Ok(unsafe { Self::decode_unchecked(input, index) })
            }

            #[inline(always)]
            unsafe fn encode_unchecked(
                &self,
                value: &$ty,
                output: &mut [u8],
                index: usize,
            ) -> Result<usize, Self::EncodeError> {
                // SAFETY: The caller upholds the `Codec::encode_unchecked` contract.
                Ok(unsafe { Self::encode_unchecked(*value, output, index) })
            }
        }

        impl BinaryCodec<$ty, LittleEndian> {
            /// Minimum number of bytes required to encode or decode this type.
            pub const MIN_UNITS_PER_VALUE: usize = $len;

            /// Maximum number of bytes required to encode or decode this type.
            pub const MAX_UNITS_PER_VALUE: usize = Self::MIN_UNITS_PER_VALUE;

            /// Decodes a value from `input` starting at `index` without bounds checks.
            ///
            /// This function is intended for hot binary codec paths where the
            /// caller has already validated the buffer length externally.
            ///
            /// # Parameters
            ///
            /// - `input`: Source byte buffer.
            /// - `index`: Start byte index in `input`.
            ///
            /// # Returns
            ///
            /// Returns the decoded value and the non-zero number of consumed bytes.
            ///
            /// # Safety
            ///
            /// The caller must guarantee that:
            ///
            /// - `index + Self::MIN_UNITS_PER_VALUE <= input.len()`
            /// - `input[index..index + Self::MIN_UNITS_PER_VALUE]`
            ///   is valid for reading.
            #[must_use]
            #[inline(always)]
            pub unsafe fn decode_unchecked(input: &[u8], index: usize) -> ($ty, core::num::NonZeroUsize) {
                debug_assert!(index + Self::MIN_UNITS_PER_VALUE <= input.len());

                // SAFETY:
                // The caller guarantees that the readable range is fully in-bounds.
                // `read_unaligned` permits unaligned memory access.
                let pointer = unsafe { input.as_ptr().add(index).cast::<$ty>() };

                // SAFETY:
                // The pointer is valid for an unaligned integer load.
                let raw = unsafe { ptr::read_unaligned(pointer) };

                (
                    <$ty>::from_le(raw),
                    // SAFETY: `$ty` is a concrete primitive integer type with non-zero size.
                    unsafe { core::num::NonZeroUsize::new_unchecked(Self::MIN_UNITS_PER_VALUE) },
                )
            }

            /// Encodes `value` into `output` starting at `index`
            /// without bounds checks.
            ///
            /// This function is intended for hot binary codec paths where the
            /// caller has already validated the buffer length externally.
            ///
            /// # Parameters
            ///
            /// - `value`: Value to encode.
            /// - `output`: Destination byte buffer.
            /// - `index`: Start byte index in `output`.
            ///
            /// # Safety
            ///
            /// The caller must guarantee that:
            ///
            /// - `index + Self::MAX_UNITS_PER_VALUE <= output.len()`
            /// - `output[index..index + Self::MAX_UNITS_PER_VALUE]`
            ///   is valid for writing.
            #[inline(always)]
            pub unsafe fn encode_unchecked(value: $ty, output: &mut [u8], index: usize) -> usize {
                debug_assert!(index + Self::MAX_UNITS_PER_VALUE <= output.len());

                let raw = value.to_le();

                // SAFETY:
                // The caller guarantees that the writable range is fully in-bounds.
                // `write_unaligned` permits unaligned memory access.
                let pointer = unsafe { output.as_mut_ptr().add(index).cast::<$ty>() };

                // SAFETY:
                // The pointer is valid for an unaligned integer store.
                unsafe {
                    ptr::write_unaligned(pointer, raw);
                }
                Self::MAX_UNITS_PER_VALUE
            }
        }

        unsafe impl Codec for BinaryCodec<$ty, LittleEndian> {
            type Value = $ty;
            type Unit = u8;
            type DecodeError = Infallible;
            type EncodeError = Infallible;

            #[inline(always)]
            fn min_units_per_value(&self) -> core::num::NonZeroUsize {
                // SAFETY: `$ty` is a concrete primitive integer type with non-zero size.
                unsafe { core::num::NonZeroUsize::new_unchecked(Self::MIN_UNITS_PER_VALUE) }
            }

            #[inline(always)]
            fn max_units_per_value(&self) -> core::num::NonZeroUsize {
                // SAFETY: `$ty` is a concrete primitive integer type with non-zero size.
                unsafe { core::num::NonZeroUsize::new_unchecked(Self::MAX_UNITS_PER_VALUE) }
            }

            #[inline(always)]
            unsafe fn decode_unchecked(
                &self,
                input: &[u8],
                index: usize,
            ) -> Result<($ty, core::num::NonZeroUsize), Self::DecodeError> {
                // SAFETY: The caller upholds the `Codec::decode_unchecked` contract.
                Ok(unsafe { Self::decode_unchecked(input, index) })
            }

            #[inline(always)]
            unsafe fn encode_unchecked(
                &self,
                value: &$ty,
                output: &mut [u8],
                index: usize,
            ) -> Result<usize, Self::EncodeError> {
                // SAFETY: The caller upholds the `Codec::encode_unchecked` contract.
                Ok(unsafe { Self::encode_unchecked(*value, output, index) })
            }
        }
    };
}

macro_rules! impl_float_binary_codec {
    ($ty:ty, $bits:ty, $len:expr) => {
        impl BinaryCodec<$ty, BigEndian> {
            /// Minimum number of bytes required to encode or decode this type.
            pub const MIN_UNITS_PER_VALUE: usize = $len;

            /// Maximum number of bytes required to encode or decode this type.
            pub const MAX_UNITS_PER_VALUE: usize = Self::MIN_UNITS_PER_VALUE;

            /// Decodes a value from `input` starting at `index` without bounds checks.
            ///
            /// This function is intended for hot binary codec paths where the
            /// caller has already validated the buffer length externally.
            ///
            /// # Parameters
            ///
            /// - `input`: Source byte buffer.
            /// - `index`: Start byte index in `input`.
            ///
            /// # Returns
            ///
            /// Returns the decoded floating-point value and the non-zero number
            /// of consumed bytes.
            ///
            /// # Safety
            ///
            /// The caller must guarantee that:
            ///
            /// - `index + Self::MIN_UNITS_PER_VALUE <= input.len()`
            /// - `input[index..index + Self::MIN_UNITS_PER_VALUE]`
            ///   is valid for reading.
            #[must_use]
            #[inline(always)]
            pub unsafe fn decode_unchecked(input: &[u8], index: usize) -> ($ty, core::num::NonZeroUsize) {
                debug_assert!(index + Self::MIN_UNITS_PER_VALUE <= input.len());

                // SAFETY:
                // The caller guarantees that the readable range is fully in-bounds.
                // `read_unaligned` permits unaligned memory access.
                let pointer = unsafe { input.as_ptr().add(index).cast::<$bits>() };

                // SAFETY:
                // The pointer is valid for an unaligned integer load.
                let raw = unsafe { ptr::read_unaligned(pointer) };

                (
                    <$ty>::from_bits(<$bits>::from_be(raw)),
                    // SAFETY: `$ty` is a concrete primitive floating-point type with non-zero size.
                    unsafe { core::num::NonZeroUsize::new_unchecked(Self::MIN_UNITS_PER_VALUE) },
                )
            }

            /// Encodes `value` into `output` starting at `index`
            /// without bounds checks.
            ///
            /// This function is intended for hot binary codec paths where the
            /// caller has already validated the buffer length externally.
            ///
            /// # Parameters
            ///
            /// - `value`: Floating-point value to encode.
            /// - `output`: Destination byte buffer.
            /// - `index`: Start byte index in `output`.
            ///
            /// # Safety
            ///
            /// The caller must guarantee that:
            ///
            /// - `index + Self::MAX_UNITS_PER_VALUE <= output.len()`
            /// - `output[index..index + Self::MAX_UNITS_PER_VALUE]`
            ///   is valid for writing.
            #[inline(always)]
            pub unsafe fn encode_unchecked(value: $ty, output: &mut [u8], index: usize) -> usize {
                debug_assert!(index + Self::MAX_UNITS_PER_VALUE <= output.len());

                let raw = value.to_bits().to_be();

                // SAFETY:
                // The caller guarantees that the writable range is fully in-bounds.
                // `write_unaligned` permits unaligned memory access.
                let pointer = unsafe { output.as_mut_ptr().add(index).cast::<$bits>() };

                // SAFETY:
                // The pointer is valid for an unaligned integer store.
                unsafe {
                    ptr::write_unaligned(pointer, raw);
                }
                Self::MAX_UNITS_PER_VALUE
            }
        }

        unsafe impl Codec for BinaryCodec<$ty, BigEndian> {
            type Value = $ty;
            type Unit = u8;
            type DecodeError = Infallible;
            type EncodeError = Infallible;

            #[inline(always)]
            fn min_units_per_value(&self) -> core::num::NonZeroUsize {
                // SAFETY: `$ty` is a concrete primitive floating-point type with non-zero size.
                unsafe { core::num::NonZeroUsize::new_unchecked(Self::MIN_UNITS_PER_VALUE) }
            }

            #[inline(always)]
            fn max_units_per_value(&self) -> core::num::NonZeroUsize {
                // SAFETY: `$ty` is a concrete primitive floating-point type with non-zero size.
                unsafe { core::num::NonZeroUsize::new_unchecked(Self::MAX_UNITS_PER_VALUE) }
            }

            #[inline(always)]
            unsafe fn decode_unchecked(
                &self,
                input: &[u8],
                index: usize,
            ) -> Result<($ty, core::num::NonZeroUsize), Self::DecodeError> {
                // SAFETY: The caller upholds the `Codec::decode_unchecked` contract.
                Ok(unsafe { Self::decode_unchecked(input, index) })
            }

            #[inline(always)]
            unsafe fn encode_unchecked(
                &self,
                value: &$ty,
                output: &mut [u8],
                index: usize,
            ) -> Result<usize, Self::EncodeError> {
                // SAFETY: The caller upholds the `Codec::encode_unchecked` contract.
                Ok(unsafe { Self::encode_unchecked(*value, output, index) })
            }
        }

        impl BinaryCodec<$ty, LittleEndian> {
            /// Minimum number of bytes required to encode or decode this type.
            pub const MIN_UNITS_PER_VALUE: usize = $len;

            /// Maximum number of bytes required to encode or decode this type.
            pub const MAX_UNITS_PER_VALUE: usize = Self::MIN_UNITS_PER_VALUE;

            /// Decodes a value from `input` starting at `index` without bounds checks.
            ///
            /// This function is intended for hot binary codec paths where the
            /// caller has already validated the buffer length externally.
            ///
            /// # Parameters
            ///
            /// - `input`: Source byte buffer.
            /// - `index`: Start byte index in `input`.
            ///
            /// # Returns
            ///
            /// Returns the decoded floating-point value and the non-zero number
            /// of consumed bytes.
            ///
            /// # Safety
            ///
            /// The caller must guarantee that:
            ///
            /// - `index + Self::MIN_UNITS_PER_VALUE <= input.len()`
            /// - `input[index..index + Self::MIN_UNITS_PER_VALUE]`
            ///   is valid for reading.
            #[must_use]
            #[inline(always)]
            pub unsafe fn decode_unchecked(input: &[u8], index: usize) -> ($ty, core::num::NonZeroUsize) {
                debug_assert!(index + Self::MIN_UNITS_PER_VALUE <= input.len());

                // SAFETY:
                // The caller guarantees that the readable range is fully in-bounds.
                // `read_unaligned` permits unaligned memory access.
                let pointer = unsafe { input.as_ptr().add(index).cast::<$bits>() };

                // SAFETY:
                // The pointer is valid for an unaligned integer load.
                let raw = unsafe { ptr::read_unaligned(pointer) };

                (
                    <$ty>::from_bits(<$bits>::from_le(raw)),
                    // SAFETY: `$ty` is a concrete primitive floating-point type with non-zero size.
                    unsafe { core::num::NonZeroUsize::new_unchecked(Self::MIN_UNITS_PER_VALUE) },
                )
            }

            /// Encodes `value` into `output` starting at `index`
            /// without bounds checks.
            ///
            /// This function is intended for hot binary codec paths where the
            /// caller has already validated the buffer length externally.
            ///
            /// # Parameters
            ///
            /// - `value`: Floating-point value to encode.
            /// - `output`: Destination byte buffer.
            /// - `index`: Start byte index in `output`.
            ///
            /// # Safety
            ///
            /// The caller must guarantee that:
            ///
            /// - `index + Self::MAX_UNITS_PER_VALUE <= output.len()`
            /// - `output[index..index + Self::MAX_UNITS_PER_VALUE]`
            ///   is valid for writing.
            #[inline(always)]
            pub unsafe fn encode_unchecked(value: $ty, output: &mut [u8], index: usize) -> usize {
                debug_assert!(index + Self::MAX_UNITS_PER_VALUE <= output.len());

                let raw = value.to_bits().to_le();

                // SAFETY:
                // The caller guarantees that the writable range is fully in-bounds.
                // `write_unaligned` permits unaligned memory access.
                let pointer = unsafe { output.as_mut_ptr().add(index).cast::<$bits>() };

                // SAFETY:
                // The pointer is valid for an unaligned integer store.
                unsafe {
                    ptr::write_unaligned(pointer, raw);
                }
                Self::MAX_UNITS_PER_VALUE
            }
        }

        unsafe impl Codec for BinaryCodec<$ty, LittleEndian> {
            type Value = $ty;
            type Unit = u8;
            type DecodeError = Infallible;
            type EncodeError = Infallible;

            #[inline(always)]
            fn min_units_per_value(&self) -> core::num::NonZeroUsize {
                // SAFETY: `$ty` is a concrete primitive floating-point type with non-zero size.
                unsafe { core::num::NonZeroUsize::new_unchecked(Self::MIN_UNITS_PER_VALUE) }
            }

            #[inline(always)]
            fn max_units_per_value(&self) -> core::num::NonZeroUsize {
                // SAFETY: `$ty` is a concrete primitive floating-point type with non-zero size.
                unsafe { core::num::NonZeroUsize::new_unchecked(Self::MAX_UNITS_PER_VALUE) }
            }

            #[inline(always)]
            unsafe fn decode_unchecked(
                &self,
                input: &[u8],
                index: usize,
            ) -> Result<($ty, core::num::NonZeroUsize), Self::DecodeError> {
                // SAFETY: The caller upholds the `Codec::decode_unchecked` contract.
                Ok(unsafe { Self::decode_unchecked(input, index) })
            }

            #[inline(always)]
            unsafe fn encode_unchecked(
                &self,
                value: &$ty,
                output: &mut [u8],
                index: usize,
            ) -> Result<usize, Self::EncodeError> {
                // SAFETY: The caller upholds the `Codec::encode_unchecked` contract.
                Ok(unsafe { Self::encode_unchecked(*value, output, index) })
            }
        }
    };
}

impl_integer_binary_codec!(u16, 2);
impl_integer_binary_codec!(u32, 4);
impl_integer_binary_codec!(u64, 8);
impl_integer_binary_codec!(u128, 16);
impl_integer_binary_codec!(i16, 2);
impl_integer_binary_codec!(i32, 4);
impl_integer_binary_codec!(i64, 8);
impl_integer_binary_codec!(i128, 16);
impl_float_binary_codec!(f32, u32, 4);
impl_float_binary_codec!(f64, u64, 8);
