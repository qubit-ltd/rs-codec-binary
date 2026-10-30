// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![no_main]

use core::fmt::Debug;
use core::num::NonZeroUsize;

use libfuzzer_sys::fuzz_target;
use qubit_codec::BigEndian;
use qubit_codec::Codec;
use qubit_codec::LittleEndian;
use qubit_codec::NativeEndian;
use qubit_codec_binary::BinaryCodec;
use qubit_codec_binary::Leb128Codec;
use qubit_codec_binary::Leb128DecodeError;
use qubit_codec_binary::Leb128DecodeErrorKind;
use qubit_codec_binary::NonStrict;
use qubit_codec_binary::Strict;
use qubit_codec_binary::ZigZagCodec;

/// Bounds each invocation independently of the fuzzer configuration.
const MAX_FUZZ_INPUT_LEN: usize = 19;

macro_rules! assert_decode_policies {
    ($codec:ident, $ty:ty, $input:expr) => {{
        let non_strict = unsafe { $codec::<$ty, NonStrict>::decode($input, 0) };
        let strict = unsafe { $codec::<$ty, Strict>::decode($input, 0) };
        assert_decode_offset_equivalence(
            $codec::<$ty, NonStrict>::decode,
            $input,
            &non_strict,
            concat!(stringify!($codec), "<", stringify!($ty), ", NonStrict>"),
        );
        assert_decode_offset_equivalence(
            $codec::<$ty, Strict>::decode,
            $input,
            &strict,
            concat!(stringify!($codec), "<", stringify!($ty), ", Strict>"),
        );
        assert_strict_matches_canonical_encoding::<
            $codec<$ty, NonStrict>,
            $ty,
            { $codec::<$ty, NonStrict>::MAX_ENCODE_UNITS_PER_VALUE },
        >($input, &strict, &non_strict);
        assert_policy_results(
            $input,
            $codec::<$ty, NonStrict>::MAX_DECODE_UNITS_PER_VALUE,
            strict,
            non_strict,
        );
    }};
}

/// Checks absolute index translation while preserving relative decode counts.
fn assert_decode_offset_equivalence<T>(
    decode: unsafe fn(&[u8], usize) -> Result<(T, NonZeroUsize), Leb128DecodeError>,
    input: &[u8],
    baseline: &Result<(T, NonZeroUsize), Leb128DecodeError>,
    label: &str,
) where
    T: Debug + PartialEq,
{
    assert!(!input.is_empty());
    assert!(input.len() <= MAX_FUZZ_INPUT_LEN);
    for offset in [1, 3, 7] {
        let mut storage = [0_u8; MAX_FUZZ_INPUT_LEN + 7];
        for (index, byte) in storage[..offset].iter_mut().enumerate() {
            *byte = [0x00, 0x80, 0xff][index % 3];
        }
        let end = offset + input.len();
        storage[offset..end].copy_from_slice(input);
        // SAFETY: input is non-empty, offset is within the exposed slice,
        // and unused storage after end is excluded from the decoder input.
        let actual = unsafe { decode(&storage[..end], offset) };
        match (baseline, &actual) {
            (Ok(expected), Ok(actual)) => {
                assert_eq!(expected, actual, "{label} offset={offset}: success");
            }
            (Err(expected), Err(actual)) => {
                assert_eq!(expected.kind(), actual.kind(), "{label} offset={offset}: kind");
                assert_eq!(
                    expected.start_index() + offset,
                    actual.start_index(),
                    "{label} offset={offset}: start_index"
                );
                assert_eq!(
                    expected.error_index() + offset,
                    actual.error_index(),
                    "{label} offset={offset}: error_index"
                );
                assert_eq!(
                    expected.consumed(),
                    actual.consumed(),
                    "{label} offset={offset}: consumed"
                );
                assert_eq!(
                    expected.required(),
                    actual.required(),
                    "{label} offset={offset}: required"
                );
                assert_eq!(
                    expected.available(),
                    actual.available(),
                    "{label} offset={offset}: available"
                );
                assert_eq!(
                    expected.additional(),
                    actual.additional(),
                    "{label} offset={offset}: additional"
                );
            }
            _ => panic!("{label} offset={offset}: result changed: {baseline:?} -> {actual:?}"),
        }
    }
}

/// Verifies Strict decoding against an independently regenerated canonical
/// representation of every NonStrict-successful input.
fn assert_strict_matches_canonical_encoding<C, T, const N: usize>(
    input: &[u8],
    strict: &Result<(T, NonZeroUsize), Leb128DecodeError>,
    non_strict: &Result<(T, NonZeroUsize), Leb128DecodeError>,
) where
    C: Codec<Value = T, Unit = u8, DecodeError = Leb128DecodeError> + Default,
    C::EncodeError: Debug,
    T: Copy + Debug + PartialEq,
{
    let Ok((value, consumed)) = non_strict else {
        return;
    };

    let mut canonical = [0_u8; N];
    let written = unsafe { Codec::encode(&mut C::default(), value, &mut canonical, 0) }
        .expect("LEB128-family canonical encoding is infallible");
    let canonical = &canonical[..written];
    let encoded_prefix = &input[..consumed.get()];

    match strict {
        Ok((strict_value, strict_consumed)) => {
            assert_eq!(*strict_value, *value);
            assert_eq!(strict_consumed, consumed);
            assert_eq!(encoded_prefix, canonical);
        }
        Err(error) if error.is_noncanonical() => {
            assert_ne!(encoded_prefix, canonical);
        }
        Err(error) => {
            panic!("Strict rejected a NonStrict-successful canonicality candidate as {error:?}");
        }
    }
}

macro_rules! assert_leb128_roundtrip {
    ($codec:ident, $ty:ty, $value:expr) => {{
        let expected: $ty = $value;
        let mut output = [0_u8; $codec::<$ty, NonStrict>::MAX_ENCODE_UNITS_PER_VALUE];
        let written = unsafe { $codec::<$ty, NonStrict>::encode(expected, &mut output, 0) };
        let mut codec = $codec::<$ty, NonStrict>::default();
        let required = codec.encode_len(&expected);
        assert_eq!(written, required);
        assert!((1..=$codec::<$ty, NonStrict>::MAX_ENCODE_UNITS_PER_VALUE).contains(&required));
        assert_eq!($codec::<$ty, Strict>::default().encode_len(&expected), required);
        let mut trait_output = [0_u8; $codec::<$ty, NonStrict>::MAX_ENCODE_UNITS_PER_VALUE];
        // SAFETY: The buffer exposes the declared maximum encode capacity.
        let trait_written = unsafe { Codec::encode(&mut codec, &expected, &mut trait_output, 0) }
            .expect("scalar encoding is infallible");
        assert_eq!(written, trait_written);
        assert_eq!(&output[..written], &trait_output[..trait_written]);
        let mut storage = [0xa5_u8; $codec::<$ty, NonStrict>::MAX_ENCODE_UNITS_PER_VALUE + 2];
        // SAFETY: Both maximum-capacity encodings established this exact width.
        let exact_written = unsafe { Codec::encode(&mut codec, &expected, &mut storage[1..1 + required], 0) }
            .expect("scalar encoding is infallible");
        assert_eq!(written, exact_written);
        assert_eq!(&output[..written], &storage[1..1 + required]);
        assert_eq!(0xa5, storage[0]);
        assert_eq!(0xa5, storage[required + 1]);
        let strict = unsafe { $codec::<$ty, Strict>::decode(&output[..written], 0) }
            .expect("canonical LEB128-family encoding must pass strict decoding");
        let non_strict = unsafe { $codec::<$ty, NonStrict>::decode(&output[..written], 0) }
            .expect("canonical LEB128-family encoding must pass non-strict decoding");

        let (strict_value, strict_consumed) = strict;
        let (non_strict_value, non_strict_consumed) = non_strict;
        assert_eq!((expected, written), (strict_value, strict_consumed.get()));
        assert_eq!((expected, written), (non_strict_value, non_strict_consumed.get()));
    }};
}

macro_rules! assert_binary_roundtrip {
    ($ty:ty, $order:ty, $bytes_method:ident, $value:expr) => {{
        let expected: $ty = $value;
        let expected_bytes = expected.$bytes_method();
        let mut output = [0xA5_u8; BinaryCodec::<$ty, $order>::MAX_ENCODE_UNITS_PER_VALUE + 2];
        // SAFETY: Index one leaves the declared fixed-width capacity and a trailing
        // guard.
        let written = unsafe { BinaryCodec::<$ty, $order>::encode(expected, &mut output, 1) };
        assert_eq!(BinaryCodec::<$ty, $order>::MAX_ENCODE_UNITS_PER_VALUE, written);
        assert_eq!(
            expected_bytes.as_slice(),
            &output[1..1 + written],
            "{} / {} wire bytes",
            stringify!($ty),
            stringify!($order)
        );
        // SAFETY: The same fixed-width range is readable from index one.
        let (actual, consumed) = unsafe { BinaryCodec::<$ty, $order>::decode(&output, 1) };
        assert_eq!(expected, actual);
        assert_eq!(written, consumed.get());
        assert_eq!(0xA5, output[0]);
        assert_eq!(0xA5, output[written + 1]);
    }};
}

macro_rules! assert_binary_float_roundtrip {
    ($ty:ty, $order:ty, $bytes_method:ident, $value:expr) => {{
        let expected: $ty = $value;
        let expected_bytes = expected.to_bits().$bytes_method();
        let mut output = [0xA5_u8; BinaryCodec::<$ty, $order>::MAX_ENCODE_UNITS_PER_VALUE + 2];
        // SAFETY: Index one leaves the declared fixed-width capacity and a trailing
        // guard.
        let written = unsafe { BinaryCodec::<$ty, $order>::encode(expected, &mut output, 1) };
        assert_eq!(BinaryCodec::<$ty, $order>::MAX_ENCODE_UNITS_PER_VALUE, written);
        assert_eq!(
            expected_bytes.as_slice(),
            &output[1..1 + written],
            "{} / {} wire bits",
            stringify!($ty),
            stringify!($order)
        );
        // SAFETY: The same fixed-width range is readable from index one.
        let (actual, consumed) = unsafe { BinaryCodec::<$ty, $order>::decode(&output, 1) };
        assert_eq!(expected.to_bits(), actual.to_bits());
        assert_eq!(written, consumed.get());
        assert_eq!(0xA5, output[0]);
        assert_eq!(0xA5, output[written + 1]);
    }};
}

fuzz_target!(|data: &[u8]| {
    let input = &data[..data.len().min(MAX_FUZZ_INPUT_LEN)];

    decode_arbitrary_input(input);

    let bits = fuzz_u128(input);
    assert_leb128_roundtrips(bits);
    assert_binary_roundtrips(bits);
});

/// Builds a deterministic integer from at most sixteen fuzz input bytes.
fn fuzz_u128(input: &[u8]) -> u128 {
    let mut bytes = [0_u8; size_of::<u128>()];
    for (output, source) in bytes.iter_mut().zip(input.iter().copied()) {
        *output = source;
    }
    u128::from_le_bytes(bytes)
}

/// Exercises both policies on arbitrary non-empty input without violating the
/// unchecked decoder precondition.
fn decode_arbitrary_input(input: &[u8]) {
    if input.is_empty() {
        return;
    }

    assert_decode_policies!(Leb128Codec, u8, input);
    assert_decode_policies!(Leb128Codec, u16, input);
    assert_decode_policies!(Leb128Codec, u32, input);
    assert_decode_policies!(Leb128Codec, u64, input);
    assert_decode_policies!(Leb128Codec, u128, input);
    assert_decode_policies!(Leb128Codec, usize, input);
    assert_decode_policies!(Leb128Codec, i8, input);
    assert_decode_policies!(Leb128Codec, i16, input);
    assert_decode_policies!(Leb128Codec, i32, input);
    assert_decode_policies!(Leb128Codec, i64, input);
    assert_decode_policies!(Leb128Codec, i128, input);
    assert_decode_policies!(Leb128Codec, isize, input);
    assert_decode_policies!(ZigZagCodec, i8, input);
    assert_decode_policies!(ZigZagCodec, i16, input);
    assert_decode_policies!(ZigZagCodec, i32, input);
    assert_decode_policies!(ZigZagCodec, i64, input);
    assert_decode_policies!(ZigZagCodec, i128, input);
    assert_decode_policies!(ZigZagCodec, isize, input);
    assert_reference_decoders(input);
}

/// Verifies result metadata and the relationship between decoding policies.
fn assert_policy_results<T>(
    input: &[u8],
    max_bytes: usize,
    strict: Result<(T, NonZeroUsize), Leb128DecodeError>,
    non_strict: Result<(T, NonZeroUsize), Leb128DecodeError>,
) where
    T: Copy + Debug + PartialEq,
{
    assert_decode_result_metadata(input, max_bytes, &strict, true);
    assert_decode_result_metadata(input, max_bytes, &non_strict, false);

    match (strict, non_strict) {
        (Ok(expected), Ok(actual)) => assert_eq!(expected, actual),
        (Err(error), Ok((_, consumed))) => {
            assert_eq!(Leb128DecodeErrorKind::NonCanonical, error.kind());
            assert_eq!(error.consumed(), Some(consumed));
        }
        (Err(expected), Err(actual)) => {
            assert_ne!(Leb128DecodeErrorKind::NonCanonical, actual.kind());
            assert_eq!(expected, actual);
        }
        (Ok(_), Err(error)) => {
            panic!("non-strict decoding rejected strict-valid input: {error}")
        }
    }
}

/// Verifies success boundaries and detailed decoding error invariants.
fn assert_decode_result_metadata<T>(
    input: &[u8],
    max_bytes: usize,
    result: &Result<(T, NonZeroUsize), Leb128DecodeError>,
    strict: bool,
) {
    match result {
        Ok((_, consumed)) => {
            let consumed = consumed.get();
            assert!(consumed <= input.len().min(max_bytes));
            assert_eq!(0, input[consumed - 1] & 0x80);
        }
        Err(error) => {
            assert_eq!(0, error.start_index());
            match error.kind() {
                Leb128DecodeErrorKind::Incomplete => {
                    assert!(input.len() < max_bytes);
                    assert!(input.iter().all(|byte| byte & 0x80 != 0));
                    assert_eq!(input.len(), error.error_index());
                    assert_eq!(None, error.consumed());
                    assert_eq!(Some(input.len()), error.available());
                    assert_eq!(Some(input.len() + 1), error.required().map(|n| n.get()));
                    assert_eq!(Some(1), error.additional().map(|n| n.get()));
                }
                Leb128DecodeErrorKind::Malformed | Leb128DecodeErrorKind::NonCanonical => {
                    let consumed = error
                        .consumed()
                        .expect("invalid input must report consumed bytes")
                        .get();
                    assert!(consumed <= input.len().min(max_bytes));
                    assert_eq!(consumed - 1, error.error_index());
                    assert_eq!(None, error.required());
                    assert_eq!(None, error.available());
                    assert_eq!(None, error.additional());
                    if error.is_noncanonical() {
                        assert!(strict);
                        assert_eq!(0, input[consumed - 1] & 0x80);
                    }
                }
            }
        }
    }
}

/// Differentially checks the permissive 64-bit decoders against `leb128`.
fn assert_reference_decoders(input: &[u8]) {
    let reference_unsigned = reference_unsigned(input);
    let unsigned = unsafe { Leb128Codec::<u64, NonStrict>::decode(input, 0) };
    assert_matches_reference(unsigned, reference_unsigned);

    let signed = unsafe { Leb128Codec::<i64, NonStrict>::decode(input, 0) };
    assert_matches_reference(signed, reference_signed(input));

    let zig_zag = unsafe { ZigZagCodec::<i64, NonStrict>::decode(input, 0) };
    let reference = reference_unsigned.map(|(encoded, consumed)| {
        let value = ((encoded >> 1) as i64) ^ (-((encoded & 1) as i64));
        (value, consumed)
    });
    assert_matches_reference(zig_zag, reference);
}

/// Compares one codec result with an independent permissive decoder result.
fn assert_matches_reference<T>(actual: Result<(T, NonZeroUsize), Leb128DecodeError>, expected: Option<(T, usize)>)
where
    T: Debug + PartialEq,
{
    let actual = actual.ok().map(|(value, consumed)| (value, consumed.get()));
    assert_eq!(expected, actual);
}

/// Decodes unsigned LEB128 with the independent reference implementation.
fn reference_unsigned(input: &[u8]) -> Option<(u64, usize)> {
    let mut remaining = input;
    leb128::read::unsigned(&mut remaining)
        .ok()
        .map(|value| (value, input.len() - remaining.len()))
}

/// Decodes signed LEB128 with the independent reference implementation.
fn reference_signed(input: &[u8]) -> Option<(i64, usize)> {
    let mut remaining = input;
    leb128::read::signed(&mut remaining)
        .ok()
        .map(|value| (value, input.len() - remaining.len()))
}

/// Verifies roundtrips for every LEB128 and ZigZag supported width.
fn assert_leb128_roundtrips(bits: u128) {
    assert_leb128_roundtrip!(Leb128Codec, u8, bits as u8);
    assert_leb128_roundtrip!(Leb128Codec, u16, bits as u16);
    assert_leb128_roundtrip!(Leb128Codec, u32, bits as u32);
    assert_leb128_roundtrip!(Leb128Codec, u64, bits as u64);
    assert_leb128_roundtrip!(Leb128Codec, u128, bits);
    assert_leb128_roundtrip!(Leb128Codec, usize, bits as usize);
    assert_leb128_roundtrip!(Leb128Codec, i8, bits as i8);
    assert_leb128_roundtrip!(Leb128Codec, i16, bits as i16);
    assert_leb128_roundtrip!(Leb128Codec, i32, bits as i32);
    assert_leb128_roundtrip!(Leb128Codec, i64, bits as i64);
    assert_leb128_roundtrip!(Leb128Codec, i128, bits as i128);
    assert_leb128_roundtrip!(Leb128Codec, isize, bits as isize);
    assert_leb128_roundtrip!(ZigZagCodec, i8, bits as i8);
    assert_leb128_roundtrip!(ZigZagCodec, i16, bits as i16);
    assert_leb128_roundtrip!(ZigZagCodec, i32, bits as i32);
    assert_leb128_roundtrip!(ZigZagCodec, i64, bits as i64);
    assert_leb128_roundtrip!(ZigZagCodec, i128, bits as i128);
    assert_leb128_roundtrip!(ZigZagCodec, isize, bits as isize);
}

/// Checks every fixed-width wire type and byte order against standard-library
/// bytes.
fn assert_binary_roundtrips(bits: u128) {
    macro_rules! assert_integer_orders {
        ($ty:ty, $value:expr) => {{
            assert_binary_roundtrip!($ty, BigEndian, to_be_bytes, $value);
            assert_binary_roundtrip!($ty, LittleEndian, to_le_bytes, $value);
            assert_binary_roundtrip!($ty, NativeEndian, to_ne_bytes, $value);
        }};
    }
    macro_rules! assert_float_orders {
        ($ty:ty, $value:expr) => {{
            assert_binary_float_roundtrip!($ty, BigEndian, to_be_bytes, $value);
            assert_binary_float_roundtrip!($ty, LittleEndian, to_le_bytes, $value);
            assert_binary_float_roundtrip!($ty, NativeEndian, to_ne_bytes, $value);
        }};
    }

    assert_integer_orders!(u8, bits as u8);
    assert_integer_orders!(i8, bits as i8);
    assert_integer_orders!(u16, bits as u16);
    assert_integer_orders!(i16, bits as i16);
    assert_integer_orders!(u32, bits as u32);
    assert_integer_orders!(i32, bits as i32);
    assert_integer_orders!(u64, bits as u64);
    assert_integer_orders!(i64, bits as i64);
    assert_integer_orders!(u128, bits);
    assert_integer_orders!(i128, bits as i128);
    assert_float_orders!(f32, f32::from_bits(bits as u32));
    assert_float_orders!(f64, f64::from_bits(bits as u64));
}
