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
    Codec,
    DecodeFailure,
};

use crate::{
    Leb128DecodeError,
    Leb128DecodePolicy,
};

/// Maps detailed LEB128 decoding failures to the generic codec failure type.
///
/// Incomplete input preserves its minimum total byte requirement. Malformed
/// and non-canonical input retains the original error and its consumed byte
/// count so callers can advance past the invalid payload.
#[inline(always)]
pub(in crate::codec) fn map_leb128_decode_failure(
    error: Leb128DecodeError,
) -> DecodeFailure<Leb128DecodeError> {
    if let Some(required) = error.required() {
        DecodeFailure::incomplete_with_source(error, required)
    } else {
        let consumed = error.consumed().expect(
            "invalid LEB128 decode errors always report consumed bytes",
        );
        DecodeFailure::invalid(error, consumed)
    }
}

/// Type-level unchecked LEB128 codec.
///
/// Encoding is always canonical; `P` only affects decoding. Encoding-only
/// callers should conventionally use [`crate::NonStrict`] because no decoding
/// policy is applied on that path.
///
/// # Type Parameters
///
/// - `T`: Integer value type to decode from LEB128 bytes and encode into
///   canonical LEB128 bytes.
/// - `P`: Required type-level decoding policy implementing
///   [`Leb128DecodePolicy`]. Use [`crate::Strict`] to reject non-canonical
///   inputs, or [`crate::NonStrict`] to accept non-canonical inputs. This
///   parameter does not affect canonical encoding.
///
/// The decoding policy is intentionally required so wire-format callers make
/// the canonicality contract explicit.
///
/// ```compile_fail
/// use qubit_codec_binary::Leb128Codec;
///
/// let _ = Leb128Codec::<u64>::default();
/// ```
///
/// # Examples
///
/// ```
/// use qubit_codec_binary::{
///     Leb128Codec,
///     NonStrict,
/// };
///
/// let mut output = [0_u8; Leb128Codec::<u64, NonStrict>::MAX_ENCODE_UNITS_PER_VALUE];
/// let written = unsafe {
///     Leb128Codec::<u64, NonStrict>::encode(300, &mut output, 0)
/// };
/// assert_eq!(2, written);
///
/// let (decoded, consumed) = unsafe {
///     Leb128Codec::<u64, NonStrict>::decode(&output[..written], 0)
/// }.expect("canonical LEB128 value should decode");
/// assert_eq!(300, decoded);
/// assert_eq!(2, consumed.get());
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Leb128Codec<T, P> {
    /// Associates the value and decode-policy types without storing either.
    marker: PhantomData<fn() -> (T, P)>,
}

macro_rules! impl_unsigned_leb128_codec {
    ($ty:ty) => {
        impl<P> Leb128Codec<$ty, P>
        where
            P: Leb128DecodePolicy,
        {
            /// Minimum number of bytes that can represent a complete value.
            pub const MIN_UNITS_PER_VALUE: usize =
                <Self as Codec>::MIN_UNITS_PER_VALUE;

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
            /// Returns [`Leb128DecodeError`] if the bytes are incomplete,
            /// malformed, or strict decoding rejects a non-canonical
            /// representation.
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
            ) -> Result<($ty, core::num::NonZeroUsize), Leb128DecodeError> {
                debug_assert!(
                    input.len().saturating_sub(input_index)
                        >= Self::MIN_UNITS_PER_VALUE
                );

                // SAFETY: The caller guarantees enough readable bytes for this
                // type.
                let (value, consumed) = unsafe {
                    read_uleb_unchecked::<P>(
                        input,
                        input_index,
                        <$ty>::BITS,
                        Self::MAX_DECODE_UNITS_PER_VALUE,
                    )?
                };
                Ok((value as $ty, consumed))
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
            /// The caller must guarantee that the canonical unsigned LEB128
            /// byte width of `value` is writable starting at `output_index`.
            /// Reserving [`Self::MAX_ENCODE_UNITS_PER_VALUE`] bytes always satisfies
            /// this requirement; for a known value, the exact
            /// [`Codec::encode_len`] is sufficient.
            #[must_use = "the returned byte count determines the encoded payload range"]
            #[inline(always)]
            pub unsafe fn encode(
                value: $ty,
                output: &mut [u8],
                output_index: usize,
            ) -> usize {
                // SAFETY: The caller guarantees enough writable bytes for the
                // canonical representation of this value.
                unsafe {
                    write_uleb_unchecked(output, output_index, value as u128)
                }
            }
        }

        impl<P> Codec for Leb128Codec<$ty, P>
        where
            P: Leb128DecodePolicy,
        {
            type Value = $ty;
            type Unit = u8;
            type DecodeError = Leb128DecodeError;
            type EncodeError = Infallible;

            const MIN_UNITS_PER_VALUE: usize = 1;
            const MAX_ENCODE_UNITS_PER_VALUE: usize =
                (<$ty>::BITS as usize).div_ceil(7);
            const MAX_DECODE_UNITS_PER_VALUE: usize =
                (<$ty>::BITS as usize).div_ceil(7);

            #[inline(always)]
            fn encode_len(&self, value: &$ty) -> usize {
                uleb_encoded_len(*value as u128)
            }

            #[inline(always)]
            unsafe fn decode(
                &mut self,
                input: &[u8],
                input_index: usize,
            ) -> Result<
                ($ty, core::num::NonZeroUsize),
                qubit_codec::DecodeFailure<Self::DecodeError>,
            > {
                debug_assert!(
                    input.len().saturating_sub(input_index)
                        >= Self::MIN_UNITS_PER_VALUE
                );

                // SAFETY: The caller upholds the `Codec::decode`
                // contract.
                unsafe { Self::decode(input, input_index) }
                    .map_err(map_leb128_decode_failure)
            }

            #[inline(always)]
            unsafe fn encode(
                &mut self,
                value: &$ty,
                output: &mut [u8],
                output_index: usize,
            ) -> Result<usize, Self::EncodeError> {
                let required = self.encode_len(value);
                debug_assert!(
                    output.len().saturating_sub(output_index) >= required
                );

                // SAFETY: The `Codec::encode` contract provides either the
                // exact canonical width or this type's maximum width.
                let written =
                    unsafe { Self::encode(*value, output, output_index) };
                Ok(written)
            }
        }
    };
}

macro_rules! impl_signed_leb128_codec {
    ($ty:ty) => {
        impl<P> Leb128Codec<$ty, P>
        where
            P: Leb128DecodePolicy,
        {
            /// Minimum number of bytes that can represent a complete value.
            pub const MIN_UNITS_PER_VALUE: usize =
                <Self as Codec>::MIN_UNITS_PER_VALUE;

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
            /// - `index`: Start index in `input`.
            ///
            /// # Returns
            ///
            /// Returns the decoded value and the non-zero number of consumed
            /// bytes.
            ///
            /// # Errors
            ///
            /// Returns [`Leb128DecodeError`] if the bytes are incomplete,
            /// malformed, or strict decoding rejects a non-canonical
            /// representation.
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
            ) -> Result<($ty, core::num::NonZeroUsize), Leb128DecodeError> {
                debug_assert!(
                    input.len().saturating_sub(input_index)
                        >= Self::MIN_UNITS_PER_VALUE
                );

                // SAFETY: The caller guarantees enough readable bytes for this
                // type.
                let (value, consumed) = unsafe {
                    read_sleb_unchecked::<P>(
                        input,
                        input_index,
                        <$ty>::BITS,
                        Self::MAX_DECODE_UNITS_PER_VALUE,
                    )?
                };
                Ok((value as $ty, consumed))
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
            /// # Returns
            ///
            /// Returns the number of written bytes.
            ///
            /// # Safety
            ///
            /// The caller must guarantee that the canonical signed LEB128 byte
            /// width of `value` is writable starting at `output_index`.
            /// Reserving [`Self::MAX_ENCODE_UNITS_PER_VALUE`] bytes always satisfies
            /// this requirement; for a known value, the exact
            /// [`Codec::encode_len`] is sufficient.
            #[must_use = "the returned byte count determines the encoded payload range"]
            #[inline(always)]
            pub unsafe fn encode(
                value: $ty,
                output: &mut [u8],
                output_index: usize,
            ) -> usize {
                // SAFETY: The caller guarantees enough writable bytes for the
                // canonical representation of this value.
                unsafe {
                    write_sleb_unchecked(output, output_index, value as i128)
                }
            }
        }

        impl<P> Codec for Leb128Codec<$ty, P>
        where
            P: Leb128DecodePolicy,
        {
            type Value = $ty;
            type Unit = u8;
            type DecodeError = Leb128DecodeError;
            type EncodeError = Infallible;

            const MIN_UNITS_PER_VALUE: usize = 1;
            const MAX_ENCODE_UNITS_PER_VALUE: usize =
                (<$ty>::BITS as usize).div_ceil(7);
            const MAX_DECODE_UNITS_PER_VALUE: usize =
                (<$ty>::BITS as usize).div_ceil(7);

            #[inline(always)]
            fn encode_len(&self, value: &$ty) -> usize {
                sleb_encoded_len(*value as i128)
            }

            #[inline(always)]
            unsafe fn decode(
                &mut self,
                input: &[u8],
                input_index: usize,
            ) -> Result<
                ($ty, core::num::NonZeroUsize),
                qubit_codec::DecodeFailure<Self::DecodeError>,
            > {
                debug_assert!(
                    input.len().saturating_sub(input_index)
                        >= Self::MIN_UNITS_PER_VALUE
                );

                // SAFETY: The caller upholds the `Codec::decode`
                // contract.
                unsafe { Self::decode(input, input_index) }
                    .map_err(map_leb128_decode_failure)
            }

            #[inline(always)]
            unsafe fn encode(
                &mut self,
                value: &$ty,
                output: &mut [u8],
                output_index: usize,
            ) -> Result<usize, Self::EncodeError> {
                let required = self.encode_len(value);
                debug_assert!(
                    output.len().saturating_sub(output_index) >= required
                );

                // SAFETY: The `Codec::encode` contract provides either the
                // exact canonical width or this type's maximum width.
                let written =
                    unsafe { Self::encode(*value, output, output_index) };
                Ok(written)
            }
        }
    };
}

impl_unsigned_leb128_codec!(u8);
impl_unsigned_leb128_codec!(u16);
impl_unsigned_leb128_codec!(u32);
impl_unsigned_leb128_codec!(u64);
impl_unsigned_leb128_codec!(u128);
impl_unsigned_leb128_codec!(usize);

impl_signed_leb128_codec!(i8);
impl_signed_leb128_codec!(i16);
impl_signed_leb128_codec!(i32);
impl_signed_leb128_codec!(i64);
impl_signed_leb128_codec!(i128);
impl_signed_leb128_codec!(isize);

/// Decodes an unsigned LEB128 value without bounds checks.
///
/// # Type Parameters
///
/// - `P`: Type-level decoding policy used to decide whether non-canonical
///   encodings are accepted.
///
/// # Parameters
///
/// - `input`: Source byte buffer.
/// - `index`: Start index in `input`.
/// - `bits`: Bit width of the target integer type.
/// - `max_bytes`: Maximum number of bytes allowed for that target width.
///
/// # Returns
///
/// Returns the decoded value and the non-zero number of consumed bytes.
///
/// # Errors
///
/// Returns [`Leb128DecodeError`] when the byte sequence is incomplete,
/// malformed, or when `P` rejects a non-canonical encoding.
///
/// # Safety
///
/// The caller must guarantee that at least one byte is readable from `index`.
#[inline]
unsafe fn read_uleb_unchecked<P>(
    input: &[u8],
    index: usize,
    bits: u32,
    max_bytes: usize,
) -> Result<(u128, core::num::NonZeroUsize), Leb128DecodeError>
where
    P: Leb128DecodePolicy,
{
    debug_assert!(input.len().saturating_sub(index) >= 1);

    let available = input.len().saturating_sub(index).min(max_bytes);
    // SAFETY: The caller guarantees that the currently available bytes are
    // readable from `index`.
    match unsafe {
        read_uleb_prefix_unchecked::<P>(
            input, index, bits, max_bytes, available,
        )
    } {
        Ok(Some((value, consumed))) => {
            debug_assert!(consumed > 0);
            // SAFETY: Prefix readers only return `Some` after consuming at
            // least one terminating byte.
            let consumed = qubit_codec::nz!(consumed);
            Ok((value, consumed))
        }
        Ok(None) => {
            debug_assert!(
                available < usize::MAX,
                "available byte count overflowed"
            );
            // SAFETY: Adding one to the available byte count produces a
            // non-zero retry lower bound.
            let required = qubit_codec::nz!(available + 1);
            Err(Leb128DecodeError::incomplete(index, required, available))
        }
        Err(error) => Err(error),
    }
}

/// Tries to decode an unsigned LEB128 value from currently available bytes.
///
/// # Type Parameters
///
/// - `P`: Type-level decoding policy used to decide whether non-canonical
///   encodings are accepted.
///
/// # Parameters
///
/// - `input`: Source byte buffer.
/// - `index`: Start index in `input`.
/// - `bits`: Bit width of the target integer type.
/// - `max_bytes`: Maximum number of bytes allowed for that target width.
/// - `available`: Number of bytes currently readable from `index`.
///
/// # Returns
///
/// Returns `Ok(Some((value, consumed)))` when a complete value is decoded,
/// `Ok(None)` when more bytes are needed, or `Err(error)` when the payload is
/// invalid. Invalid errors carry the consumable byte count.
///
/// # Safety
///
/// The caller must guarantee that `input.as_ptr().add(index)` is valid to read
/// `available` bytes and that `available <= max_bytes`.
#[inline]
unsafe fn read_uleb_prefix_unchecked<P>(
    input: &[u8],
    index: usize,
    bits: u32,
    max_bytes: usize,
    available: usize,
) -> Result<Option<(u128, usize)>, Leb128DecodeError>
where
    P: Leb128DecodePolicy,
{
    debug_assert!(
        available <= max_bytes,
        "available bytes exceed LEB128 maximum width"
    );
    let mut value = 0u128;
    let mut shift = 0u32;
    for offset in 0..available {
        // SAFETY: The caller guarantees enough readable bytes for this loop.
        let byte =
            unsafe { qubit_io::UncheckedSlice::read(input, index + offset) };
        let payload = u128::from(byte & 0x7F);
        value |= payload << shift;
        if byte & 0x80 == 0 {
            if offset == max_bytes - 1
                && !unsigned_final_payload_fits(byte, bits, offset)
            {
                return Err(malformed_decode_error(
                    index,
                    index + offset,
                    offset + 1,
                ));
            }
            let consumed = offset + 1;
            if P::STRICT && !is_canonical_uleb_terminal(offset, byte) {
                return Err(noncanonical_decode_error(index, consumed));
            }
            return Ok(Some((value, consumed)));
        }
        shift += 7;
    }
    if available < max_bytes {
        return Ok(None);
    }
    Err(malformed_decode_error(
        index,
        index + max_bytes - 1,
        max_bytes,
    ))
}

/// Decodes a signed LEB128 value without bounds checks.
///
/// # Type Parameters
///
/// - `P`: Type-level decoding policy used to decide whether non-canonical
///   encodings are accepted.
///
/// # Parameters
///
/// - `input`: Source byte buffer.
/// - `index`: Start index in `input`.
/// - `bits`: Bit width of the target integer type.
/// - `max_bytes`: Maximum number of bytes allowed for that target width.
///
/// # Returns
///
/// Returns the decoded value and the non-zero number of consumed bytes.
///
/// # Errors
///
/// Returns [`Leb128DecodeError`] when the byte sequence is incomplete,
/// malformed, or when `P` rejects a non-canonical encoding.
///
/// # Safety
///
/// The caller must guarantee that at least one byte is readable from `index`.
#[inline]
unsafe fn read_sleb_unchecked<P>(
    input: &[u8],
    index: usize,
    bits: u32,
    max_bytes: usize,
) -> Result<(i128, core::num::NonZeroUsize), Leb128DecodeError>
where
    P: Leb128DecodePolicy,
{
    debug_assert!(input.len().saturating_sub(index) >= 1);

    let available = input.len().saturating_sub(index).min(max_bytes);
    // SAFETY: The caller guarantees that the currently available bytes are
    // readable from `index`.
    match unsafe {
        read_sleb_prefix_unchecked::<P>(
            input, index, bits, max_bytes, available,
        )
    } {
        Ok(Some((value, consumed))) => {
            debug_assert!(consumed > 0);
            // SAFETY: Prefix readers only return `Some` after consuming at
            // least one terminating byte.
            let consumed = qubit_codec::nz!(consumed);
            Ok((value, consumed))
        }
        Ok(None) => {
            debug_assert!(
                available < usize::MAX,
                "available byte count overflowed"
            );
            // SAFETY: Adding one to the available byte count produces a
            // non-zero retry lower bound.
            let required = qubit_codec::nz!(available + 1);
            Err(Leb128DecodeError::incomplete(index, required, available))
        }
        Err(error) => Err(error),
    }
}

/// Tries to decode a signed LEB128 value from currently available bytes.
///
/// # Type Parameters
///
/// - `P`: Type-level decoding policy used to decide whether non-canonical
///   encodings are accepted.
///
/// # Parameters
///
/// - `input`: Source byte buffer.
/// - `index`: Start index in `input`.
/// - `bits`: Bit width of the target integer type.
/// - `max_bytes`: Maximum number of bytes allowed for that target width.
/// - `available`: Number of bytes currently readable from `index`.
///
/// # Returns
///
/// Returns `Ok(Some((value, consumed)))` when a complete value is decoded,
/// `Ok(None)` when more bytes are needed, or `Err(error)` when the payload is
/// invalid. Invalid errors carry the consumable byte count.
///
/// # Safety
///
/// The caller must guarantee that `input.as_ptr().add(index)` is valid to read
/// `available` bytes and that `available <= max_bytes`.
#[inline]
unsafe fn read_sleb_prefix_unchecked<P>(
    input: &[u8],
    index: usize,
    bits: u32,
    max_bytes: usize,
    available: usize,
) -> Result<Option<(i128, usize)>, Leb128DecodeError>
where
    P: Leb128DecodePolicy,
{
    debug_assert!(
        available <= max_bytes,
        "available bytes exceed LEB128 maximum width"
    );
    let mut value = 0i128;
    let mut shift = 0u32;
    let mut previous_byte = 0u8;
    for offset in 0..available {
        // SAFETY: The caller guarantees enough readable bytes for this loop.
        let byte =
            unsafe { qubit_io::UncheckedSlice::read(input, index + offset) };
        let payload = i128::from(byte & 0x7F);
        value |= payload << shift;
        if byte & 0x80 == 0 {
            if offset == max_bytes - 1
                && !signed_final_payload_fits(byte, bits, offset)
            {
                return Err(malformed_decode_error(
                    index,
                    index + offset,
                    offset + 1,
                ));
            }
            if byte & 0x40 != 0 && shift + 7 < i128::BITS {
                value |= (!0i128) << (shift + 7);
            }
            let consumed = offset + 1;
            if P::STRICT
                && !is_canonical_sleb_terminal(offset, previous_byte, byte)
            {
                return Err(noncanonical_decode_error(index, consumed));
            }
            return Ok(Some((value, consumed)));
        }
        previous_byte = byte;
        shift += 7;
    }
    if available < max_bytes {
        return Ok(None);
    }
    Err(malformed_decode_error(
        index,
        index + max_bytes - 1,
        max_bytes,
    ))
}

/// Builds a malformed LEB128 error on the cold error path.
///
/// # Parameters
///
/// - `start_index`: Absolute byte index where the malformed payload starts.
/// - `error_index`: Absolute byte index associated with the malformed payload.
/// - `consumed`: Number of bytes that should be consumed before reporting the
///   error.
///
/// # Returns
///
/// Returns the error carrying the byte count to consume.
#[cold]
fn malformed_decode_error(
    start_index: usize,
    error_index: usize,
    consumed: usize,
) -> Leb128DecodeError {
    debug_assert!(consumed > 0, "malformed LEB128 errors must consume bytes");
    // SAFETY: All malformed call sites pass either `offset + 1` or `max_bytes`,
    // both of which are non-zero for supported LEB128 codecs.
    let consumed = qubit_codec::nz!(consumed);
    Leb128DecodeError::malformed(start_index, error_index, consumed)
}

/// Builds a non-canonical LEB128 error on the cold error path.
///
/// # Parameters
///
/// - `index`: Absolute byte index at which the non-canonical payload starts.
/// - `consumed`: Number of bytes that should be consumed before reporting the
///   error.
///
/// # Returns
///
/// Returns the error carrying the byte count to consume.
#[cold]
fn noncanonical_decode_error(
    index: usize,
    consumed: usize,
) -> Leb128DecodeError {
    debug_assert!(
        consumed > 0,
        "non-canonical LEB128 errors must consume bytes"
    );
    // SAFETY: Non-canonical errors are detected only after reading at least one
    // terminating byte.
    let consumed = qubit_codec::nz!(consumed);
    Leb128DecodeError::noncanonical(index, consumed)
}

/// Checks whether the final unsigned LEB128 payload byte fits the target width.
///
/// # Parameters
///
/// - `byte`: Final LEB128 byte to validate.
/// - `bits`: Bit width of the target unsigned integer type.
/// - `offset`: Zero-based byte offset of `byte` within the encoded value.
///
/// # Returns
///
/// Returns `true` if the unused payload bits are all zero.
#[must_use]
#[inline(always)]
fn unsigned_final_payload_fits(byte: u8, bits: u32, offset: usize) -> bool {
    let used_bits = bits - offset as u32 * 7;
    byte >> used_bits == 0
}

/// Checks whether the final signed LEB128 payload byte fits the target width.
///
/// # Parameters
///
/// - `byte`: Final LEB128 byte to validate.
/// - `bits`: Bit width of the target signed integer type.
/// - `offset`: Zero-based byte offset of `byte` within the encoded value.
///
/// # Returns
///
/// Returns `true` if the unused payload bits are a valid sign extension.
#[must_use]
#[inline(always)]
fn signed_final_payload_fits(byte: u8, bits: u32, offset: usize) -> bool {
    let used_bits = bits - offset as u32 * 7;
    let payload = byte & 0x7F;
    let used_mask = ((1u16 << used_bits) - 1) as u8;
    let unused_mask = 0x7F & !used_mask;
    let sign_bit = 1u8 << (used_bits - 1);
    if payload & sign_bit == 0 {
        payload & unused_mask == 0
    } else {
        payload & unused_mask == unused_mask
    }
}

/// Checks whether an unsigned LEB128 terminal byte is canonical.
///
/// # Parameters
///
/// - `offset`: Zero-based byte offset of the terminal byte.
/// - `byte`: Terminal byte without a continuation flag.
///
/// # Returns
///
/// Returns `true` when the terminal byte is necessary to encode the value.
#[must_use]
#[inline(always)]
fn is_canonical_uleb_terminal(offset: usize, byte: u8) -> bool {
    offset == 0 || byte & 0x7F != 0
}

/// Checks whether a signed LEB128 terminal byte is canonical.
///
/// # Parameters
///
/// - `offset`: Zero-based byte offset of the terminal byte.
/// - `previous_byte`: The preceding continuation byte when `offset` is
///   non-zero.
/// - `byte`: Terminal byte without a continuation flag.
///
/// # Returns
///
/// Returns `true` when the terminal byte is necessary to preserve the signed
/// value represented by the preceding byte.
#[must_use]
#[inline(always)]
fn is_canonical_sleb_terminal(
    offset: usize,
    previous_byte: u8,
    byte: u8,
) -> bool {
    if offset == 0 {
        return true;
    }
    let payload = byte & 0x7F;
    !((payload == 0 && previous_byte & 0x40 == 0)
        || (payload == 0x7F && previous_byte & 0x40 != 0))
}

/// Encodes an unsigned integer as canonical LEB128 without bounds checks.
///
/// # Parameters
///
/// - `output`: Destination byte buffer.
/// - `index`: Start index in `output`.
/// - `value`: Unsigned value to encode.
///
/// # Returns
///
/// Returns the number of written bytes.
///
/// # Safety
///
/// The caller must guarantee that `output.as_mut_ptr().add(index)` is valid to
/// write the full canonical LEB128 representation of `value`.
unsafe fn write_uleb_unchecked(
    output: &mut [u8],
    index: usize,
    mut value: u128,
) -> usize {
    let mut offset = 0;
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        // SAFETY: The caller guarantees enough writable bytes for the encoded
        // value.
        unsafe {
            qubit_io::UncheckedSlice::write(output, index + offset, byte);
        }
        offset += 1;
        if value == 0 {
            return offset;
        }
    }
}

/// Computes the canonical unsigned LEB128 byte width for `value`.
#[must_use]
#[inline(always)]
pub(in crate::codec) fn uleb_encoded_len(mut value: u128) -> usize {
    let mut len = 0;
    loop {
        value >>= 7;
        len += 1;
        if value == 0 {
            return len;
        }
    }
}

/// Encodes a signed integer as canonical LEB128 without bounds checks.
///
/// # Parameters
///
/// - `output`: Destination byte buffer.
/// - `index`: Start index in `output`.
/// - `value`: Signed value to encode.
///
/// # Returns
///
/// Returns the number of written bytes.
///
/// # Safety
///
/// The caller must guarantee that `output.as_mut_ptr().add(index)` is valid to
/// write the full canonical LEB128 representation of `value`.
unsafe fn write_sleb_unchecked(
    output: &mut [u8],
    index: usize,
    mut value: i128,
) -> usize {
    let mut offset = 0;
    loop {
        let mut byte = (value & 0x7F) as u8;
        let sign_bit_set = byte & 0x40 != 0;
        value >>= 7;
        let done =
            (value == 0 && !sign_bit_set) || (value == -1 && sign_bit_set);
        if !done {
            byte |= 0x80;
        }
        // SAFETY: The caller guarantees enough writable bytes for the encoded
        // value.
        unsafe {
            qubit_io::UncheckedSlice::write(output, index + offset, byte);
        }
        offset += 1;
        if done {
            return offset;
        }
    }
}

/// Computes the canonical signed LEB128 byte width for `value`.
#[must_use]
#[inline(always)]
fn sleb_encoded_len(mut value: i128) -> usize {
    let mut len = 0;
    loop {
        let byte = (value & 0x7F) as u8;
        let sign_bit_set = byte & 0x40 != 0;
        value >>= 7;
        let done =
            (value == 0 && !sign_bit_set) || (value == -1 && sign_bit_set);
        len += 1;
        if done {
            return len;
        }
    }
}
