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

use qubit_codec::{
    BigEndian,
    Codec,
    LittleEndian,
    NativeEndian,
};
use qubit_io::UncheckedSlice;

/// Type-level unchecked binary codec for one scalar type and one byte order.
///
/// `BinaryCodec` is intentionally a zero-sized codec type. It exposes
/// type-level unchecked helpers for direct hot-path use and also implements
/// [`Codec`] for generic codec pipelines. Callers must validate buffer lengths
/// before entering the hot path.
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
/// use qubit_codec::BigEndian;
/// use qubit_codec_binary::BinaryCodec;
///
/// let mut output = [0_u8; BinaryCodec::<u32, BigEndian>::MAX_ENCODE_UNITS_PER_VALUE];
/// let written = unsafe {
///     BinaryCodec::<u32, BigEndian>::encode(0x0102_0304, &mut output, 0)
/// };
/// assert_eq!(4, written);
/// assert_eq!([1, 2, 3, 4], output);
///
/// let (decoded, consumed) = unsafe {
///     BinaryCodec::<u32, BigEndian>::decode(&output, 0)
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
    pub const MIN_UNITS_PER_VALUE: usize = <Self as Codec>::MIN_UNITS_PER_VALUE;

    /// Maximum number of bytes emitted when encoding this type.
    pub const MAX_ENCODE_UNITS_PER_VALUE: usize =
        <Self as Codec>::MAX_ENCODE_UNITS_PER_VALUE;

    /// Maximum number of bytes consumed when decoding this type.
    pub const MAX_DECODE_UNITS_PER_VALUE: usize =
        <Self as Codec>::MAX_DECODE_UNITS_PER_VALUE;

    /// Decodes a value from `input` starting at `input_index` without bounds
    /// checks.
    ///
    /// # Parameters
    ///
    /// - `input`: Source byte buffer.
    /// - `input_index`: Start index in `input`.
    ///
    /// # Returns
    ///
    /// Returns the decoded value and the non-zero number of consumed bytes.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that `input.as_ptr().add(input_index)` is
    /// valid to read [`Self::MIN_UNITS_PER_VALUE`] bytes.
    #[must_use]
    #[inline(always)]
    pub unsafe fn decode(
        input: &[u8],
        input_index: usize,
    ) -> (u8, core::num::NonZeroUsize) {
        debug_assert!(input_index + Self::MIN_UNITS_PER_VALUE <= input.len());

        // SAFETY: The caller guarantees that the indexed byte is readable.
        (
            unsafe { qubit_io::UncheckedSlice::read(input, input_index) },
            qubit_codec::nz!(Self::MIN_UNITS_PER_VALUE),
        )
    }

    /// Encodes `value` into `output` starting at `output_index` without bounds
    /// checks.
    ///
    /// # Parameters
    ///
    /// - `value`: Value to encode.
    /// - `output`: Destination byte buffer.
    /// - `output_index`: Start index in `output`.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that `output.as_mut_ptr().add(output_index)`
    /// is valid to write [`Self::MAX_ENCODE_UNITS_PER_VALUE`] bytes.
    #[inline(always)]
    pub unsafe fn encode(
        value: u8,
        output: &mut [u8],
        output_index: usize,
    ) -> usize {
        debug_assert!(
            output_index + Self::MAX_ENCODE_UNITS_PER_VALUE <= output.len()
        );

        // SAFETY: The caller guarantees that the indexed byte is writable.
        unsafe {
            qubit_io::UncheckedSlice::write(output, output_index, value);
        }
        Self::MAX_ENCODE_UNITS_PER_VALUE
    }
}

impl<O> Codec for BinaryCodec<u8, O> {
    type Value = u8;
    type Unit = u8;
    type DecodeError = Infallible;
    type EncodeError = Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 1;

    #[inline(always)]
    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<
        (u8, core::num::NonZeroUsize),
        qubit_codec::DecodeFailure<Self::DecodeError>,
    > {
        // SAFETY: The caller upholds the `Codec::decode` contract.
        Ok(unsafe { Self::decode(input, input_index) })
    }

    #[inline(always)]
    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        // SAFETY: The caller upholds the `Codec::encode` contract.
        unsafe {
            Self::encode(*value, output, output_index);
        }
        Ok(Self::MAX_ENCODE_UNITS_PER_VALUE)
    }
}

impl<O> BinaryCodec<i8, O> {
    /// Minimum number of bytes required to encode or decode this type.
    pub const MIN_UNITS_PER_VALUE: usize = <Self as Codec>::MIN_UNITS_PER_VALUE;

    /// Maximum number of bytes emitted when encoding this type.
    pub const MAX_ENCODE_UNITS_PER_VALUE: usize =
        <Self as Codec>::MAX_ENCODE_UNITS_PER_VALUE;

    /// Maximum number of bytes consumed when decoding this type.
    pub const MAX_DECODE_UNITS_PER_VALUE: usize =
        <Self as Codec>::MAX_DECODE_UNITS_PER_VALUE;

    /// Decodes a value from `input` starting at `input_index` without bounds
    /// checks.
    ///
    /// # Parameters
    ///
    /// - `input`: Source byte buffer.
    /// - `input_index`: Start index in `input`.
    ///
    /// # Returns
    ///
    /// Returns the decoded value and the non-zero number of consumed bytes.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that `input.as_ptr().add(input_index)` is
    /// valid to read [`Self::MIN_UNITS_PER_VALUE`] bytes.
    #[must_use]
    #[inline(always)]
    pub unsafe fn decode(
        input: &[u8],
        input_index: usize,
    ) -> (i8, core::num::NonZeroUsize) {
        debug_assert!(input_index + Self::MIN_UNITS_PER_VALUE <= input.len());

        // SAFETY: The caller guarantees that the indexed byte is readable.
        (
            unsafe { UncheckedSlice::read(input, input_index) } as i8,
            qubit_codec::nz!(Self::MIN_UNITS_PER_VALUE),
        )
    }

    /// Encodes `value` into `output` starting at `output_index` without bounds
    /// checks.
    ///
    /// # Parameters
    ///
    /// - `value`: Value to encode.
    /// - `output`: Destination byte buffer.
    /// - `output_index`: Start index in `output`.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that `output.as_mut_ptr().add(output_index)`
    /// is valid to write [`Self::MAX_ENCODE_UNITS_PER_VALUE`] bytes.
    #[inline(always)]
    pub unsafe fn encode(
        value: i8,
        output: &mut [u8],
        output_index: usize,
    ) -> usize {
        debug_assert!(
            output_index + Self::MAX_ENCODE_UNITS_PER_VALUE <= output.len()
        );

        // SAFETY: The caller guarantees that the indexed byte is writable.
        unsafe {
            qubit_io::UncheckedSlice::write(output, output_index, value as u8);
        }
        Self::MAX_ENCODE_UNITS_PER_VALUE
    }
}

impl<O> Codec for BinaryCodec<i8, O> {
    type Value = i8;
    type Unit = u8;
    type DecodeError = Infallible;
    type EncodeError = Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 1;

    #[inline(always)]
    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<
        (i8, core::num::NonZeroUsize),
        qubit_codec::DecodeFailure<Self::DecodeError>,
    > {
        // SAFETY: The caller upholds the `Codec::decode` contract.
        Ok(unsafe { Self::decode(input, input_index) })
    }

    #[inline(always)]
    unsafe fn encode(
        &mut self,
        value: &i8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        // SAFETY: The caller upholds the `Codec::encode` contract.
        unsafe {
            Self::encode(*value, output, output_index);
        }
        Ok(Self::MAX_ENCODE_UNITS_PER_VALUE)
    }
}

macro_rules! impl_integer_binary_codec {
    ($ty:ty, $len:expr) => {
        impl BinaryCodec<$ty, BigEndian> {
            /// Minimum number of bytes required to encode or decode this type.
            pub const MIN_UNITS_PER_VALUE: usize =
                <Self as Codec>::MIN_UNITS_PER_VALUE;

            /// Maximum number of bytes emitted when encoding this type.
            pub const MAX_ENCODE_UNITS_PER_VALUE: usize =
                <Self as Codec>::MAX_ENCODE_UNITS_PER_VALUE;

            /// Maximum number of bytes consumed when decoding this type.
            pub const MAX_DECODE_UNITS_PER_VALUE: usize =
                <Self as Codec>::MAX_DECODE_UNITS_PER_VALUE;

            /// Decodes a value from `input` starting at `index` without bounds
            /// checks.
            ///
            /// This function is intended for hot binary codec paths where the
            /// caller has already validated the buffer length externally.
            ///
            /// # Parameters
            ///
            /// - `input`: Source byte buffer.
            /// - `input_index`: Start byte index in `input`.
            ///
            /// # Returns
            ///
            /// Returns the decoded value and the non-zero number of consumed
            /// bytes.
            ///
            /// # Safety
            ///
            /// The caller must guarantee that:
            ///
            /// - `input_index + Self::MIN_UNITS_PER_VALUE <= input.len()`
            /// - `input[input_index..input_index + Self::MIN_UNITS_PER_VALUE]`
            ///   is valid for reading.
            #[must_use]
            #[inline(always)]
            pub unsafe fn decode(
                input: &[u8],
                input_index: usize,
            ) -> ($ty, core::num::NonZeroUsize) {
                // SAFETY:
                // The caller guarantees that the readable range is fully
                // in-bounds. This unaligned helper handles byte-aligned load.
                let raw = unsafe {
                    UncheckedSlice::read_ne_unaligned(input, input_index)
                };

                (
                    <$ty>::from_be(raw),
                    qubit_codec::nz!(Self::MIN_UNITS_PER_VALUE),
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
            /// - `output_index`: Start byte index in `output`.
            ///
            /// # Safety
            ///
            /// The caller must guarantee that:
            ///
            /// - `output_index + Self::MAX_ENCODE_UNITS_PER_VALUE <=
            ///   output.len()`
            /// - `output[output_index..output_index +
            ///   Self::MAX_ENCODE_UNITS_PER_VALUE]` is valid for writing.
            #[inline(always)]
            pub unsafe fn encode(
                value: $ty,
                output: &mut [u8],
                output_index: usize,
            ) -> usize {
                let raw = value.to_be();

                // SAFETY:
                // The caller guarantees that the writable range is fully
                // in-bounds. This unaligned helper handles byte-aligned store.
                unsafe {
                    UncheckedSlice::write_ne_unaligned::<$ty>(
                        output,
                        output_index,
                        raw,
                    );
                }
                Self::MAX_ENCODE_UNITS_PER_VALUE
            }
        }

        impl Codec for BinaryCodec<$ty, BigEndian> {
            type Value = $ty;
            type Unit = u8;
            type DecodeError = Infallible;
            type EncodeError = Infallible;

            const MIN_UNITS_PER_VALUE: usize = $len;
            const MAX_ENCODE_UNITS_PER_VALUE: usize = $len;

            const MAX_DECODE_UNITS_PER_VALUE: usize = $len;

            #[inline(always)]
            unsafe fn decode(
                &mut self,
                input: &[u8],
                input_index: usize,
            ) -> Result<
                ($ty, core::num::NonZeroUsize),
                qubit_codec::DecodeFailure<Self::DecodeError>,
            > {
                // SAFETY: The caller upholds the `Codec::decode`
                // contract.
                Ok(unsafe { Self::decode(input, input_index) })
            }

            #[inline(always)]
            unsafe fn encode(
                &mut self,
                value: &$ty,
                output: &mut [u8],
                output_index: usize,
            ) -> Result<usize, Self::EncodeError> {
                // SAFETY: The caller upholds the `Codec::encode`
                // contract.
                unsafe {
                    Self::encode(*value, output, output_index);
                }
                Ok(Self::MAX_ENCODE_UNITS_PER_VALUE)
            }
        }

        impl BinaryCodec<$ty, LittleEndian> {
            /// Minimum number of bytes required to encode or decode this type.
            pub const MIN_UNITS_PER_VALUE: usize =
                <Self as Codec>::MIN_UNITS_PER_VALUE;

            /// Maximum number of bytes emitted when encoding this type.
            pub const MAX_ENCODE_UNITS_PER_VALUE: usize =
                <Self as Codec>::MAX_ENCODE_UNITS_PER_VALUE;

            /// Maximum number of bytes consumed when decoding this type.
            pub const MAX_DECODE_UNITS_PER_VALUE: usize =
                <Self as Codec>::MAX_DECODE_UNITS_PER_VALUE;

            /// Decodes a value from `input` starting at `index` without bounds
            /// checks.
            ///
            /// This function is intended for hot binary codec paths where the
            /// caller has already validated the buffer length externally.
            ///
            /// # Parameters
            ///
            /// - `input`: Source byte buffer.
            /// - `input_index`: Start byte index in `input`.
            ///
            /// # Returns
            ///
            /// Returns the decoded value and the non-zero number of consumed
            /// bytes.
            ///
            /// # Safety
            ///
            /// The caller must guarantee that:
            ///
            /// - `input_index + Self::MIN_UNITS_PER_VALUE <= input.len()`
            /// - `input[input_index..input_index + Self::MIN_UNITS_PER_VALUE]`
            ///   is valid for reading.
            #[must_use]
            #[inline(always)]
            pub unsafe fn decode(
                input: &[u8],
                input_index: usize,
            ) -> ($ty, core::num::NonZeroUsize) {
                // SAFETY:
                // The caller guarantees that the readable range is fully
                // in-bounds. This unaligned helper handles byte-aligned load.
                let raw = unsafe {
                    UncheckedSlice::read_ne_unaligned(input, input_index)
                };

                (
                    <$ty>::from_le(raw),
                    qubit_codec::nz!(Self::MIN_UNITS_PER_VALUE),
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
            /// - `output_index`: Start byte index in `output`.
            ///
            /// # Safety
            ///
            /// The caller must guarantee that:
            ///
            /// - `output_index + Self::MAX_ENCODE_UNITS_PER_VALUE <=
            ///   output.len()`
            /// - `output[output_index..output_index +
            ///   Self::MAX_ENCODE_UNITS_PER_VALUE]` is valid for writing.
            #[inline(always)]
            pub unsafe fn encode(
                value: $ty,
                output: &mut [u8],
                output_index: usize,
            ) -> usize {
                let raw = value.to_le();

                // SAFETY:
                // The caller guarantees that the writable range is fully
                // in-bounds. This unaligned helper handles byte-aligned store.
                unsafe {
                    UncheckedSlice::write_ne_unaligned::<$ty>(
                        output,
                        output_index,
                        raw,
                    );
                }
                Self::MAX_ENCODE_UNITS_PER_VALUE
            }
        }

        impl Codec for BinaryCodec<$ty, LittleEndian> {
            type Value = $ty;
            type Unit = u8;
            type DecodeError = Infallible;
            type EncodeError = Infallible;

            const MIN_UNITS_PER_VALUE: usize = $len;
            const MAX_ENCODE_UNITS_PER_VALUE: usize = $len;

            const MAX_DECODE_UNITS_PER_VALUE: usize = $len;

            #[inline(always)]
            unsafe fn decode(
                &mut self,
                input: &[u8],
                input_index: usize,
            ) -> Result<
                ($ty, core::num::NonZeroUsize),
                qubit_codec::DecodeFailure<Self::DecodeError>,
            > {
                // SAFETY: The caller upholds the `Codec::decode`
                // contract.
                Ok(unsafe { Self::decode(input, input_index) })
            }

            #[inline(always)]
            unsafe fn encode(
                &mut self,
                value: &$ty,
                output: &mut [u8],
                output_index: usize,
            ) -> Result<usize, Self::EncodeError> {
                // SAFETY: The caller upholds the `Codec::encode`
                // contract.
                unsafe {
                    Self::encode(*value, output, output_index);
                }
                Ok(Self::MAX_ENCODE_UNITS_PER_VALUE)
            }
        }
    };
}

macro_rules! impl_float_binary_codec {
    ($ty:ty, $bits:ty, $len:expr) => {
        impl BinaryCodec<$ty, BigEndian> {
            /// Minimum number of bytes required to encode or decode this type.
            pub const MIN_UNITS_PER_VALUE: usize =
                <Self as Codec>::MIN_UNITS_PER_VALUE;

            /// Maximum number of bytes emitted when encoding this type.
            pub const MAX_ENCODE_UNITS_PER_VALUE: usize =
                <Self as Codec>::MAX_ENCODE_UNITS_PER_VALUE;

            /// Maximum number of bytes consumed when decoding this type.
            pub const MAX_DECODE_UNITS_PER_VALUE: usize =
                <Self as Codec>::MAX_DECODE_UNITS_PER_VALUE;

            /// Decodes a value from `input` starting at `index` without bounds
            /// checks.
            ///
            /// This function is intended for hot binary codec paths where the
            /// caller has already validated the buffer length externally.
            ///
            /// # Parameters
            ///
            /// - `input`: Source byte buffer.
            /// - `input_index`: Start byte index in `input`.
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
            /// - `input_index + Self::MIN_UNITS_PER_VALUE <= input.len()`
            /// - `input[input_index..input_index + Self::MIN_UNITS_PER_VALUE]`
            ///   is valid for reading.
            #[must_use]
            #[inline(always)]
            pub unsafe fn decode(
                input: &[u8],
                input_index: usize,
            ) -> ($ty, core::num::NonZeroUsize) {
                // SAFETY:
                // The caller guarantees that the readable range is fully
                // in-bounds. This unaligned helper handles byte-aligned load.
                let raw = unsafe {
                    UncheckedSlice::read_ne_unaligned(input, input_index)
                };

                (
                    <$ty>::from_bits(<$bits>::from_be(raw)),
                    qubit_codec::nz!(Self::MIN_UNITS_PER_VALUE),
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
            /// - `output_index`: Start byte index in `output`.
            ///
            /// # Safety
            ///
            /// The caller must guarantee that:
            ///
            /// - `output_index + Self::MAX_ENCODE_UNITS_PER_VALUE <=
            ///   output.len()`
            /// - `output[output_index..output_index +
            ///   Self::MAX_ENCODE_UNITS_PER_VALUE]` is valid for writing.
            #[inline(always)]
            pub unsafe fn encode(
                value: $ty,
                output: &mut [u8],
                output_index: usize,
            ) -> usize {
                let raw = value.to_bits().to_be();

                // SAFETY:
                // The caller guarantees that the writable range is fully
                // in-bounds. This unaligned helper handles byte-aligned store.
                unsafe {
                    UncheckedSlice::write_ne_unaligned::<$bits>(
                        output,
                        output_index,
                        raw,
                    );
                }
                Self::MAX_ENCODE_UNITS_PER_VALUE
            }
        }

        impl Codec for BinaryCodec<$ty, BigEndian> {
            type Value = $ty;
            type Unit = u8;
            type DecodeError = Infallible;
            type EncodeError = Infallible;

            const MIN_UNITS_PER_VALUE: usize = $len;
            const MAX_ENCODE_UNITS_PER_VALUE: usize = $len;

            const MAX_DECODE_UNITS_PER_VALUE: usize = $len;

            #[inline(always)]
            unsafe fn decode(
                &mut self,
                input: &[u8],
                input_index: usize,
            ) -> Result<
                ($ty, core::num::NonZeroUsize),
                qubit_codec::DecodeFailure<Self::DecodeError>,
            > {
                // SAFETY: The caller upholds the `Codec::decode`
                // contract.
                Ok(unsafe { Self::decode(input, input_index) })
            }

            #[inline(always)]
            unsafe fn encode(
                &mut self,
                value: &$ty,
                output: &mut [u8],
                output_index: usize,
            ) -> Result<usize, Self::EncodeError> {
                // SAFETY: The caller upholds the `Codec::encode`
                // contract.
                unsafe {
                    Self::encode(*value, output, output_index);
                }
                Ok(Self::MAX_ENCODE_UNITS_PER_VALUE)
            }
        }

        impl BinaryCodec<$ty, LittleEndian> {
            /// Minimum number of bytes required to encode or decode this type.
            pub const MIN_UNITS_PER_VALUE: usize =
                <Self as Codec>::MIN_UNITS_PER_VALUE;

            /// Maximum number of bytes emitted when encoding this type.
            pub const MAX_ENCODE_UNITS_PER_VALUE: usize =
                <Self as Codec>::MAX_ENCODE_UNITS_PER_VALUE;

            /// Maximum number of bytes consumed when decoding this type.
            pub const MAX_DECODE_UNITS_PER_VALUE: usize =
                <Self as Codec>::MAX_DECODE_UNITS_PER_VALUE;

            /// Decodes a value from `input` starting at `index` without bounds
            /// checks.
            ///
            /// This function is intended for hot binary codec paths where the
            /// caller has already validated the buffer length externally.
            ///
            /// # Parameters
            ///
            /// - `input`: Source byte buffer.
            /// - `input_index`: Start byte index in `input`.
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
            /// - `input_index + Self::MIN_UNITS_PER_VALUE <= input.len()`
            /// - `input[input_index..input_index + Self::MIN_UNITS_PER_VALUE]`
            ///   is valid for reading.
            #[must_use]
            #[inline(always)]
            pub unsafe fn decode(
                input: &[u8],
                input_index: usize,
            ) -> ($ty, core::num::NonZeroUsize) {
                // SAFETY:
                // The caller guarantees that the readable range is fully
                // in-bounds. This unaligned helper handles byte-aligned load.
                let raw = unsafe {
                    UncheckedSlice::read_ne_unaligned(input, input_index)
                };

                (
                    <$ty>::from_bits(<$bits>::from_le(raw)),
                    qubit_codec::nz!(Self::MIN_UNITS_PER_VALUE),
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
            /// - `output_index`: Start byte index in `output`.
            ///
            /// # Safety
            ///
            /// The caller must guarantee that:
            ///
            /// - `output_index + Self::MAX_ENCODE_UNITS_PER_VALUE <=
            ///   output.len()`
            /// - `output[output_index..output_index +
            ///   Self::MAX_ENCODE_UNITS_PER_VALUE]` is valid for writing.
            #[inline(always)]
            pub unsafe fn encode(
                value: $ty,
                output: &mut [u8],
                output_index: usize,
            ) -> usize {
                let raw = value.to_bits().to_le();

                // SAFETY:
                // The caller guarantees that the writable range is fully
                // in-bounds. This unaligned helper handles byte-aligned store.
                unsafe {
                    UncheckedSlice::write_ne_unaligned::<$bits>(
                        output,
                        output_index,
                        raw,
                    );
                }
                Self::MAX_ENCODE_UNITS_PER_VALUE
            }
        }

        impl Codec for BinaryCodec<$ty, LittleEndian> {
            type Value = $ty;
            type Unit = u8;
            type DecodeError = Infallible;
            type EncodeError = Infallible;

            const MIN_UNITS_PER_VALUE: usize = $len;
            const MAX_ENCODE_UNITS_PER_VALUE: usize = $len;

            const MAX_DECODE_UNITS_PER_VALUE: usize = $len;

            #[inline(always)]
            unsafe fn decode(
                &mut self,
                input: &[u8],
                input_index: usize,
            ) -> Result<
                ($ty, core::num::NonZeroUsize),
                qubit_codec::DecodeFailure<Self::DecodeError>,
            > {
                // SAFETY: The caller upholds the `Codec::decode`
                // contract.
                Ok(unsafe { Self::decode(input, input_index) })
            }

            #[inline(always)]
            unsafe fn encode(
                &mut self,
                value: &$ty,
                output: &mut [u8],
                output_index: usize,
            ) -> Result<usize, Self::EncodeError> {
                // SAFETY: The caller upholds the `Codec::encode`
                // contract.
                unsafe {
                    Self::encode(*value, output, output_index);
                }
                Ok(Self::MAX_ENCODE_UNITS_PER_VALUE)
            }
        }
    };
}

macro_rules! impl_native_integer_binary_codec {
    ($ty:ty, $len:expr) => {
        impl BinaryCodec<$ty, NativeEndian> {
            pub const MIN_UNITS_PER_VALUE: usize = <Self as Codec>::MIN_UNITS_PER_VALUE;
            pub const MAX_ENCODE_UNITS_PER_VALUE: usize = <Self as Codec>::MAX_ENCODE_UNITS_PER_VALUE;
            pub const MAX_DECODE_UNITS_PER_VALUE: usize = <Self as Codec>::MAX_DECODE_UNITS_PER_VALUE;

            #[must_use]
            #[inline(always)]
            pub unsafe fn decode(input: &[u8], input_index: usize) -> ($ty, core::num::NonZeroUsize) {
                let raw = unsafe { UncheckedSlice::read_ne_unaligned(input, input_index) };
                (raw, qubit_codec::nz!(Self::MIN_UNITS_PER_VALUE))
            }

            #[inline(always)]
            pub unsafe fn encode(value: $ty, output: &mut [u8], output_index: usize) -> usize {
                unsafe { UncheckedSlice::write_ne_unaligned(output, output_index, value) };
                Self::MAX_ENCODE_UNITS_PER_VALUE
            }
        }

        impl Codec for BinaryCodec<$ty, NativeEndian> {
            type Value = $ty;
            type Unit = u8;
            type DecodeError = Infallible;
            type EncodeError = Infallible;
            const MIN_UNITS_PER_VALUE: usize = $len;
            const MAX_ENCODE_UNITS_PER_VALUE: usize = $len;
            const MAX_DECODE_UNITS_PER_VALUE: usize = $len;

            #[inline(always)]
            unsafe fn decode(&mut self, input: &[u8], input_index: usize) -> Result<($ty, core::num::NonZeroUsize), qubit_codec::DecodeFailure<Self::DecodeError>> {
                Ok(unsafe { Self::decode(input, input_index) })
            }

            #[inline(always)]
            unsafe fn encode(&mut self, value: &$ty, output: &mut [u8], output_index: usize) -> Result<usize, Self::EncodeError> {
                unsafe { Self::encode(*value, output, output_index) };
                Ok(Self::MAX_ENCODE_UNITS_PER_VALUE)
            }
        }
    };
}

macro_rules! impl_native_float_binary_codec {
    ($ty:ty, $bits:ty, $len:expr) => {
        impl BinaryCodec<$ty, NativeEndian> {
            pub const MIN_UNITS_PER_VALUE: usize = <Self as Codec>::MIN_UNITS_PER_VALUE;
            pub const MAX_ENCODE_UNITS_PER_VALUE: usize = <Self as Codec>::MAX_ENCODE_UNITS_PER_VALUE;
            pub const MAX_DECODE_UNITS_PER_VALUE: usize = <Self as Codec>::MAX_DECODE_UNITS_PER_VALUE;

            #[must_use]
            #[inline(always)]
            pub unsafe fn decode(input: &[u8], input_index: usize) -> ($ty, core::num::NonZeroUsize) {
                let raw = unsafe { UncheckedSlice::read_ne_unaligned(input, input_index) };
                (<$ty>::from_bits(raw), qubit_codec::nz!(Self::MIN_UNITS_PER_VALUE))
            }

            #[inline(always)]
            pub unsafe fn encode(value: $ty, output: &mut [u8], output_index: usize) -> usize {
                unsafe { UncheckedSlice::write_ne_unaligned(output, output_index, value.to_bits()) };
                Self::MAX_ENCODE_UNITS_PER_VALUE
            }
        }

        impl Codec for BinaryCodec<$ty, NativeEndian> {
            type Value = $ty;
            type Unit = u8;
            type DecodeError = Infallible;
            type EncodeError = Infallible;
            const MIN_UNITS_PER_VALUE: usize = $len;
            const MAX_ENCODE_UNITS_PER_VALUE: usize = $len;
            const MAX_DECODE_UNITS_PER_VALUE: usize = $len;

            #[inline(always)]
            unsafe fn decode(&mut self, input: &[u8], input_index: usize) -> Result<($ty, core::num::NonZeroUsize), qubit_codec::DecodeFailure<Self::DecodeError>> {
                Ok(unsafe { Self::decode(input, input_index) })
            }

            #[inline(always)]
            unsafe fn encode(&mut self, value: &$ty, output: &mut [u8], output_index: usize) -> Result<usize, Self::EncodeError> {
                unsafe { Self::encode(*value, output, output_index) };
                Ok(Self::MAX_ENCODE_UNITS_PER_VALUE)
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
impl_native_integer_binary_codec!(u16, 2);
impl_native_integer_binary_codec!(u32, 4);
impl_native_integer_binary_codec!(u64, 8);
impl_native_integer_binary_codec!(u128, 16);
impl_native_integer_binary_codec!(i16, 2);
impl_native_integer_binary_codec!(i32, 4);
impl_native_integer_binary_codec!(i64, 8);
impl_native_integer_binary_codec!(i128, 16);
impl_native_float_binary_codec!(f32, u32, 4);
impl_native_float_binary_codec!(f64, u64, 8);
