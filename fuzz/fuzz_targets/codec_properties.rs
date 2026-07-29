// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![no_main]

use libfuzzer_sys::fuzz_target;
use qubit_codec_binary::{
    Leb128Codec,
    Leb128DecodeErrorKind,
    NonStrict,
    Strict,
    ZigZagCodec,
};

/// Bounds each invocation independently of the fuzzer configuration.
const MAX_FUZZ_INPUT_LEN: usize = 19;

fuzz_target!(|data: &[u8]| {
    let input = &data[..data.len().min(MAX_FUZZ_INPUT_LEN)];

    decode_arbitrary_input(input);
    assert_noncanonical_policy_behavior();

    let unsigned = fuzz_u64(input);
    assert_unsigned_leb128_roundtrip(unsigned);
    assert_signed_leb128_roundtrip(unsigned as i64);
    assert_zig_zag_roundtrip(unsigned as i64);
});

/// Builds a deterministic integer from at most eight fuzz input bytes.
fn fuzz_u64(input: &[u8]) -> u64 {
    let mut bytes = [0u8; size_of::<u64>()];
    for (output, source) in bytes.iter_mut().zip(input.iter().copied()) {
        *output = source;
    }
    u64::from_le_bytes(bytes)
}

/// Exercises both policies on arbitrary non-empty input without violating the
/// unchecked decoder precondition.
fn decode_arbitrary_input(input: &[u8]) {
    if input.is_empty() {
        return;
    }

    let unsigned_non_strict =
        unsafe { Leb128Codec::<u64, NonStrict>::decode(input, 0) };
    let unsigned_strict =
        unsafe { Leb128Codec::<u64, Strict>::decode(input, 0) };
    assert_strict_success_is_non_strict_success(
        unsigned_strict,
        unsigned_non_strict,
    );

    let signed_non_strict =
        unsafe { Leb128Codec::<i64, NonStrict>::decode(input, 0) };
    let signed_strict = unsafe { Leb128Codec::<i64, Strict>::decode(input, 0) };
    assert_strict_success_is_non_strict_success(
        signed_strict,
        signed_non_strict,
    );

    let zig_zag_non_strict =
        unsafe { ZigZagCodec::<i64, NonStrict>::decode(input, 0) };
    let zig_zag_strict =
        unsafe { ZigZagCodec::<i64, Strict>::decode(input, 0) };
    assert_strict_success_is_non_strict_success(
        zig_zag_strict,
        zig_zag_non_strict,
    );
}

/// Verifies that strict acceptance implies the same non-strict value and
/// consumed byte count.
fn assert_strict_success_is_non_strict_success<T, E>(
    strict: Result<(T, core::num::NonZeroUsize), E>,
    non_strict: Result<(T, core::num::NonZeroUsize), E>,
) where
    T: core::fmt::Debug + PartialEq,
    E: core::fmt::Debug,
{
    if let Ok((expected_value, expected_consumed)) = strict {
        let (actual_value, actual_consumed) = non_strict
            .expect("non-strict decoding must accept strict-valid input");
        assert_eq!(expected_value, actual_value);
        assert_eq!(expected_consumed, actual_consumed);
    }
}

/// Verifies canonical unsigned LEB128 encode/decode roundtrips.
fn assert_unsigned_leb128_roundtrip(value: u64) {
    let mut output = [0u8; Leb128Codec::<u64, NonStrict>::MAX_UNITS_PER_VALUE];
    let written =
        unsafe { Leb128Codec::<u64, NonStrict>::encode(value, &mut output, 0) };
    let strict =
        unsafe { Leb128Codec::<u64, Strict>::decode(&output[..written], 0) }
            .expect(
                "canonical unsigned LEB128 encoding must pass strict decoding",
            );
    let non_strict = unsafe {
        Leb128Codec::<u64, NonStrict>::decode(&output[..written], 0)
    }
    .expect("canonical unsigned LEB128 encoding must pass non-strict decoding");

    let (strict_value, strict_consumed) = strict;
    let (non_strict_value, non_strict_consumed) = non_strict;
    assert_eq!((value, written), (strict_value, strict_consumed.get()));
    assert_eq!(
        (value, written),
        (non_strict_value, non_strict_consumed.get())
    );
}

/// Verifies canonical signed LEB128 encode/decode roundtrips.
fn assert_signed_leb128_roundtrip(value: i64) {
    let mut output = [0u8; Leb128Codec::<i64, NonStrict>::MAX_UNITS_PER_VALUE];
    let written =
        unsafe { Leb128Codec::<i64, NonStrict>::encode(value, &mut output, 0) };
    let strict =
        unsafe { Leb128Codec::<i64, Strict>::decode(&output[..written], 0) }
            .expect(
                "canonical signed LEB128 encoding must pass strict decoding",
            );
    let non_strict = unsafe {
        Leb128Codec::<i64, NonStrict>::decode(&output[..written], 0)
    }
    .expect("canonical signed LEB128 encoding must pass non-strict decoding");

    let (strict_value, strict_consumed) = strict;
    let (non_strict_value, non_strict_consumed) = non_strict;
    assert_eq!((value, written), (strict_value, strict_consumed.get()));
    assert_eq!(
        (value, written),
        (non_strict_value, non_strict_consumed.get())
    );
}

/// Verifies canonical ZigZag encode/decode roundtrips.
fn assert_zig_zag_roundtrip(value: i64) {
    let mut output = [0u8; ZigZagCodec::<i64, NonStrict>::MAX_UNITS_PER_VALUE];
    let written =
        unsafe { ZigZagCodec::<i64, NonStrict>::encode(value, &mut output, 0) };
    let strict =
        unsafe { ZigZagCodec::<i64, Strict>::decode(&output[..written], 0) }
            .expect("canonical ZigZag encoding must pass strict decoding");
    let non_strict =
        unsafe { ZigZagCodec::<i64, NonStrict>::decode(&output[..written], 0) }
            .expect("canonical ZigZag encoding must pass non-strict decoding");

    let (strict_value, strict_consumed) = strict;
    let (non_strict_value, non_strict_consumed) = non_strict;
    assert_eq!((value, written), (strict_value, strict_consumed.get()));
    assert_eq!(
        (value, written),
        (non_strict_value, non_strict_consumed.get())
    );
}

/// Verifies that only strict decoding rejects redundant valid encodings.
fn assert_noncanonical_policy_behavior() {
    let unsigned_non_strict =
        unsafe { Leb128Codec::<u64, NonStrict>::decode(&[0x80, 0x00], 0) }
            .expect("non-strict unsigned decoding must accept redundant zero");
    let (unsigned_value, unsigned_consumed) = unsigned_non_strict;
    assert_eq!((0, 2), (unsigned_value, unsigned_consumed.get()));
    let unsigned_strict =
        unsafe { Leb128Codec::<u64, Strict>::decode(&[0x80, 0x00], 0) }
            .expect_err("strict unsigned decoding must reject redundant zero");
    assert_eq!(Leb128DecodeErrorKind::NonCanonical, unsigned_strict.kind());

    let signed_non_strict =
        unsafe { Leb128Codec::<i64, NonStrict>::decode(&[0xff, 0x7f], 0) }
            .expect(
                "non-strict signed decoding must accept redundant negative one",
            );
    let (signed_value, signed_consumed) = signed_non_strict;
    assert_eq!((-1, 2), (signed_value, signed_consumed.get()));
    let signed_strict =
        unsafe { Leb128Codec::<i64, Strict>::decode(&[0xff, 0x7f], 0) }
            .expect_err(
                "strict signed decoding must reject redundant negative one",
            );
    assert_eq!(Leb128DecodeErrorKind::NonCanonical, signed_strict.kind());

    let zig_zag_non_strict =
        unsafe { ZigZagCodec::<i64, NonStrict>::decode(&[0x80, 0x00], 0) }
            .expect("non-strict ZigZag decoding must accept redundant zero");
    let (zig_zag_value, zig_zag_consumed) = zig_zag_non_strict;
    assert_eq!((0, 2), (zig_zag_value, zig_zag_consumed.get()));
    let zig_zag_strict =
        unsafe { ZigZagCodec::<i64, Strict>::decode(&[0x80, 0x00], 0) }
            .expect_err("strict ZigZag decoding must reject redundant zero");
    assert_eq!(Leb128DecodeErrorKind::NonCanonical, zig_zag_strict.kind());
}
