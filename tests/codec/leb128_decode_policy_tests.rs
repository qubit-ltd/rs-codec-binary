// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec_binary::Leb128DecodePolicy;
use qubit_codec_binary::NonStrict;
use qubit_codec_binary::Strict;

/// Returns the canonicality behavior of a built-in LEB128 policy marker.
fn is_strict<P>() -> bool
where
    P: Leb128DecodePolicy,
{
    P::STRICT
}

#[test]
fn test_leb128_decode_policy_reports_builtin_strictness() {
    assert!(!is_strict::<NonStrict>());
    assert!(is_strict::<Strict>());
}
