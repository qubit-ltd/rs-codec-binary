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
};

use qubit_codec::Codec;

use crate::{
    DecodePolicy,
    Leb128DecodeError,
    NonStrict,
};

/// Type-level unchecked LEB128 codec.
///
/// Encoding is always canonical; `P` only affects decoding.
///
/// # Type Parameters
///
/// - `T`: Integer value type to decode from LEB128 bytes and encode into
///   canonical LEB128 bytes.
/// - `P`: Type-level decoding policy implementing [`DecodePolicy`]. Use
///   [`crate::Strict`] to reject non-canonical inputs, or [`NonStrict`] to
///   accept non-canonical inputs.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Leb128Codec<T, P = NonStrict> {
    marker: PhantomData<fn() -> (T, P)>,
}

macro_rules! impl_unsigned_leb128_codec {
    ($ty:ty) => {
        impl<P> Leb128Codec<$ty, P>
        where
            P: DecodePolicy,
        {
            /// Minimum number of bytes that can represent a complete value.
            pub const MIN_UNITS_PER_VALUE: usize = 1;

            /// Maximum number of bytes required to encode or decode this type.
            pub const MAX_UNITS_PER_VALUE: usize = (<$ty>::BITS as usize).div_ceil(7);

            /// Decodes a value from `input` starting at `index` without bounds checks.
            ///
            /// # Parameters
            ///
            /// - `input`: Source byte buffer.
            /// - `index`: Start index in `input`.
            ///
            /// # Returns
            ///
            /// Returns the decoded value and the number of consumed bytes.
            ///
            /// # Errors
            ///
            /// Returns [`Leb128DecodeError`] if the bytes are incomplete,
            /// malformed, or strict decoding rejects a non-canonical
            /// representation.
            ///
            /// # Safety
            ///
            /// The caller must guarantee that `index` is a valid boundary and
            /// at least [`Self::MIN_UNITS_PER_VALUE`] byte is readable from
            /// `index`.
            #[inline(always)]
            pub unsafe fn decode_unchecked(input: &[u8], index: usize) -> Result<($ty, usize), Leb128DecodeError> {
                debug_assert!(input.len().saturating_sub(index) >= Self::MIN_UNITS_PER_VALUE);

                // SAFETY: The caller guarantees enough readable bytes for this type.
                let (value, consumed) =
                    unsafe { read_uleb_unchecked::<P>(input, index, <$ty>::BITS, Self::MAX_UNITS_PER_VALUE)? };
                Ok((value as $ty, consumed))
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
            pub unsafe fn encode_unchecked(value: $ty, output: &mut [u8], index: usize) -> usize {
                // SAFETY: The caller guarantees enough writable bytes for this type.
                unsafe { write_uleb_unchecked(output, index, value as u128) }
            }
        }

        unsafe impl<P> Codec<$ty, u8> for Leb128Codec<$ty, P>
        where
            P: DecodePolicy,
        {
            type DecodeError = Leb128DecodeError;
            type EncodeError = Infallible;

            #[inline(always)]
            fn min_units_per_value(&self) -> usize {
                Self::MIN_UNITS_PER_VALUE
            }

            #[inline(always)]
            fn max_units_per_value(&self) -> usize {
                Self::MAX_UNITS_PER_VALUE
            }

            #[inline(always)]
            unsafe fn decode_unchecked(&self, input: &[u8], index: usize) -> Result<($ty, usize), Self::DecodeError> {
                debug_assert!(input.len().saturating_sub(index) >= Self::MIN_UNITS_PER_VALUE);

                // SAFETY: The caller upholds the `Codec::decode_unchecked` contract.
                unsafe { Self::decode_unchecked(input, index) }
            }

            #[inline(always)]
            unsafe fn encode_unchecked(
                &self,
                value: &$ty,
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

macro_rules! impl_signed_leb128_codec {
    ($ty:ty) => {
        impl<P> Leb128Codec<$ty, P>
        where
            P: DecodePolicy,
        {
            /// Minimum number of bytes that can represent a complete value.
            pub const MIN_UNITS_PER_VALUE: usize = 1;

            /// Maximum number of bytes required to encode or decode this type.
            pub const MAX_UNITS_PER_VALUE: usize = (<$ty>::BITS as usize).div_ceil(7);

            /// Decodes a value from `input` starting at `index` without bounds checks.
            ///
            /// # Parameters
            ///
            /// - `input`: Source byte buffer.
            /// - `index`: Start index in `input`.
            ///
            /// # Returns
            ///
            /// Returns the decoded value and the number of consumed bytes.
            ///
            /// # Errors
            ///
            /// Returns [`Leb128DecodeError`] if the bytes are incomplete,
            /// malformed, or strict decoding rejects a non-canonical
            /// representation.
            ///
            /// # Safety
            ///
            /// The caller must guarantee that `index` is a valid boundary and
            /// at least [`Self::MIN_UNITS_PER_VALUE`] byte is readable from
            /// `index`.
            #[inline(always)]
            pub unsafe fn decode_unchecked(input: &[u8], index: usize) -> Result<($ty, usize), Leb128DecodeError> {
                debug_assert!(input.len().saturating_sub(index) >= Self::MIN_UNITS_PER_VALUE);

                // SAFETY: The caller guarantees enough readable bytes for this type.
                let (value, consumed) =
                    unsafe { read_sleb_unchecked::<P>(input, index, <$ty>::BITS, Self::MAX_UNITS_PER_VALUE)? };
                Ok((value as $ty, consumed))
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
            pub unsafe fn encode_unchecked(value: $ty, output: &mut [u8], index: usize) -> usize {
                // SAFETY: The caller guarantees enough writable bytes for this type.
                unsafe { write_sleb_unchecked(output, index, value as i128) }
            }
        }

        unsafe impl<P> Codec<$ty, u8> for Leb128Codec<$ty, P>
        where
            P: DecodePolicy,
        {
            type DecodeError = Leb128DecodeError;
            type EncodeError = Infallible;

            #[inline(always)]
            fn min_units_per_value(&self) -> usize {
                Self::MIN_UNITS_PER_VALUE
            }

            #[inline(always)]
            fn max_units_per_value(&self) -> usize {
                Self::MAX_UNITS_PER_VALUE
            }

            #[inline(always)]
            unsafe fn decode_unchecked(&self, input: &[u8], index: usize) -> Result<($ty, usize), Self::DecodeError> {
                debug_assert!(input.len().saturating_sub(index) >= Self::MIN_UNITS_PER_VALUE);

                // SAFETY: The caller upholds the `Codec::decode_unchecked` contract.
                unsafe { Self::decode_unchecked(input, index) }
            }

            #[inline(always)]
            unsafe fn encode_unchecked(
                &self,
                value: &$ty,
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
/// Returns the decoded value and the number of consumed bytes.
///
/// # Errors
///
/// Returns [`Leb128DecodeError`] when the byte sequence is incomplete,
/// malformed, or when `P` rejects a non-canonical encoding.
///
/// # Safety
///
/// The caller must guarantee that at least one byte is readable from `index`.
#[inline(always)]
unsafe fn read_uleb_unchecked<P>(
    input: &[u8],
    index: usize,
    bits: u32,
    max_bytes: usize,
) -> Result<(u128, usize), Leb128DecodeError>
where
    P: DecodePolicy,
{
    debug_assert!(input.len().saturating_sub(index) >= 1);

    let available = input.len().saturating_sub(index).min(max_bytes);
    // SAFETY: The caller guarantees that the currently available bytes are
    // readable from `index`.
    match unsafe { read_uleb_prefix_unchecked::<P>(input, index, bits, max_bytes, available) } {
        Ok(Some(value)) => Ok(value),
        Ok(None) => Err(Leb128DecodeError::incomplete(index, available + 1, available)),
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
#[inline(always)]
unsafe fn read_uleb_prefix_unchecked<P>(
    input: &[u8],
    index: usize,
    bits: u32,
    max_bytes: usize,
    available: usize,
) -> Result<Option<(u128, usize)>, Leb128DecodeError>
where
    P: DecodePolicy,
{
    debug_assert!(available <= max_bytes, "available bytes exceed LEB128 maximum width");
    let mut value = 0u128;
    let mut shift = 0u32;
    // SAFETY: The caller guarantees that `available` bytes are readable from
    // `index`, so this base pointer can be advanced by every loop offset.
    let base = unsafe { input.as_ptr().add(index) };
    for offset in 0..available {
        // SAFETY: The caller guarantees enough readable bytes for this loop.
        let byte = unsafe { *base.add(offset) };
        let payload = u128::from(byte & 0x7F);
        value |= payload << shift;
        if byte & 0x80 == 0 {
            if offset == max_bytes - 1 && !unsigned_final_payload_fits(byte, bits, offset) {
                return Err(malformed_decode_error(index + offset, offset + 1));
            }
            let consumed = offset + 1;
            if P::STRICT && !has_canonical_uleb_len(value, consumed) {
                return Err(noncanonical_decode_error(index, consumed));
            }
            return Ok(Some((value, consumed)));
        }
        shift += 7;
    }
    if available < max_bytes {
        return Ok(None);
    }
    Err(malformed_decode_error(index + max_bytes - 1, max_bytes))
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
/// Returns the decoded value and the number of consumed bytes.
///
/// # Errors
///
/// Returns [`Leb128DecodeError`] when the byte sequence is incomplete,
/// malformed, or when `P` rejects a non-canonical encoding.
///
/// # Safety
///
/// The caller must guarantee that at least one byte is readable from `index`.
#[inline(always)]
unsafe fn read_sleb_unchecked<P>(
    input: &[u8],
    index: usize,
    bits: u32,
    max_bytes: usize,
) -> Result<(i128, usize), Leb128DecodeError>
where
    P: DecodePolicy,
{
    debug_assert!(input.len().saturating_sub(index) >= 1);

    let available = input.len().saturating_sub(index).min(max_bytes);
    // SAFETY: The caller guarantees that the currently available bytes are
    // readable from `index`.
    match unsafe { read_sleb_prefix_unchecked::<P>(input, index, bits, max_bytes, available) } {
        Ok(Some(value)) => Ok(value),
        Ok(None) => Err(Leb128DecodeError::incomplete(index, available + 1, available)),
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
#[inline(always)]
unsafe fn read_sleb_prefix_unchecked<P>(
    input: &[u8],
    index: usize,
    bits: u32,
    max_bytes: usize,
    available: usize,
) -> Result<Option<(i128, usize)>, Leb128DecodeError>
where
    P: DecodePolicy,
{
    debug_assert!(available <= max_bytes, "available bytes exceed LEB128 maximum width");
    let mut value = 0i128;
    let mut shift = 0u32;
    // SAFETY: The caller guarantees that `available` bytes are readable from
    // `index`, so this base pointer can be advanced by every loop offset.
    let base = unsafe { input.as_ptr().add(index) };
    for offset in 0..available {
        // SAFETY: The caller guarantees enough readable bytes for this loop.
        let byte = unsafe { *base.add(offset) };
        let payload = i128::from(byte & 0x7F);
        value |= payload << shift;
        if byte & 0x80 == 0 {
            if offset == max_bytes - 1 && !signed_final_payload_fits(byte, bits, offset) {
                return Err(malformed_decode_error(index + offset, offset + 1));
            }
            if byte & 0x40 != 0 && shift + 7 < i128::BITS {
                value |= (!0i128) << (shift + 7);
            }
            let consumed = offset + 1;
            if P::STRICT && !has_canonical_sleb_len(value, consumed) {
                return Err(noncanonical_decode_error(index, consumed));
            }
            return Ok(Some((value, consumed)));
        }
        shift += 7;
    }
    if available < max_bytes {
        return Ok(None);
    }
    Err(malformed_decode_error(index + max_bytes - 1, max_bytes))
}

/// Builds a malformed LEB128 error on the cold error path.
///
/// # Parameters
///
/// - `index`: Absolute byte index associated with the malformed payload.
/// - `consumed`: Number of bytes that should be consumed before reporting the
///   error.
///
/// # Returns
///
/// Returns the error carrying the byte count to consume.
#[cold]
#[inline(never)]
fn malformed_decode_error(index: usize, consumed: usize) -> Leb128DecodeError {
    Leb128DecodeError::malformed(index, consumed)
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
#[inline(never)]
fn noncanonical_decode_error(index: usize, consumed: usize) -> Leb128DecodeError {
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

/// Checks whether an unsigned LEB128 value used its canonical encoded length.
///
/// # Parameters
///
/// - `value`: Decoded unsigned value.
/// - `actual_len`: Number of bytes consumed from the input.
///
/// # Returns
///
/// Returns `true` if `actual_len` is the canonical encoded length of `value`.
#[must_use]
#[inline(always)]
fn has_canonical_uleb_len(value: u128, actual_len: usize) -> bool {
    canonical_uleb_len(value) == actual_len
}

/// Checks whether a signed LEB128 value used its canonical encoded length.
///
/// # Parameters
///
/// - `value`: Decoded signed value.
/// - `actual_len`: Number of bytes consumed from the input.
///
/// # Returns
///
/// Returns `true` if `actual_len` is the canonical encoded length of `value`.
#[must_use]
#[inline(always)]
fn has_canonical_sleb_len(value: i128, actual_len: usize) -> bool {
    canonical_sleb_len(value) == actual_len
}

/// Computes the canonical unsigned LEB128 encoded length.
///
/// # Parameters
///
/// - `value`: Unsigned value to measure.
///
/// # Returns
///
/// Returns the number of bytes used by the canonical unsigned LEB128 encoding.
#[must_use]
#[inline(always)]
fn canonical_uleb_len(mut value: u128) -> usize {
    let mut len = 1;
    while value >= 0x80 {
        value >>= 7;
        len += 1;
    }
    len
}

/// Computes the canonical signed LEB128 encoded length.
///
/// # Parameters
///
/// - `value`: Signed value to measure.
///
/// # Returns
///
/// Returns the number of bytes used by the canonical signed LEB128 encoding.
#[must_use]
#[inline(always)]
fn canonical_sleb_len(mut value: i128) -> usize {
    let mut len = 0;
    loop {
        let byte = (value & 0x7F) as u8;
        let sign_bit_set = byte & 0x40 != 0;
        value >>= 7;
        len += 1;
        if (value == 0 && !sign_bit_set) || (value == -1 && sign_bit_set) {
            return len;
        }
    }
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
unsafe fn write_uleb_unchecked(output: &mut [u8], index: usize, mut value: u128) -> usize {
    let mut offset = 0;
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        // SAFETY: The caller guarantees enough writable bytes for the encoded value.
        unsafe {
            *output.as_mut_ptr().add(index + offset) = byte;
        }
        offset += 1;
        if value == 0 {
            return offset;
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
unsafe fn write_sleb_unchecked(output: &mut [u8], index: usize, mut value: i128) -> usize {
    let mut offset = 0;
    loop {
        let mut byte = (value & 0x7F) as u8;
        let sign_bit_set = byte & 0x40 != 0;
        value >>= 7;
        let done = (value == 0 && !sign_bit_set) || (value == -1 && sign_bit_set);
        if !done {
            byte |= 0x80;
        }
        // SAFETY: The caller guarantees enough writable bytes for the encoded value.
        unsafe {
            *output.as_mut_ptr().add(index + offset) = byte;
        }
        offset += 1;
        if done {
            return offset;
        }
    }
}
