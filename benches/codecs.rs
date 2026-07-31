// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Direct throughput benchmarks for the binary codec hot paths.

use std::{
    fmt::Debug,
    hint::black_box,
};

use criterion::{
    BenchmarkId,
    Criterion,
    Throughput,
    criterion_group,
    criterion_main,
};
use qubit_codec::{
    BigEndian,
    Codec,
};
use qubit_codec_binary::{
    BinaryCodec,
    Leb128Codec,
    Leb128DecodeError,
    Leb128DecodeErrorKind,
    Leb128DecodePolicy,
    NonStrict,
    Strict,
    ZigZagCodec,
};

/// Number of values processed by each benchmark iteration.
const BATCH_SIZE: usize = 1_024;

/// Maximum encoded width of each 64-bit varint family benchmarked here.
const MAX_VARINT_BYTES: usize = 10;

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
    C: Codec<Unit = u8> + Default,
    C::EncodeError: Debug,
{
    assert!(C::MAX_ENCODE_UNITS_PER_VALUE <= MAX_VARINT_BYTES);
    let mut codec = C::default();
    let mut checksum = 0_u64;
    let mut storage = [GUARD_BYTE; MAX_VARINT_BYTES + 2];
    for value in black_box(values) {
        assert!(codec.can_encode_value(value));
        let required = codec.encode_len(black_box(value));
        assert!(required <= C::MAX_ENCODE_UNITS_PER_VALUE);
        let output = &mut storage[1..1 + required];
        let written = unsafe {
            // SAFETY: `output` exposes exactly the width reported for the same
            // value and unchanged codec state.
            Codec::encode(&mut codec, value, output, 0)
        }
        .expect("benchmark value should encode");
        assert_eq!(required, written);
        checksum = checksum_encoded(checksum, output, written);
    }
    assert_eq!(GUARD_BYTE, storage[0]);
    assert_eq!(GUARD_BYTE, storage[MAX_VARINT_BYTES + 1]);
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
        let (value, consumed) = unsafe {
            Leb128Codec::<u64, P>::decode(&payload.bytes[..payload.len], 0)
        }
        .expect("canonical fixture should decode");
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
        let (value, consumed) = unsafe {
            Leb128Codec::<i64, P>::decode(&payload.bytes[..payload.len], 0)
        }
        .expect("canonical fixture should decode");
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
        let (value, consumed) = unsafe {
            ZigZagCodec::<i64, P>::decode(&payload.bytes[..payload.len], 0)
        }
        .expect("canonical fixture should decode");
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
        let error = unsafe { Leb128Codec::<u64, P>::decode(input, 0) }
            .expect_err("error fixture should be rejected");
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
        let error = unsafe { Leb128Codec::<i64, P>::decode(input, 0) }
            .expect_err("error fixture should be rejected");
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
        let error = unsafe { ZigZagCodec::<i64, P>::decode(input, 0) }
            .expect_err("error fixture should be rejected");
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
    bench_leb128,
    bench_zig_zag,
    bench_decode_errors,
);
criterion_main!(codec_benches);
