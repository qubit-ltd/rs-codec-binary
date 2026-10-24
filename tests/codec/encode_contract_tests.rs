// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use core::convert::Infallible;
use core::fmt::Debug;

use qubit_codec::Codec;
use qubit_codec_binary::Leb128Codec;
use qubit_codec_binary::NonStrict;
use qubit_codec_binary::Strict;
use qubit_codec_binary::ZigZagCodec;

/// Checks length and exact output slices only after maximum-capacity encoding.
fn assert_encode_contract<C>(value: C::Value, direct_encode: unsafe fn(C::Value, &mut [u8], usize) -> usize)
where
    C: Codec<Unit = u8, EncodeError = Infallible> + Default,
    C::Value: Copy + Debug,
{
    let mut codec = C::default();
    let required = codec.encode_len(&value);
    let maximum = C::MAX_ENCODE_UNITS_PER_VALUE;
    assert!((1..=maximum).contains(&required), "invalid length for {value:?}");
    let mut reference = vec![0; maximum];
    // SAFETY: The buffer exposes the codec's maximum encode capacity.
    let written = unsafe { direct_encode(value, &mut reference, 0) };
    assert_eq!(required, written, "length mismatch for {value:?}");
    let mut trait_output = vec![0; maximum];
    // SAFETY: These scalar codecs accept every value and have maximum capacity.
    let trait_written = unsafe { Codec::encode(&mut codec, &value, &mut trait_output, 0) }.unwrap();
    assert_eq!(written, trait_written);
    assert_eq!(&reference[..written], &trait_output[..trait_written]);

    let mut direct_storage = vec![0xa5; required + 2];
    // SAFETY: Maximum-capacity encoding above established the exact width.
    let direct_written = unsafe { direct_encode(value, &mut direct_storage[1..1 + required], 0) };
    assert_eq!(required, direct_written);
    assert_eq!(&reference[..written], &direct_storage[1..1 + required]);
    assert_eq!(0xa5, direct_storage[0]);
    assert_eq!(0xa5, direct_storage[required + 1]);

    let mut trait_storage = vec![0xa5; required + 2];
    // SAFETY: The unchanged stateless codec and value require exactly this width.
    let exact_written = unsafe { Codec::encode(&mut codec, &value, &mut trait_storage[..1 + required], 1) }.unwrap();
    assert_eq!(required, exact_written);
    assert_eq!(&reference[..written], &trait_storage[1..1 + required]);
    assert_eq!(0xa5, trait_storage[0]);
    assert_eq!(0xa5, trait_storage[required + 1]);
}

macro_rules! encode_contract_test {
    ($name:ident, $codec:ident, $ty:ty) => {
        #[test]
        fn $name() {
            let mut values = vec![0, 1, <$ty>::MIN, <$ty>::MAX];
            for bit in 0..<$ty>::BITS {
                let power = (1_u128 << bit) as $ty;
                values.extend([
                    power,
                    power.wrapping_sub(1),
                    power.wrapping_add(1),
                    power.wrapping_neg(),
                    !power,
                ]);
            }
            for value in values {
                assert_encode_contract::<$codec<$ty, NonStrict>>(value, $codec::<$ty, NonStrict>::encode);
                assert_encode_contract::<$codec<$ty, Strict>>(value, $codec::<$ty, Strict>::encode);
            }
        }
    };
}

encode_contract_test!(test_uleb_u8_contract, Leb128Codec, u8);
encode_contract_test!(test_uleb_u16_contract, Leb128Codec, u16);
encode_contract_test!(test_uleb_u32_contract, Leb128Codec, u32);
encode_contract_test!(test_uleb_u64_contract, Leb128Codec, u64);
encode_contract_test!(test_uleb_u128_contract, Leb128Codec, u128);
encode_contract_test!(test_uleb_usize_contract, Leb128Codec, usize);
encode_contract_test!(test_sleb_i8_contract, Leb128Codec, i8);
encode_contract_test!(test_sleb_i16_contract, Leb128Codec, i16);
encode_contract_test!(test_sleb_i32_contract, Leb128Codec, i32);
encode_contract_test!(test_sleb_i64_contract, Leb128Codec, i64);
encode_contract_test!(test_sleb_i128_contract, Leb128Codec, i128);
encode_contract_test!(test_sleb_isize_contract, Leb128Codec, isize);
encode_contract_test!(test_zigzag_i8_contract, ZigZagCodec, i8);
encode_contract_test!(test_zigzag_i16_contract, ZigZagCodec, i16);
encode_contract_test!(test_zigzag_i32_contract, ZigZagCodec, i32);
encode_contract_test!(test_zigzag_i64_contract, ZigZagCodec, i64);
encode_contract_test!(test_zigzag_i128_contract, ZigZagCodec, i128);
encode_contract_test!(test_zigzag_isize_contract, ZigZagCodec, isize);
