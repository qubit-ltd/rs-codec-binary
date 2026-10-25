// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

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

fn nonzero(value: usize) -> NonZeroUsize {
    NonZeroUsize::new(value).expect("fixture length must be positive")
}

/// Checks explicit expected results through both public decode entry points.
fn assert_decode_offset_cases<C>(
    decode: unsafe fn(&[u8], usize) -> DecodeResult<C::Value>,
    expected_one: C::Value,
    strict: bool,
) where
    C: Codec<Unit = u8, DecodeError = Leb128DecodeError> + Default,
    C::Value: Copy + Debug + Default + PartialEq,
{
    let malformed = vec![0x80; C::MAX_DECODE_UNITS_PER_VALUE];
    let cases: [(&str, &[u8]); 4] = [
        ("canonical_with_trailing", &[0x01, 0x7f]),
        ("incomplete", &[0x80]),
        ("malformed", &malformed),
        ("noncanonical_zero", &[0x80, 0x00]),
    ];
    for offset in [0, 1, 3, 7] {
        for (case, payload) in cases {
            let expected = match case {
                "canonical_with_trailing" => Ok((expected_one, nonzero(1))),
                "incomplete" => Err(Leb128DecodeError::incomplete(offset, nonzero(2), 1)),
                "malformed" => Err(Leb128DecodeError::malformed(
                    offset,
                    offset + malformed.len() - 1,
                    nonzero(malformed.len()),
                )),
                "noncanonical_zero" if strict => Err(Leb128DecodeError::noncanonical(offset, nonzero(2))),
                "noncanonical_zero" => Ok((C::Value::default(), nonzero(2))),
                _ => unreachable!("unknown fixture"),
            };
            let mut input: Vec<u8> = [0x00, 0x80, 0xff].into_iter().cycle().take(offset).collect();
            input.extend_from_slice(payload);
            let label = format!("{} {case} offset={offset}", core::any::type_name::<C>());
            // SAFETY: Every fixture is non-empty at the checked start index.
            assert!(offset < input.len());
            let actual = unsafe { decode(&input, offset) };
            assert_eq!(expected, actual, "{label}: inherent decode");

            let expected_trait = expected.map_err(|source| {
                if let Some(required) = source.required() {
                    DecodeFailure::incomplete_with_source(source, required)
                } else {
                    DecodeFailure::invalid(source, source.consumed().expect("invalid fixture must consume bytes"))
                }
            });
            // SAFETY: Every binary codec tested here requires one readable byte.
            let actual_trait = unsafe { Codec::decode(&mut C::default(), &input, offset) };
            assert_eq!(expected_trait, actual_trait, "{label}: trait decode");
        }
    }
}

#[test]
fn test_decode_offset_unsigned_leb128() {
    macro_rules! check {
        ($ty:ty) => {
            assert_decode_offset_cases::<Leb128Codec<$ty, NonStrict>>(Leb128Codec::<$ty, NonStrict>::decode, 1, false);
            assert_decode_offset_cases::<Leb128Codec<$ty, Strict>>(Leb128Codec::<$ty, Strict>::decode, 1, true);
        };
    }
    check!(u8);
    check!(u16);
    check!(u32);
    check!(u64);
    check!(u128);
    check!(usize);
}

#[test]
fn test_decode_offset_signed_leb128() {
    macro_rules! check {
        ($ty:ty) => {
            assert_decode_offset_cases::<Leb128Codec<$ty, NonStrict>>(Leb128Codec::<$ty, NonStrict>::decode, 1, false);
            assert_decode_offset_cases::<Leb128Codec<$ty, Strict>>(Leb128Codec::<$ty, Strict>::decode, 1, true);
        };
    }
    check!(i8);
    check!(i16);
    check!(i32);
    check!(i64);
    check!(i128);
    check!(isize);
}

#[test]
fn test_decode_offset_zig_zag() {
    macro_rules! check {
        ($ty:ty) => {
            assert_decode_offset_cases::<ZigZagCodec<$ty, NonStrict>>(ZigZagCodec::<$ty, NonStrict>::decode, -1, false);
            assert_decode_offset_cases::<ZigZagCodec<$ty, Strict>>(ZigZagCodec::<$ty, Strict>::decode, -1, true);
        };
    }
    check!(i8);
    check!(i16);
    check!(i32);
    check!(i64);
    check!(i128);
    check!(isize);
}
