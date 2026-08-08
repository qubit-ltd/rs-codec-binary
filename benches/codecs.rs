// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Direct throughput benchmarks for the binary codec hot paths.
//!
//! The mixed-width groups mirror the production field schemas used by
//! `rs-io-binary`. Their `unchecked` variants pass one whole buffer with a
//! runtime offset, while `safe_slices` recreates an exact checked slice for
//! every field before calling the same codec helper.

use std::convert::Infallible;
use std::fmt::Debug;
use std::hint::black_box;

use criterion::BatchSize;
use criterion::BenchmarkId;
use criterion::Criterion;
use criterion::Throughput;
use criterion::criterion_group;
use criterion::criterion_main;
use qubit_codec::BigEndian;
use qubit_codec::Codec;
use qubit_codec_binary::BinaryCodec;
use qubit_codec_binary::Leb128Codec;
use qubit_codec_binary::Leb128DecodeError;
use qubit_codec_binary::Leb128DecodeErrorKind;
use qubit_codec_binary::Leb128DecodePolicy;
use qubit_codec_binary::NonStrict;
use qubit_codec_binary::Strict;
use qubit_codec_binary::ZigZagCodec;

/// Number of values processed by each benchmark iteration.
const BATCH_SIZE: usize = 1_024;

/// Number of fields in each mixed-width benchmark fixture.
const MIXED_FIELD_COUNT: usize = 16_384;

/// Maximum encoded width of each 64-bit varint family benchmarked here.
const MAX_VARINT_BYTES: usize = 10;

/// Maximum encoded width among the mixed unsigned LEB128 field types.
const MAX_MIXED_ULEB_BYTES: usize = 19;

/// Sentinel used to validate exact-capacity writes before timing them.
const GUARD_BYTE: u8 = 0xa5;

/// Multiplier used by the deterministic fixture generator.
const LCG_MULTIPLIER: u64 = 6_364_136_223_846_793_005;

/// Increment used by the deterministic fixture generator.
const LCG_INCREMENT: u64 = 1;

/// Value distributions that expose different varint workloads.
#[derive(Clone, Copy)]
enum Distribution {
    Boundary,
    Short,
    Uniform,
    MaxWidth,
}

impl Distribution {
    /// Returns the stable Criterion parameter name for this distribution.
    const fn name(self) -> &'static str {
        match self {
            Self::Boundary => "boundary",
            Self::Short => "short",
            Self::Uniform => "uniform",
            Self::MaxWidth => "max_width",
        }
    }
}

/// All value distributions used by each canonical throughput operation.
const DISTRIBUTIONS: [Distribution; 4] = [
    Distribution::Boundary,
    Distribution::Short,
    Distribution::Uniform,
    Distribution::MaxWidth,
];

/// A canonical unsigned LEB128 payload.
#[derive(Clone, Copy)]
struct UnsignedPayload {
    bytes: [u8; MAX_VARINT_BYTES],
    len: usize,
}

/// A canonical signed LEB128 payload.
#[derive(Clone, Copy)]
struct SignedLeb128Payload {
    bytes: [u8; MAX_VARINT_BYTES],
    len: usize,
}

/// A canonical ZigZag LEB128 payload.
#[derive(Clone, Copy)]
struct ZigZagPayload {
    bytes: [u8; MAX_VARINT_BYTES],
    len: usize,
}

/// Advances one deterministic pseudo-random fixture state.
#[inline(always)]
fn next_state(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(LCG_MULTIPLIER)
        .wrapping_add(LCG_INCREMENT);
    *state
}

/// Repeats a non-empty boundary pattern to fill one benchmark batch.
fn repeat_pattern<T>(pattern: &[T]) -> Vec<T>
where
    T: Copy,
{
    assert!(!pattern.is_empty());
    pattern.iter().copied().cycle().take(BATCH_SIZE).collect()
}

/// Converts an unsigned ZigZag payload back to its signed value.
#[inline(always)]
fn decode_zig_zag_value(encoded: u64) -> i64 {
    ((encoded >> 1) as i64) ^ (-((encoded & 1) as i64))
}

/// Builds unsigned values for one benchmark distribution.
fn unsigned_values(distribution: Distribution) -> Vec<u64> {
    match distribution {
        Distribution::Boundary => {
            let mut pattern = vec![0, 1, 0x7e, 0x7f, 0x80, 0x81];
            for shift in (7_u32..u64::BITS).step_by(7) {
                let boundary = 1_u64 << shift;
                pattern.extend([
                    boundary - 1,
                    boundary,
                    boundary.saturating_add(1),
                ]);
            }
            pattern.extend([u64::MAX - 1, u64::MAX]);
            repeat_pattern(&pattern)
        }
        Distribution::Short => {
            let mut state = 0xD1CE_BA5E_1234_5678;
            (0..BATCH_SIZE)
                .map(|_| next_state(&mut state) & 0x3fff)
                .collect()
        }
        Distribution::Uniform => {
            let mut state = 0xD1CE_BA5E_1234_5678;
            (0..BATCH_SIZE).map(|_| next_state(&mut state)).collect()
        }
        Distribution::MaxWidth => {
            let mut state = 0xD1CE_BA5E_1234_5678;
            (0..BATCH_SIZE)
                .map(|_| next_state(&mut state) | (1_u64 << 63))
                .collect()
        }
    }
}

/// Builds signed LEB128 values for one benchmark distribution.
fn signed_leb128_values(distribution: Distribution) -> Vec<i64> {
    match distribution {
        Distribution::Boundary => {
            let mut pattern = vec![0, 1, -1, 63, 64, -64, -65];
            for shift in (6_u32..i64::BITS).step_by(7) {
                let boundary = 1_i64 << shift;
                pattern.extend([
                    boundary - 1,
                    boundary,
                    boundary + 1,
                    -boundary + 1,
                    -boundary,
                    -boundary - 1,
                ]);
            }
            pattern.extend([i64::MIN, i64::MAX]);
            repeat_pattern(&pattern)
        }
        Distribution::Short => {
            let mut state = 0xA11C_E5E1_5EED_1234;
            (0..BATCH_SIZE)
                .map(|_| (next_state(&mut state) & 0x3fff) as i64 - 8_192)
                .collect()
        }
        Distribution::Uniform => {
            let mut state = 0xA11C_E5E1_5EED_1234;
            (0..BATCH_SIZE)
                .map(|_| next_state(&mut state) as i64)
                .collect()
        }
        Distribution::MaxWidth => {
            let mut state = 0xA11C_E5E1_5EED_1234;
            (0..BATCH_SIZE)
                .map(|index| {
                    let magnitude = (next_state(&mut state)
                        & ((1_u64 << 62) - 1))
                        | (1_u64 << 62);
                    let positive = magnitude as i64;
                    if index & 1 == 0 { positive } else { !positive }
                })
                .collect()
        }
    }
}

/// Builds ZigZag values for one benchmark distribution.
fn zig_zag_values(distribution: Distribution) -> Vec<i64> {
    match distribution {
        Distribution::Boundary => {
            let mut encoded = vec![0, 1, 0x7e, 0x7f, 0x80, 0x81];
            for shift in (7_u32..u64::BITS).step_by(7) {
                let boundary = 1_u64 << shift;
                encoded.extend([
                    boundary - 1,
                    boundary,
                    boundary.saturating_add(1),
                ]);
            }
            encoded.extend([u64::MAX - 1, u64::MAX]);
            let pattern = encoded
                .into_iter()
                .map(decode_zig_zag_value)
                .collect::<Vec<_>>();
            repeat_pattern(&pattern)
        }
        Distribution::Short => {
            let mut state = 0x21A2_A612_5EED_5678;
            (0..BATCH_SIZE)
                .map(|_| decode_zig_zag_value(next_state(&mut state) & 0x3fff))
                .collect()
        }
        Distribution::Uniform => {
            let mut state = 0x21A2_A612_5EED_5678;
            (0..BATCH_SIZE)
                .map(|_| next_state(&mut state) as i64)
                .collect()
        }
        Distribution::MaxWidth => {
            let mut state = 0x21A2_A612_5EED_5678;
            (0..BATCH_SIZE)
                .map(|_| {
                    decode_zig_zag_value(next_state(&mut state) | (1_u64 << 63))
                })
                .collect()
        }
    }
}

/// Encodes the fixtures used by unsigned LEB128 decode benchmarks.
fn unsigned_payloads(values: &[u64]) -> Vec<UnsignedPayload> {
    values
        .iter()
        .copied()
        .map(|value| {
            let mut bytes = [0_u8; MAX_VARINT_BYTES];
            let len = unsafe {
                Leb128Codec::<u64, NonStrict>::encode(value, &mut bytes, 0)
            };
            UnsignedPayload { bytes, len }
        })
        .collect()
}

/// Encodes the fixtures used by signed LEB128 decode benchmarks.
fn signed_leb128_payloads(values: &[i64]) -> Vec<SignedLeb128Payload> {
    values
        .iter()
        .copied()
        .map(|value| {
            let mut bytes = [0_u8; MAX_VARINT_BYTES];
            let len = unsafe {
                Leb128Codec::<i64, NonStrict>::encode(value, &mut bytes, 0)
            };
            SignedLeb128Payload { bytes, len }
        })
        .collect()
}

/// Encodes the fixtures used by ZigZag decode benchmarks.
fn zig_zag_payloads(values: &[i64]) -> Vec<ZigZagPayload> {
    values
        .iter()
        .copied()
        .map(|value| {
            let mut bytes = [0_u8; MAX_VARINT_BYTES];
            let len = unsafe {
                ZigZagCodec::<i64, NonStrict>::encode(value, &mut bytes, 0)
            };
            ZigZagPayload { bytes, len }
        })
        .collect()
}

/// Validates that a named fixture distribution has the promised widths.
fn validate_distribution_widths(
    distribution: Distribution,
    widths: impl IntoIterator<Item = usize>,
) {
    match distribution {
        Distribution::Boundary => {
            let mut seen = [false; MAX_VARINT_BYTES + 1];
            for width in widths {
                assert!((1..=MAX_VARINT_BYTES).contains(&width));
                seen[width] = true;
            }
            assert!(seen[1..].iter().all(|present| *present));
        }
        Distribution::Short => {
            let mut seen = [false; 3];
            for width in widths {
                assert!((1..=2).contains(&width));
                seen[width] = true;
            }
            assert!(seen[1] && seen[2]);
        }
        Distribution::Uniform => {}
        Distribution::MaxWidth => {
            assert!(widths.into_iter().all(|width| width == MAX_VARINT_BYTES));
        }
    }
}

/// Mixes one observed value into a benchmark checksum.
#[inline(always)]
fn mix_checksum(checksum: u64, value: u64) -> u64 {
    (checksum.rotate_left(7) ^ value).wrapping_mul(0x9E37_79B1_85EB_CA87)
}

/// Observes every encoded byte so the optimizer must retain the full write.
#[inline(always)]
fn checksum_encoded(mut checksum: u64, bytes: &[u8], written: usize) -> u64 {
    checksum = mix_checksum(checksum, written as u64);
    for &byte in black_box(&bytes[..written]) {
        checksum = mix_checksum(checksum, u64::from(byte));
    }
    checksum
}

/// Measures exact encoded-length calculation through the generic trait.
fn encode_lengths<C>(values: &[C::Value]) -> u64
where
    C: Codec + Default,
{
    let codec = C::default();
    let mut checksum = 0_u64;
    for value in black_box(values) {
        let len = codec.encode_len(black_box(value));
        checksum = mix_checksum(checksum, len as u64);
    }
    checksum
}

/// Encodes through `Codec` with exactly the reported value width writable.
fn encode_exact_capacity<C>(values: &[C::Value]) -> u64
where
    C: Codec<Unit = u8, EncodeError = Infallible> + Default,
{
    let mut codec = C::default();
    let mut checksum = 0_u64;
    let mut storage = [GUARD_BYTE; MAX_VARINT_BYTES + 2];
    for value in black_box(values) {
        let required = codec.encode_len(black_box(value));
        let output = &mut storage[1..1 + required];
        let written = unsafe {
            // SAFETY: `output` exposes exactly the width reported for the same
            // value and unchanged codec state.
            Codec::encode(&mut codec, value, output, 0)
        }
        .unwrap_or_else(|error| match error {});
        checksum = checksum_encoded(checksum, output, written);
    }
    checksum
}

/// Validates every exact-capacity fixture and both adjacent guards.
fn validate_exact_capacity<C>(values: &[C::Value])
where
    C: Codec<Unit = u8> + Default,
    C::EncodeError: Debug,
{
    assert!(C::MAX_ENCODE_UNITS_PER_VALUE <= MAX_VARINT_BYTES);
    let mut codec = C::default();
    for value in values {
        assert!(codec.can_encode_value(value));
        let required = codec.encode_len(value);
        assert!(required <= C::MAX_ENCODE_UNITS_PER_VALUE);
        let mut storage = [GUARD_BYTE; MAX_VARINT_BYTES + 2];
        let written = unsafe {
            // SAFETY: The interior slice exposes exactly `required` bytes.
            Codec::encode(&mut codec, value, &mut storage[1..1 + required], 0)
        }
        .expect("benchmark fixture should encode");
        assert_eq!(required, written);
        assert_eq!(GUARD_BYTE, storage[0]);
        assert_eq!(GUARD_BYTE, storage[required + 1]);
    }
}

macro_rules! define_direct_encoder {
    ($function:ident, $codec:ty, $value:ty) => {
        /// Encodes a batch through the codec's direct inherent method.
        fn $function(values: &[$value]) -> u64 {
            let mut checksum = 0_u64;
            let mut bytes = [0_u8; MAX_VARINT_BYTES];
            for &value in black_box(values) {
                let written = unsafe {
                    <$codec>::encode(black_box(value), &mut bytes, 0)
                };
                checksum = checksum_encoded(checksum, &bytes, written);
            }
            checksum
        }
    };
}

define_direct_encoder!(
    encode_unsigned_direct,
    Leb128Codec<u64, NonStrict>,
    u64
);
define_direct_encoder!(
    encode_signed_leb128_direct,
    Leb128Codec<i64, NonStrict>,
    i64
);
define_direct_encoder!(
    encode_zig_zag_direct,
    ZigZagCodec<i64, NonStrict>,
    i64
);

/// Decodes every canonical unsigned LEB128 fixture under one policy.
fn decode_unsigned_payloads<P>(payloads: &[UnsignedPayload]) -> u64
where
    P: Leb128DecodePolicy,
{
    let mut checksum = 0_u64;
    for payload in black_box(payloads) {
        // SAFETY: `validate_distribution_widths` only supplies canonical
        // payloads produced by the corresponding encoder.
        let (value, consumed) = unsafe {
            Leb128Codec::<u64, P>::decode(&payload.bytes[..payload.len], 0)
                .unwrap_unchecked()
        };
        checksum = mix_checksum(checksum, value);
        checksum = mix_checksum(checksum, consumed.get() as u64);
    }
    checksum
}

/// Decodes every canonical signed LEB128 fixture under one policy.
fn decode_signed_leb128_payloads<P>(payloads: &[SignedLeb128Payload]) -> u64
where
    P: Leb128DecodePolicy,
{
    let mut checksum = 0_u64;
    for payload in black_box(payloads) {
        // SAFETY: `validate_distribution_widths` only supplies canonical
        // payloads produced by the corresponding encoder.
        let (value, consumed) = unsafe {
            Leb128Codec::<i64, P>::decode(&payload.bytes[..payload.len], 0)
                .unwrap_unchecked()
        };
        checksum = mix_checksum(checksum, value as u64);
        checksum = mix_checksum(checksum, consumed.get() as u64);
    }
    checksum
}

/// Decodes every canonical ZigZag fixture under one policy.
fn decode_zig_zag_payloads<P>(payloads: &[ZigZagPayload]) -> u64
where
    P: Leb128DecodePolicy,
{
    let mut checksum = 0_u64;
    for payload in black_box(payloads) {
        // SAFETY: `validate_distribution_widths` only supplies canonical
        // payloads produced by the corresponding encoder.
        let (value, consumed) = unsafe {
            ZigZagCodec::<i64, P>::decode(&payload.bytes[..payload.len], 0)
                .unwrap_unchecked()
        };
        checksum = mix_checksum(checksum, value as u64);
        checksum = mix_checksum(checksum, consumed.get() as u64);
    }
    checksum
}

/// Observes detailed error metadata for an expected decode failure.
#[inline(always)]
fn checksum_decode_error(checksum: u64, error: &Leb128DecodeError) -> u64 {
    let kind = match error.kind() {
        Leb128DecodeErrorKind::Incomplete => 1,
        Leb128DecodeErrorKind::Malformed => 2,
        Leb128DecodeErrorKind::NonCanonical => 3,
    };
    let mut checksum = mix_checksum(checksum, kind);
    checksum = mix_checksum(checksum, error.error_index() as u64);
    checksum = mix_checksum(
        checksum,
        error.consumed().map_or(0, |count| count.get() as u64),
    );
    mix_checksum(
        checksum,
        error.required().map_or(0, |count| count.get() as u64),
    )
}

/// Repeatedly exercises one unsigned LEB128 error path.
fn decode_unsigned_errors<P>(input: &[u8]) -> u64
where
    P: Leb128DecodePolicy,
{
    let input = black_box(input);
    let mut checksum = 0_u64;
    for _ in 0..BATCH_SIZE {
        // SAFETY: callers pass a fixture known to be rejected by this codec.
        let error = unsafe {
            Leb128Codec::<u64, P>::decode(input, 0).unwrap_err_unchecked()
        };
        checksum = checksum_decode_error(checksum, &error);
    }
    checksum
}

/// Repeatedly exercises one signed LEB128 error path.
fn decode_signed_leb128_errors<P>(input: &[u8]) -> u64
where
    P: Leb128DecodePolicy,
{
    let input = black_box(input);
    let mut checksum = 0_u64;
    for _ in 0..BATCH_SIZE {
        // SAFETY: callers pass a fixture known to be rejected by this codec.
        let error = unsafe {
            Leb128Codec::<i64, P>::decode(input, 0).unwrap_err_unchecked()
        };
        checksum = checksum_decode_error(checksum, &error);
    }
    checksum
}

/// Repeatedly exercises one ZigZag error path.
fn decode_zig_zag_errors<P>(input: &[u8]) -> u64
where
    P: Leb128DecodePolicy,
{
    let input = black_box(input);
    let mut checksum = 0_u64;
    for _ in 0..BATCH_SIZE {
        // SAFETY: callers pass a fixture known to be rejected by this codec.
        let error = unsafe {
            ZigZagCodec::<i64, P>::decode(input, 0).unwrap_err_unchecked()
        };
        checksum = checksum_decode_error(checksum, &error);
    }
    checksum
}

/// Encodes every fixed-width fixture and observes every output byte.
fn encode_binary_values(values: &[u64]) -> u64 {
    let mut checksum = 0_u64;
    let mut bytes =
        [0_u8; BinaryCodec::<u64, BigEndian>::MAX_ENCODE_UNITS_PER_VALUE];
    for &value in black_box(values) {
        unsafe {
            BinaryCodec::<u64, BigEndian>::encode(value, &mut bytes, 0);
        }
        checksum = checksum_encoded(checksum, &bytes, bytes.len());
    }
    checksum
}

/// A deterministic mixed fixed-width unsigned binary field.
#[derive(Clone, Copy)]
enum MixedBinaryField {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
}

/// Builds a mixed fixed-width fixture with randomly selected field types.
fn build_mixed_binary_fields() -> Vec<MixedBinaryField> {
    let mut state = 0x5EED_CAFE_1234_5678;
    (0..MIXED_FIELD_COUNT)
        .map(|_| match next_state(&mut state) % 5 {
            0 => MixedBinaryField::U8(next_state(&mut state) as u8),
            1 => MixedBinaryField::U16(next_state(&mut state) as u16),
            2 => MixedBinaryField::U32(next_state(&mut state) as u32),
            3 => MixedBinaryField::U64(next_state(&mut state)),
            _ => {
                let high = u128::from(next_state(&mut state));
                let low = u128::from(next_state(&mut state));
                MixedBinaryField::U128((high << 64) | low)
            }
        })
        .collect()
}

/// Returns the fixed encoded width of one mixed binary field.
fn mixed_binary_width(field: &MixedBinaryField) -> usize {
    match field {
        MixedBinaryField::U8(_) => {
            BinaryCodec::<u8, BigEndian>::MAX_ENCODE_UNITS_PER_VALUE
        }
        MixedBinaryField::U16(_) => {
            BinaryCodec::<u16, BigEndian>::MAX_ENCODE_UNITS_PER_VALUE
        }
        MixedBinaryField::U32(_) => {
            BinaryCodec::<u32, BigEndian>::MAX_ENCODE_UNITS_PER_VALUE
        }
        MixedBinaryField::U64(_) => {
            BinaryCodec::<u64, BigEndian>::MAX_ENCODE_UNITS_PER_VALUE
        }
        MixedBinaryField::U128(_) => {
            BinaryCodec::<u128, BigEndian>::MAX_ENCODE_UNITS_PER_VALUE
        }
    }
}

/// Returns the total encoded width of a mixed binary fixture.
fn mixed_binary_storage_len(fields: &[MixedBinaryField]) -> usize {
    fields.iter().map(mixed_binary_width).sum()
}

/// Checksums a complete output buffer after a mixed encode operation.
#[inline(always)]
fn checksum_buffer(bytes: &[u8]) -> u64 {
    let mut checksum = mix_checksum(0, bytes.len() as u64);
    for &byte in black_box(bytes) {
        checksum = mix_checksum(checksum, u64::from(byte));
    }
    checksum
}

/// Encodes mixed fixed-width fields without recreating a checked slice.
fn encode_mixed_binary_unchecked(
    fields: &[MixedBinaryField],
    output: &mut [u8],
) -> u64 {
    let fields = black_box(fields);
    let mut offset = 0_usize;
    for field in fields {
        let width = mixed_binary_width(field);
        let output_index = black_box(offset);
        match field {
            MixedBinaryField::U8(value) => {
                let written = unsafe {
                    BinaryCodec::<u8, BigEndian>::encode(
                        *value,
                        output,
                        output_index,
                    )
                };
                debug_assert_eq!(written, width);
            }
            MixedBinaryField::U16(value) => {
                let written = unsafe {
                    BinaryCodec::<u16, BigEndian>::encode(
                        *value,
                        output,
                        output_index,
                    )
                };
                debug_assert_eq!(written, width);
            }
            MixedBinaryField::U32(value) => {
                let written = unsafe {
                    BinaryCodec::<u32, BigEndian>::encode(
                        *value,
                        output,
                        output_index,
                    )
                };
                debug_assert_eq!(written, width);
            }
            MixedBinaryField::U64(value) => {
                let written = unsafe {
                    BinaryCodec::<u64, BigEndian>::encode(
                        *value,
                        output,
                        output_index,
                    )
                };
                debug_assert_eq!(written, width);
            }
            MixedBinaryField::U128(value) => {
                let written = unsafe {
                    BinaryCodec::<u128, BigEndian>::encode(
                        *value,
                        output,
                        output_index,
                    )
                };
                debug_assert_eq!(written, width);
            }
        }
        offset += width;
    }
    debug_assert_eq!(offset, output.len());
    checksum_buffer(output)
}

/// Encodes mixed fixed-width fields through a checked slice per field.
fn encode_mixed_binary_safe_slices(
    fields: &[MixedBinaryField],
    output: &mut [u8],
) -> u64 {
    let fields = black_box(fields);
    let mut offset = 0_usize;
    for field in fields {
        let width = mixed_binary_width(field);
        let output_index = black_box(offset);
        let window = &mut output[output_index..output_index + width];
        match field {
            MixedBinaryField::U8(value) => unsafe {
                BinaryCodec::<u8, BigEndian>::encode(*value, window, 0);
            },
            MixedBinaryField::U16(value) => unsafe {
                BinaryCodec::<u16, BigEndian>::encode(*value, window, 0);
            },
            MixedBinaryField::U32(value) => unsafe {
                BinaryCodec::<u32, BigEndian>::encode(*value, window, 0);
            },
            MixedBinaryField::U64(value) => unsafe {
                BinaryCodec::<u64, BigEndian>::encode(*value, window, 0);
            },
            MixedBinaryField::U128(value) => unsafe {
                BinaryCodec::<u128, BigEndian>::encode(*value, window, 0);
            },
        }
        offset += width;
    }
    debug_assert_eq!(offset, output.len());
    checksum_buffer(output)
}

/// Decodes mixed fixed-width fields without recreating a checked slice.
fn decode_mixed_binary_unchecked(
    fields: &[MixedBinaryField],
    input: &[u8],
) -> u64 {
    let fields = black_box(fields);
    let input = black_box(input);
    let mut offset = 0_usize;
    let mut checksum = 0_u64;
    for field in fields {
        let width = mixed_binary_width(field);
        let input_index = black_box(offset);
        match field {
            MixedBinaryField::U8(_) => {
                let (value, consumed) = unsafe {
                    BinaryCodec::<u8, BigEndian>::decode(input, input_index)
                };
                debug_assert_eq!(consumed.get(), width);
                checksum = mix_checksum(checksum, u64::from(value));
            }
            MixedBinaryField::U16(_) => {
                let (value, consumed) = unsafe {
                    BinaryCodec::<u16, BigEndian>::decode(input, input_index)
                };
                debug_assert_eq!(consumed.get(), width);
                checksum = mix_checksum(checksum, u64::from(value));
            }
            MixedBinaryField::U32(_) => {
                let (value, consumed) = unsafe {
                    BinaryCodec::<u32, BigEndian>::decode(input, input_index)
                };
                debug_assert_eq!(consumed.get(), width);
                checksum = mix_checksum(checksum, u64::from(value));
            }
            MixedBinaryField::U64(_) => {
                let (value, consumed) = unsafe {
                    BinaryCodec::<u64, BigEndian>::decode(input, input_index)
                };
                debug_assert_eq!(consumed.get(), width);
                checksum = mix_checksum(checksum, value);
            }
            MixedBinaryField::U128(_) => {
                let (value, consumed) = unsafe {
                    BinaryCodec::<u128, BigEndian>::decode(input, input_index)
                };
                debug_assert_eq!(consumed.get(), width);
                checksum = mix_checksum(checksum, value as u64);
            }
        }
        offset += width;
    }
    debug_assert_eq!(offset, input.len());
    checksum
}

/// Decodes mixed fixed-width fields through a checked slice per field.
fn decode_mixed_binary_safe_slices(
    fields: &[MixedBinaryField],
    input: &[u8],
) -> u64 {
    let fields = black_box(fields);
    let input = black_box(input);
    let mut offset = 0_usize;
    let mut checksum = 0_u64;
    for field in fields {
        let width = mixed_binary_width(field);
        let input_index = black_box(offset);
        let window = &input[input_index..input_index + width];
        match field {
            MixedBinaryField::U8(_) => {
                let (value, consumed) =
                    unsafe { BinaryCodec::<u8, BigEndian>::decode(window, 0) };
                debug_assert_eq!(consumed.get(), width);
                checksum = mix_checksum(checksum, u64::from(value));
            }
            MixedBinaryField::U16(_) => {
                let (value, consumed) =
                    unsafe { BinaryCodec::<u16, BigEndian>::decode(window, 0) };
                debug_assert_eq!(consumed.get(), width);
                checksum = mix_checksum(checksum, u64::from(value));
            }
            MixedBinaryField::U32(_) => {
                let (value, consumed) =
                    unsafe { BinaryCodec::<u32, BigEndian>::decode(window, 0) };
                debug_assert_eq!(consumed.get(), width);
                checksum = mix_checksum(checksum, u64::from(value));
            }
            MixedBinaryField::U64(_) => {
                let (value, consumed) =
                    unsafe { BinaryCodec::<u64, BigEndian>::decode(window, 0) };
                debug_assert_eq!(consumed.get(), width);
                checksum = mix_checksum(checksum, value);
            }
            MixedBinaryField::U128(_) => {
                let (value, consumed) = unsafe {
                    BinaryCodec::<u128, BigEndian>::decode(window, 0)
                };
                debug_assert_eq!(consumed.get(), width);
                checksum = mix_checksum(checksum, value as u64);
            }
        }
        offset += width;
    }
    debug_assert_eq!(offset, input.len());
    checksum
}

/// Benchmarks mixed fixed-width binary fields with and without checked slices.
fn bench_mixed_binary(c: &mut Criterion) {
    let fields = build_mixed_binary_fields();
    let storage_len = mixed_binary_storage_len(&fields);
    let mut unchecked_encoded = vec![0_u8; storage_len];
    let mut safe_encoded = vec![0_u8; storage_len];
    let unchecked_checksum =
        encode_mixed_binary_unchecked(&fields, &mut unchecked_encoded);
    let safe_checksum =
        encode_mixed_binary_safe_slices(&fields, &mut safe_encoded);
    assert_eq!(unchecked_encoded, safe_encoded);
    assert_eq!(unchecked_checksum, safe_checksum);
    assert_eq!(
        decode_mixed_binary_unchecked(&fields, &unchecked_encoded),
        decode_mixed_binary_safe_slices(&fields, &safe_encoded),
    );

    let mut group = c.benchmark_group("mixed_binary_codec");
    group.throughput(Throughput::Bytes(storage_len as u64));
    group.bench_function("encode_unchecked", |bencher| {
        bencher.iter_batched(
            || vec![0_u8; storage_len],
            |mut output| {
                black_box(encode_mixed_binary_unchecked(&fields, &mut output));
            },
            BatchSize::LargeInput,
        );
    });
    group.bench_function("encode_safe_slices", |bencher| {
        bencher.iter_batched(
            || vec![0_u8; storage_len],
            |mut output| {
                black_box(encode_mixed_binary_safe_slices(
                    &fields,
                    &mut output,
                ));
            },
            BatchSize::LargeInput,
        );
    });
    group.bench_function("decode_unchecked", |bencher| {
        bencher.iter(|| {
            black_box(decode_mixed_binary_unchecked(
                &fields,
                &unchecked_encoded,
            ))
        });
    });
    group.bench_function("decode_safe_slices", |bencher| {
        bencher.iter(|| {
            black_box(decode_mixed_binary_safe_slices(&fields, &safe_encoded))
        });
    });
    group.finish();
}

/// Benchmarks direct fixed-width big-endian integer encoding and decoding.
fn bench_binary(c: &mut Criterion) {
    let values = unsigned_values(Distribution::Uniform);
    let mut group = c.benchmark_group("binary_codec");
    group.throughput(Throughput::Elements(BATCH_SIZE as u64));
    group.bench_function("u64_big_endian_roundtrip", |bencher| {
        bencher.iter(|| {
            let mut checksum = 0_u64;
            let mut bytes = [0_u8;
                BinaryCodec::<u64, BigEndian>::MAX_ENCODE_UNITS_PER_VALUE];
            for &value in black_box(&values) {
                unsafe {
                    BinaryCodec::<u64, BigEndian>::encode(value, &mut bytes, 0);
                }
                let (decoded, _) =
                    unsafe { BinaryCodec::<u64, BigEndian>::decode(&bytes, 0) };
                checksum = mix_checksum(checksum, decoded);
            }
            black_box(checksum)
        });
    });
    group.bench_function("u64_big_endian_encode", |bencher| {
        bencher.iter(|| black_box(encode_binary_values(&values)));
    });
    group.finish();
}

/// A deterministic mixed unsigned LEB128 field schema.
#[derive(Clone, Copy)]
enum MixedUlebField {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    Usize(usize),
    U128(u128),
}

/// Encoded bytes and per-field widths for one mixed LEB128 fixture.
struct MixedUlebFixture {
    fields: Vec<MixedUlebField>,
    widths: Vec<usize>,
    encoded: Vec<u8>,
}

/// Builds a mixed unsigned LEB128 fixture with random field types and widths.
fn build_mixed_uleb_fixture() -> MixedUlebFixture {
    let mut state = 0xCAFE_BABE_1234_5678;
    let fields = (0..MIXED_FIELD_COUNT)
        .map(|_| match next_state(&mut state) % 6 {
            0 => MixedUlebField::U8(next_state(&mut state) as u8),
            1 => MixedUlebField::U16(next_state(&mut state) as u16),
            2 => MixedUlebField::U32(next_state(&mut state) as u32),
            3 => MixedUlebField::U64(next_state(&mut state)),
            4 => MixedUlebField::Usize(next_state(&mut state) as usize),
            _ => {
                let high = u128::from(next_state(&mut state));
                let low = u128::from(next_state(&mut state));
                MixedUlebField::U128((high << 64) | low)
            }
        })
        .collect::<Vec<_>>();
    let widths = fields.iter().map(mixed_uleb_width).collect::<Vec<_>>();
    assert!(
        widths
            .iter()
            .all(|&width| (1..=MAX_MIXED_ULEB_BYTES).contains(&width))
    );
    let storage_len = widths.iter().sum();
    let mut encoded = vec![0_u8; storage_len];
    encode_mixed_uleb_unchecked(&fields, &widths, &mut encoded);
    MixedUlebFixture {
        fields,
        widths,
        encoded,
    }
}

/// Returns the canonical encoded width of one mixed unsigned LEB128 field.
fn mixed_uleb_width(field: &MixedUlebField) -> usize {
    match field {
        MixedUlebField::U8(value) => {
            Leb128Codec::<u8, NonStrict>::default().encode_len(value)
        }
        MixedUlebField::U16(value) => {
            Leb128Codec::<u16, NonStrict>::default().encode_len(value)
        }
        MixedUlebField::U32(value) => {
            Leb128Codec::<u32, NonStrict>::default().encode_len(value)
        }
        MixedUlebField::U64(value) => {
            Leb128Codec::<u64, NonStrict>::default().encode_len(value)
        }
        MixedUlebField::Usize(value) => {
            Leb128Codec::<usize, NonStrict>::default().encode_len(value)
        }
        MixedUlebField::U128(value) => {
            Leb128Codec::<u128, NonStrict>::default().encode_len(value)
        }
    }
}

/// Encodes mixed LEB128 fields without recreating a checked slice.
fn encode_mixed_uleb_unchecked(
    fields: &[MixedUlebField],
    widths: &[usize],
    output: &mut [u8],
) -> u64 {
    debug_assert_eq!(fields.len(), widths.len());
    let fields = black_box(fields);
    let widths = black_box(widths);
    let mut offset = 0_usize;
    for index in 0..fields.len() {
        let field = &fields[index];
        let width = black_box(widths[index]);
        let output_index = black_box(offset);
        let written = match field {
            MixedUlebField::U8(value) => unsafe {
                Leb128Codec::<u8, NonStrict>::encode(
                    *value,
                    output,
                    output_index,
                )
            },
            MixedUlebField::U16(value) => unsafe {
                Leb128Codec::<u16, NonStrict>::encode(
                    *value,
                    output,
                    output_index,
                )
            },
            MixedUlebField::U32(value) => unsafe {
                Leb128Codec::<u32, NonStrict>::encode(
                    *value,
                    output,
                    output_index,
                )
            },
            MixedUlebField::U64(value) => unsafe {
                Leb128Codec::<u64, NonStrict>::encode(
                    *value,
                    output,
                    output_index,
                )
            },
            MixedUlebField::Usize(value) => unsafe {
                Leb128Codec::<usize, NonStrict>::encode(
                    *value,
                    output,
                    output_index,
                )
            },
            MixedUlebField::U128(value) => unsafe {
                Leb128Codec::<u128, NonStrict>::encode(
                    *value,
                    output,
                    output_index,
                )
            },
        };
        debug_assert_eq!(written, width);
        offset += width;
    }
    debug_assert_eq!(offset, output.len());
    checksum_buffer(output)
}

/// Encodes mixed LEB128 fields through a checked slice per field.
fn encode_mixed_uleb_safe_slices(
    fields: &[MixedUlebField],
    widths: &[usize],
    output: &mut [u8],
) -> u64 {
    debug_assert_eq!(fields.len(), widths.len());
    let fields = black_box(fields);
    let widths = black_box(widths);
    let mut offset = 0_usize;
    for index in 0..fields.len() {
        let field = &fields[index];
        let width = black_box(widths[index]);
        let output_index = black_box(offset);
        let window = &mut output[output_index..output_index + width];
        match field {
            MixedUlebField::U8(value) => unsafe {
                let _ = Leb128Codec::<u8, NonStrict>::encode(*value, window, 0);
            },
            MixedUlebField::U16(value) => unsafe {
                let _ =
                    Leb128Codec::<u16, NonStrict>::encode(*value, window, 0);
            },
            MixedUlebField::U32(value) => unsafe {
                let _ =
                    Leb128Codec::<u32, NonStrict>::encode(*value, window, 0);
            },
            MixedUlebField::U64(value) => unsafe {
                let _ =
                    Leb128Codec::<u64, NonStrict>::encode(*value, window, 0);
            },
            MixedUlebField::Usize(value) => unsafe {
                let _ =
                    Leb128Codec::<usize, NonStrict>::encode(*value, window, 0);
            },
            MixedUlebField::U128(value) => unsafe {
                let _ =
                    Leb128Codec::<u128, NonStrict>::encode(*value, window, 0);
            },
        }
        offset += width;
    }
    debug_assert_eq!(offset, output.len());
    checksum_buffer(output)
}

/// Decodes mixed LEB128 fields without recreating a checked slice.
fn decode_mixed_uleb_unchecked(fields: &[MixedUlebField], input: &[u8]) -> u64 {
    let fields = black_box(fields);
    let input = black_box(input);
    let mut offset = 0_usize;
    let mut checksum = 0_u64;
    for field in fields {
        let input_index = black_box(offset);
        match field {
            MixedUlebField::U8(_) => {
                let (value, consumed) = unsafe {
                    Leb128Codec::<u8, NonStrict>::decode(input, input_index)
                        .unwrap_unchecked()
                };
                checksum = mix_checksum(checksum, u64::from(value));
                offset += consumed.get();
            }
            MixedUlebField::U16(_) => {
                let (value, consumed) = unsafe {
                    Leb128Codec::<u16, NonStrict>::decode(input, input_index)
                        .unwrap_unchecked()
                };
                checksum = mix_checksum(checksum, u64::from(value));
                offset += consumed.get();
            }
            MixedUlebField::U32(_) => {
                let (value, consumed) = unsafe {
                    Leb128Codec::<u32, NonStrict>::decode(input, input_index)
                        .unwrap_unchecked()
                };
                checksum = mix_checksum(checksum, u64::from(value));
                offset += consumed.get();
            }
            MixedUlebField::U64(_) => {
                let (value, consumed) = unsafe {
                    Leb128Codec::<u64, NonStrict>::decode(input, input_index)
                        .unwrap_unchecked()
                };
                checksum = mix_checksum(checksum, value);
                offset += consumed.get();
            }
            MixedUlebField::Usize(_) => {
                let (value, consumed) = unsafe {
                    Leb128Codec::<usize, NonStrict>::decode(input, input_index)
                        .unwrap_unchecked()
                };
                checksum = mix_checksum(checksum, value as u64);
                offset += consumed.get();
            }
            MixedUlebField::U128(_) => {
                let (value, consumed) = unsafe {
                    Leb128Codec::<u128, NonStrict>::decode(input, input_index)
                        .unwrap_unchecked()
                };
                checksum = mix_checksum(checksum, value as u64);
                offset += consumed.get();
            }
        }
    }
    debug_assert_eq!(offset, input.len());
    checksum
}

/// Decodes mixed LEB128 fields through a checked remaining-input slice per
/// field.
fn decode_mixed_uleb_safe_slices(
    fields: &[MixedUlebField],
    input: &[u8],
) -> u64 {
    let fields = black_box(fields);
    let input = black_box(input);
    let mut offset = 0_usize;
    let mut checksum = 0_u64;
    for field in fields {
        let input_index = black_box(offset);
        let window = &input[input_index..];
        match field {
            MixedUlebField::U8(_) => {
                let (value, consumed) = unsafe {
                    Leb128Codec::<u8, NonStrict>::decode(window, 0)
                        .unwrap_unchecked()
                };
                checksum = mix_checksum(checksum, u64::from(value));
                offset += consumed.get();
            }
            MixedUlebField::U16(_) => {
                let (value, consumed) = unsafe {
                    Leb128Codec::<u16, NonStrict>::decode(window, 0)
                        .unwrap_unchecked()
                };
                checksum = mix_checksum(checksum, u64::from(value));
                offset += consumed.get();
            }
            MixedUlebField::U32(_) => {
                let (value, consumed) = unsafe {
                    Leb128Codec::<u32, NonStrict>::decode(window, 0)
                        .unwrap_unchecked()
                };
                checksum = mix_checksum(checksum, u64::from(value));
                offset += consumed.get();
            }
            MixedUlebField::U64(_) => {
                let (value, consumed) = unsafe {
                    Leb128Codec::<u64, NonStrict>::decode(window, 0)
                        .unwrap_unchecked()
                };
                checksum = mix_checksum(checksum, value);
                offset += consumed.get();
            }
            MixedUlebField::Usize(_) => {
                let (value, consumed) = unsafe {
                    Leb128Codec::<usize, NonStrict>::decode(window, 0)
                        .unwrap_unchecked()
                };
                checksum = mix_checksum(checksum, value as u64);
                offset += consumed.get();
            }
            MixedUlebField::U128(_) => {
                let (value, consumed) = unsafe {
                    Leb128Codec::<u128, NonStrict>::decode(window, 0)
                        .unwrap_unchecked()
                };
                checksum = mix_checksum(checksum, value as u64);
                offset += consumed.get();
            }
        }
    }
    debug_assert_eq!(offset, input.len());
    checksum
}

/// Benchmarks mixed unsigned LEB128 fields with and without checked slices.
fn bench_mixed_uleb(c: &mut Criterion) {
    let fixture = build_mixed_uleb_fixture();
    let storage_len = fixture.encoded.len();
    let mut unchecked_encoded = vec![0_u8; storage_len];
    let mut safe_encoded = vec![0_u8; storage_len];
    let unchecked_checksum = encode_mixed_uleb_unchecked(
        &fixture.fields,
        &fixture.widths,
        &mut unchecked_encoded,
    );
    let safe_checksum = encode_mixed_uleb_safe_slices(
        &fixture.fields,
        &fixture.widths,
        &mut safe_encoded,
    );
    assert_eq!(unchecked_encoded, safe_encoded);
    assert_eq!(unchecked_checksum, safe_checksum);
    assert_eq!(unchecked_encoded, fixture.encoded);
    assert_eq!(
        decode_mixed_uleb_unchecked(&fixture.fields, &fixture.encoded,),
        decode_mixed_uleb_safe_slices(&fixture.fields, &fixture.encoded,),
    );

    let mut group = c.benchmark_group("mixed_uleb128_codec");
    group.throughput(Throughput::Bytes(storage_len as u64));
    group.bench_function("encode_unchecked", |bencher| {
        bencher.iter_batched(
            || vec![0_u8; storage_len],
            |mut output| {
                black_box(encode_mixed_uleb_unchecked(
                    &fixture.fields,
                    &fixture.widths,
                    &mut output,
                ));
            },
            BatchSize::LargeInput,
        );
    });
    group.bench_function("encode_safe_slices", |bencher| {
        bencher.iter_batched(
            || vec![0_u8; storage_len],
            |mut output| {
                black_box(encode_mixed_uleb_safe_slices(
                    &fixture.fields,
                    &fixture.widths,
                    &mut output,
                ));
            },
            BatchSize::LargeInput,
        );
    });
    group.bench_function("decode_unchecked", |bencher| {
        bencher.iter(|| {
            black_box(decode_mixed_uleb_unchecked(
                &fixture.fields,
                &fixture.encoded,
            ))
        });
    });
    group.bench_function("decode_safe_slices", |bencher| {
        bencher.iter(|| {
            black_box(decode_mixed_uleb_safe_slices(
                &fixture.fields,
                &fixture.encoded,
            ))
        });
    });
    group.finish();
}

/// Benchmarks unsigned and signed LEB128 operations by value distribution.
fn bench_leb128(c: &mut Criterion) {
    let mut group = c.benchmark_group("leb128_codec");
    group.throughput(Throughput::Elements(BATCH_SIZE as u64));

    for distribution in DISTRIBUTIONS {
        let name = distribution.name();
        let values = unsigned_values(distribution);
        validate_exact_capacity::<Leb128Codec<u64, NonStrict>>(&values);
        let payloads = unsigned_payloads(&values);
        validate_distribution_widths(
            distribution,
            payloads.iter().map(|payload| payload.len),
        );

        group.bench_with_input(
            BenchmarkId::new("u64_encode_len", name),
            &values,
            |bencher, values| {
                bencher.iter(|| {
                    black_box(encode_lengths::<Leb128Codec<u64, NonStrict>>(
                        values,
                    ))
                });
            },
        );
        group.bench_with_input(
            BenchmarkId::new("u64_encode_direct", name),
            &values,
            |bencher, values| {
                bencher.iter(|| black_box(encode_unsigned_direct(values)));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("u64_encode_codec_exact", name),
            &values,
            |bencher, values| {
                bencher.iter(|| {
                    black_box(encode_exact_capacity::<
                        Leb128Codec<u64, NonStrict>,
                    >(values))
                });
            },
        );
        group.bench_with_input(
            BenchmarkId::new("u64_decode_non_strict", name),
            &payloads,
            |bencher, payloads| {
                bencher.iter(|| {
                    black_box(decode_unsigned_payloads::<NonStrict>(payloads))
                });
            },
        );
        group.bench_with_input(
            BenchmarkId::new("u64_decode_strict", name),
            &payloads,
            |bencher, payloads| {
                bencher.iter(|| {
                    black_box(decode_unsigned_payloads::<Strict>(payloads))
                });
            },
        );

        let values = signed_leb128_values(distribution);
        validate_exact_capacity::<Leb128Codec<i64, NonStrict>>(&values);
        let payloads = signed_leb128_payloads(&values);
        validate_distribution_widths(
            distribution,
            payloads.iter().map(|payload| payload.len),
        );

        group.bench_with_input(
            BenchmarkId::new("i64_encode_len", name),
            &values,
            |bencher, values| {
                bencher.iter(|| {
                    black_box(encode_lengths::<Leb128Codec<i64, NonStrict>>(
                        values,
                    ))
                });
            },
        );
        group.bench_with_input(
            BenchmarkId::new("i64_encode_direct", name),
            &values,
            |bencher, values| {
                bencher.iter(|| black_box(encode_signed_leb128_direct(values)));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("i64_encode_codec_exact", name),
            &values,
            |bencher, values| {
                bencher.iter(|| {
                    black_box(encode_exact_capacity::<
                        Leb128Codec<i64, NonStrict>,
                    >(values))
                });
            },
        );
        group.bench_with_input(
            BenchmarkId::new("i64_decode_non_strict", name),
            &payloads,
            |bencher, payloads| {
                bencher.iter(|| {
                    black_box(decode_signed_leb128_payloads::<NonStrict>(
                        payloads,
                    ))
                });
            },
        );
        group.bench_with_input(
            BenchmarkId::new("i64_decode_strict", name),
            &payloads,
            |bencher, payloads| {
                bencher.iter(|| {
                    black_box(decode_signed_leb128_payloads::<Strict>(payloads))
                });
            },
        );
    }
    group.finish();
}

/// Benchmarks ZigZag operations by signed value distribution.
fn bench_zig_zag(c: &mut Criterion) {
    let mut group = c.benchmark_group("zig_zag_codec");
    group.throughput(Throughput::Elements(BATCH_SIZE as u64));

    for distribution in DISTRIBUTIONS {
        let name = distribution.name();
        let values = zig_zag_values(distribution);
        validate_exact_capacity::<ZigZagCodec<i64, NonStrict>>(&values);
        let payloads = zig_zag_payloads(&values);
        validate_distribution_widths(
            distribution,
            payloads.iter().map(|payload| payload.len),
        );

        group.bench_with_input(
            BenchmarkId::new("i64_encode_len", name),
            &values,
            |bencher, values| {
                bencher.iter(|| {
                    black_box(encode_lengths::<ZigZagCodec<i64, NonStrict>>(
                        values,
                    ))
                });
            },
        );
        group.bench_with_input(
            BenchmarkId::new("i64_encode_direct", name),
            &values,
            |bencher, values| {
                bencher.iter(|| black_box(encode_zig_zag_direct(values)));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("i64_encode_codec_exact", name),
            &values,
            |bencher, values| {
                bencher.iter(|| {
                    black_box(encode_exact_capacity::<
                        ZigZagCodec<i64, NonStrict>,
                    >(values))
                });
            },
        );
        group.bench_with_input(
            BenchmarkId::new("i64_decode_non_strict", name),
            &payloads,
            |bencher, payloads| {
                bencher.iter(|| {
                    black_box(decode_zig_zag_payloads::<NonStrict>(payloads))
                });
            },
        );
        group.bench_with_input(
            BenchmarkId::new("i64_decode_strict", name),
            &payloads,
            |bencher, payloads| {
                bencher.iter(|| {
                    black_box(decode_zig_zag_payloads::<Strict>(payloads))
                });
            },
        );
    }
    group.finish();
}

/// Benchmarks incomplete, malformed, and non-canonical decode paths separately.
fn bench_decode_errors(c: &mut Criterion) {
    let incomplete = [0x80_u8];
    let malformed = [0x80_u8; MAX_VARINT_BYTES];
    let non_canonical = [0x80_u8, 0x00];
    let mut group = c.benchmark_group("varint_decode_errors");
    group.throughput(Throughput::Elements(BATCH_SIZE as u64));

    group.bench_with_input(
        BenchmarkId::new("uleb_u64", "incomplete"),
        &incomplete,
        |bencher, input| {
            bencher
                .iter(|| black_box(decode_unsigned_errors::<NonStrict>(input)));
        },
    );
    group.bench_with_input(
        BenchmarkId::new("uleb_u64", "malformed"),
        &malformed,
        |bencher, input| {
            bencher
                .iter(|| black_box(decode_unsigned_errors::<NonStrict>(input)));
        },
    );
    group.bench_with_input(
        BenchmarkId::new("uleb_u64", "non_canonical"),
        &non_canonical,
        |bencher, input| {
            bencher.iter(|| black_box(decode_unsigned_errors::<Strict>(input)));
        },
    );

    group.bench_with_input(
        BenchmarkId::new("sleb_i64", "incomplete"),
        &incomplete,
        |bencher, input| {
            bencher.iter(|| {
                black_box(decode_signed_leb128_errors::<NonStrict>(input))
            });
        },
    );
    group.bench_with_input(
        BenchmarkId::new("sleb_i64", "malformed"),
        &malformed,
        |bencher, input| {
            bencher.iter(|| {
                black_box(decode_signed_leb128_errors::<NonStrict>(input))
            });
        },
    );
    group.bench_with_input(
        BenchmarkId::new("sleb_i64", "non_canonical"),
        &non_canonical,
        |bencher, input| {
            bencher.iter(|| {
                black_box(decode_signed_leb128_errors::<Strict>(input))
            });
        },
    );

    group.bench_with_input(
        BenchmarkId::new("zig_zag_i64", "incomplete"),
        &incomplete,
        |bencher, input| {
            bencher
                .iter(|| black_box(decode_zig_zag_errors::<NonStrict>(input)));
        },
    );
    group.bench_with_input(
        BenchmarkId::new("zig_zag_i64", "malformed"),
        &malformed,
        |bencher, input| {
            bencher
                .iter(|| black_box(decode_zig_zag_errors::<NonStrict>(input)));
        },
    );
    group.bench_with_input(
        BenchmarkId::new("zig_zag_i64", "non_canonical"),
        &non_canonical,
        |bencher, input| {
            bencher.iter(|| black_box(decode_zig_zag_errors::<Strict>(input)));
        },
    );
    group.finish();
}

criterion_group!(
    codec_benches,
    bench_binary,
    bench_mixed_binary,
    bench_mixed_uleb,
    bench_leb128,
    bench_zig_zag,
    bench_decode_errors,
);
criterion_main!(codec_benches);
