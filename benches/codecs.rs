// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Direct throughput benchmarks for the binary codec hot paths.

use std::hint::black_box;

use criterion::{
    BenchmarkId,
    Criterion,
    Throughput,
    criterion_group,
    criterion_main,
};
use qubit_codec::BigEndian;
use qubit_codec_binary::{
    BinaryCodec,
    Leb128Codec,
    Leb128DecodePolicy,
    NonStrict,
    Strict,
    ZigZagCodec,
};

/// Number of values processed by each benchmark iteration.
const BATCH_SIZE: usize = 1_024;

/// A canonical LEB128 payload and its decoded unsigned value.
#[derive(Clone, Copy)]
struct UnsignedPayload {
    bytes: [u8; Leb128Codec::<u64, NonStrict>::MAX_UNITS_PER_VALUE],
    len: usize,
}

/// A canonical ZigZag payload and its decoded signed value.
#[derive(Clone, Copy)]
struct SignedPayload {
    bytes: [u8; ZigZagCodec::<i64, NonStrict>::MAX_UNITS_PER_VALUE],
    len: usize,
}

/// A canonical signed LEB128 payload and its decoded value.
#[derive(Clone, Copy)]
struct SignedLeb128Payload {
    bytes: [u8; Leb128Codec::<i64, NonStrict>::MAX_UNITS_PER_VALUE],
    len: usize,
}

/// Builds deterministic values spanning common and multi-byte encodings.
fn values() -> Vec<u64> {
    let mut state = 0xD1CE_BA5E_1234_5678_u64;
    let mut values = Vec::with_capacity(BATCH_SIZE);
    for index in 0..BATCH_SIZE {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        values.push(match index % 4 {
            0 => state & 0x7F,
            1 => state & 0x3FFF,
            2 => state & 0x1F_FFFF,
            _ => state,
        });
    }
    values
}

/// Encodes the fixture used by unsigned LEB128 decode benchmarks.
fn unsigned_payloads(values: &[u64]) -> Vec<UnsignedPayload> {
    values
        .iter()
        .copied()
        .map(|value| {
            let mut bytes =
                [0_u8; Leb128Codec::<u64, NonStrict>::MAX_UNITS_PER_VALUE];
            let len = unsafe {
                Leb128Codec::<u64, NonStrict>::encode(value, &mut bytes, 0)
            };
            UnsignedPayload { bytes, len }
        })
        .collect()
}

/// Encodes the fixture used by signed ZigZag decode benchmarks.
fn signed_payloads(values: &[u64]) -> Vec<SignedPayload> {
    values
        .iter()
        .copied()
        .map(|value| {
            let value = value as i64;
            let mut bytes =
                [0_u8; ZigZagCodec::<i64, NonStrict>::MAX_UNITS_PER_VALUE];
            let len = unsafe {
                ZigZagCodec::<i64, NonStrict>::encode(value, &mut bytes, 0)
            };
            SignedPayload { bytes, len }
        })
        .collect()
}

/// Encodes the fixture used by signed LEB128 decode benchmarks.
fn signed_leb128_payloads(values: &[u64]) -> Vec<SignedLeb128Payload> {
    values
        .iter()
        .copied()
        .map(|value| {
            let value = value as i64;
            let mut bytes =
                [0_u8; Leb128Codec::<i64, NonStrict>::MAX_UNITS_PER_VALUE];
            let len = unsafe {
                Leb128Codec::<i64, NonStrict>::encode(value, &mut bytes, 0)
            };
            SignedLeb128Payload { bytes, len }
        })
        .collect()
}

/// Decodes every canonical unsigned LEB128 fixture under one policy.
fn decode_unsigned_payloads<P>(payloads: &[UnsignedPayload]) -> u64
where
    P: Leb128DecodePolicy,
{
    let mut checksum = 0_u64;
    for payload in black_box(payloads) {
        let value = match unsafe {
            Leb128Codec::<u64, P>::decode(&payload.bytes[..payload.len], 0)
        } {
            Ok((value, _)) => value,
            Err(error) => panic!("canonical fixture rejected: {error}"),
        };
        checksum ^= value;
    }
    checksum
}

/// Decodes every canonical ZigZag fixture under one policy.
fn decode_signed_payloads<P>(payloads: &[SignedPayload]) -> i64
where
    P: Leb128DecodePolicy,
{
    let mut checksum = 0_i64;
    for payload in black_box(payloads) {
        let value = match unsafe {
            ZigZagCodec::<i64, P>::decode(&payload.bytes[..payload.len], 0)
        } {
            Ok((value, _)) => value,
            Err(error) => panic!("canonical fixture rejected: {error}"),
        };
        checksum ^= value;
    }
    checksum
}

/// Decodes every canonical signed LEB128 fixture under one policy.
fn decode_signed_leb128_payloads<P>(payloads: &[SignedLeb128Payload]) -> i64
where
    P: Leb128DecodePolicy,
{
    let mut checksum = 0_i64;
    for payload in black_box(payloads) {
        let value = match unsafe {
            Leb128Codec::<i64, P>::decode(&payload.bytes[..payload.len], 0)
        } {
            Ok((value, _)) => value,
            Err(error) => panic!("canonical fixture rejected: {error}"),
        };
        checksum ^= value;
    }
    checksum
}

/// Encodes every fixed-width fixture and returns a checksum.
fn encode_binary_values(values: &[u64]) -> u64 {
    let mut checksum = 0_u64;
    let mut bytes = [0_u8; BinaryCodec::<u64, BigEndian>::MAX_UNITS_PER_VALUE];
    for &value in black_box(values) {
        unsafe {
            BinaryCodec::<u64, BigEndian>::encode(value, &mut bytes, 0);
        }
        checksum ^= u64::from(bytes[0]);
    }
    checksum
}

/// Encodes every unsigned LEB128 fixture and returns a checksum.
fn encode_unsigned_leb128_values(values: &[u64]) -> usize {
    let mut checksum = 0_usize;
    let mut bytes = [0_u8; Leb128Codec::<u64, NonStrict>::MAX_UNITS_PER_VALUE];
    for &value in black_box(values) {
        let len = unsafe {
            Leb128Codec::<u64, NonStrict>::encode(value, &mut bytes, 0)
        };
        checksum ^= len ^ usize::from(bytes[0]);
    }
    checksum
}

/// Encodes every signed LEB128 fixture and returns a checksum.
fn encode_signed_leb128_values(values: &[u64]) -> usize {
    let mut checksum = 0_usize;
    let mut bytes = [0_u8; Leb128Codec::<i64, NonStrict>::MAX_UNITS_PER_VALUE];
    for &value in black_box(values) {
        let len = unsafe {
            Leb128Codec::<i64, NonStrict>::encode(value as i64, &mut bytes, 0)
        };
        checksum ^= len ^ usize::from(bytes[0]);
    }
    checksum
}

/// Encodes every ZigZag fixture and returns a checksum.
fn encode_zig_zag_values(values: &[u64]) -> usize {
    let mut checksum = 0_usize;
    let mut bytes = [0_u8; ZigZagCodec::<i64, NonStrict>::MAX_UNITS_PER_VALUE];
    for &value in black_box(values) {
        let len = unsafe {
            ZigZagCodec::<i64, NonStrict>::encode(value as i64, &mut bytes, 0)
        };
        checksum ^= len ^ usize::from(bytes[0]);
    }
    checksum
}

/// Benchmarks direct fixed-width big-endian integer encoding and decoding.
fn bench_binary(c: &mut Criterion) {
    let values = values();
    let mut group = c.benchmark_group("binary_codec");
    group.throughput(Throughput::Elements(BATCH_SIZE as u64));
    group.bench_function("u64_big_endian_roundtrip", |bencher| {
        bencher.iter(|| {
            let mut checksum = 0_u64;
            let mut bytes =
                [0_u8; BinaryCodec::<u64, BigEndian>::MAX_UNITS_PER_VALUE];
            for &value in black_box(&values) {
                unsafe {
                    BinaryCodec::<u64, BigEndian>::encode(value, &mut bytes, 0);
                }
                let (decoded, _) =
                    unsafe { BinaryCodec::<u64, BigEndian>::decode(&bytes, 0) };
                checksum ^= decoded;
            }
            black_box(checksum)
        });
    });
    group.bench_function("u64_big_endian_encode", |bencher| {
        bencher.iter(|| black_box(encode_binary_values(&values)));
    });
    group.finish();
}

/// Benchmarks canonical unsigned LEB128 decode policy overhead directly.
fn bench_leb128(c: &mut Criterion) {
    let values = values();
    let payloads = unsigned_payloads(&values);
    let signed_payloads = signed_leb128_payloads(&values);
    let mut group = c.benchmark_group("leb128_codec");
    group.throughput(Throughput::Elements(BATCH_SIZE as u64));
    group.bench_with_input(
        BenchmarkId::new("u64_decode", "non_strict"),
        &payloads,
        |bencher, payloads| {
            bencher.iter(|| {
                black_box(decode_unsigned_payloads::<NonStrict>(payloads))
            })
        },
    );
    group.bench_function("u64_encode", |bencher| {
        bencher.iter(|| black_box(encode_unsigned_leb128_values(&values)));
    });
    group.bench_with_input(
        BenchmarkId::new("i64_decode", "non_strict"),
        &signed_payloads,
        |bencher, payloads| {
            bencher.iter(|| {
                black_box(decode_signed_leb128_payloads::<NonStrict>(payloads))
            })
        },
    );
    group.bench_with_input(
        BenchmarkId::new("i64_decode", "strict"),
        &signed_payloads,
        |bencher, payloads| {
            bencher.iter(|| {
                black_box(decode_signed_leb128_payloads::<Strict>(payloads))
            })
        },
    );
    group.bench_function("i64_encode", |bencher| {
        bencher.iter(|| black_box(encode_signed_leb128_values(&values)));
    });
    group.bench_with_input(
        BenchmarkId::new("u64_decode", "strict"),
        &payloads,
        |bencher, payloads| {
            bencher.iter(|| {
                black_box(decode_unsigned_payloads::<Strict>(payloads))
            })
        },
    );
    group.finish();
}

/// Benchmarks canonical ZigZag decode policy overhead directly.
fn bench_zig_zag(c: &mut Criterion) {
    let values = values();
    let payloads = signed_payloads(&values);
    let mut group = c.benchmark_group("zig_zag_codec");
    group.throughput(Throughput::Elements(BATCH_SIZE as u64));
    group.bench_with_input(
        BenchmarkId::new("i64_decode", "non_strict"),
        &payloads,
        |bencher, payloads| {
            bencher.iter(|| {
                black_box(decode_signed_payloads::<NonStrict>(payloads))
            })
        },
    );
    group.bench_function("i64_encode", |bencher| {
        bencher.iter(|| black_box(encode_zig_zag_values(&values)));
    });
    group.bench_with_input(
        BenchmarkId::new("i64_decode", "strict"),
        &payloads,
        |bencher, payloads| {
            bencher
                .iter(|| black_box(decode_signed_payloads::<Strict>(payloads)))
        },
    );
    group.finish();
}

criterion_group!(codec_benches, bench_binary, bench_leb128, bench_zig_zag);
criterion_main!(codec_benches);
