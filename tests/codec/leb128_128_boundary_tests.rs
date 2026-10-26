// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use core::any::type_name;
use core::fmt::Debug;
use core::num::NonZeroUsize;

use qubit_codec::Codec;
use qubit_codec::DecodeFailure;
use qubit_codec_binary::Leb128Codec;
use qubit_codec_binary::Leb128DecodeError;
use qubit_codec_binary::NonStrict;
use qubit_codec_binary::Strict;
use qubit_codec_binary::ZigZagCodec;

type DecodeResult<T> = Result<(T, NonZeroUsize), Leb128DecodeError>;

fn count(value: usize) -> NonZeroUsize {
    NonZeroUsize::new(value).expect("fixture counts are non-zero")
}

/// Constructs eighteen fixed continuation bytes and one terminal payload.
fn payload(prefix: u8, terminal: u8) -> [u8; 19] {
    assert_ne!(0, prefix & 0x80);
    assert_eq!(0, terminal & 0x80);
    let mut bytes = [prefix; 19];
    bytes[18] = terminal;
    bytes
}

/// Checks an independently expected result through inherent and trait APIs.
fn check<C>(
    decode: unsafe fn(&[u8], usize) -> DecodeResult<C::Value>,
    bytes: &[u8],
    expected: Result<C::Value, &str>,
) where
    C: Codec<Unit = u8, DecodeError = Leb128DecodeError> + Default,
    C::Value: Copy + Debug + PartialEq,
{
    assert!(!bytes.is_empty());
    for offset in [0, 3] {
        let mut input = vec![0xa5; offset];
        input.extend_from_slice(bytes);
        assert!(offset < input.len());
        let expected = match expected {
            Ok(value) => Ok((value, count(bytes.len()))),
            Err("noncanonical") => Err(Leb128DecodeError::noncanonical(offset, count(bytes.len()))),
            Err("malformed") => Err(Leb128DecodeError::malformed(
                offset,
                offset + bytes.len() - 1,
                count(bytes.len()),
            )),
            Err("incomplete") => Err(Leb128DecodeError::incomplete(
                offset,
                count(bytes.len() + 1),
                bytes.len(),
            )),
            Err(other) => panic!("unknown fixture category: {other}"),
        };
        let label = format!("{} offset={offset} bytes={bytes:02x?}", type_name::<C>());
        // SAFETY: Each fixture is non-empty from the valid offset.
        let actual = unsafe { decode(&input, offset) };
        assert_eq!(expected, actual, "{label}: inherent");

        let expected_trait = expected.map_err(|error| {
            if let Some(required) = error.required() {
                DecodeFailure::incomplete_with_source(error, required)
            } else {
                DecodeFailure::invalid(
                    error,
                    error.consumed().expect("invalid fixture consumes bytes"),
                )
            }
        });
        // SAFETY: All tested codecs require one readable byte at offset.
        let actual = unsafe { Codec::decode(&mut C::default(), &input, offset) };
        assert_eq!(expected_trait, actual, "{label}: trait");
    }
}

macro_rules! check_both {
    ($codec:ident, $ty:ty, $bytes:expr, $strict:expr, $non_strict:expr) => {{
        let bytes = $bytes;
        check::<$codec<$ty, Strict>>($codec::<$ty, Strict>::decode, bytes, $strict);
        check::<$codec<$ty, NonStrict>>($codec::<$ty, NonStrict>::decode, bytes, $non_strict);
    }};
}

#[test]
fn test_128_boundary_unsigned_golden_vectors() {
    check_both!(Leb128Codec, u128, &payload(0xff, 0x03), Ok(u128::MAX), Ok(u128::MAX));
    check_both!(Leb128Codec, u128, &payload(0x80, 0x01), Ok(1_u128 << 126), Ok(1_u128 << 126));
    check_both!(Leb128Codec, u128, &payload(0x80, 0x00), Err("noncanonical"), Ok(0));
}

#[test]
fn test_128_boundary_signed_golden_vectors() {
    check_both!(Leb128Codec, i128, &payload(0xff, 0x01), Ok(i128::MAX), Ok(i128::MAX));
    check_both!(Leb128Codec, i128, &payload(0x80, 0x7e), Ok(i128::MIN), Ok(i128::MIN));
    check_both!(Leb128Codec, i128, &payload(0x80, 0x01), Ok(1_i128 << 126), Ok(1_i128 << 126));
    check_both!(Leb128Codec, i128, &payload(0x80, 0x7f), Ok(-(1_i128 << 126)), Ok(-(1_i128 << 126)));
    check_both!(Leb128Codec, i128, &payload(0x80, 0x00), Err("noncanonical"), Ok(0));
    check_both!(Leb128Codec, i128, &payload(0xff, 0x7f), Err("noncanonical"), Ok(-1));
}

#[test]
fn test_128_boundary_zig_zag_golden_vectors() {
    check_both!(ZigZagCodec, i128, &payload(0xff, 0x03), Ok(i128::MIN), Ok(i128::MIN));
    let mut maximum = payload(0xff, 0x03);
    maximum[0] = 0xfe;
    check_both!(ZigZagCodec, i128, &maximum, Ok(i128::MAX), Ok(i128::MAX));
}

#[test]
fn test_128_boundary_rejects_invalid_terminal_payloads() {
    for terminal in 0_u8..=0x7f {
        let bytes = payload(0x80, terminal);
        if terminal > 0x03 {
            check_both!(Leb128Codec, u128, &bytes, Err("malformed"), Err("malformed"));
            check_both!(ZigZagCodec, i128, &bytes, Err("malformed"), Err("malformed"));
        }
        if ![0x00, 0x01, 0x7e, 0x7f].contains(&terminal) {
            check_both!(Leb128Codec, i128, &bytes, Err("malformed"), Err("malformed"));
        }
    }
}

#[test]
fn test_128_boundary_continuation_limits() {
    check_both!(Leb128Codec, u128, &[0x80; 18], Err("incomplete"), Err("incomplete"));
    check_both!(Leb128Codec, i128, &[0x80; 18], Err("incomplete"), Err("incomplete"));
    check_both!(ZigZagCodec, i128, &[0x80; 18], Err("incomplete"), Err("incomplete"));
    check_both!(Leb128Codec, u128, &[0x80; 19], Err("malformed"), Err("malformed"));
    check_both!(Leb128Codec, i128, &[0x80; 19], Err("malformed"), Err("malformed"));
    check_both!(ZigZagCodec, i128, &[0x80; 19], Err("malformed"), Err("malformed"));
}
