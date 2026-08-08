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

/// Requires a type implementing the crate's LEB128 policy contract.
fn require_sealed_policy<P>()
where
    P: Leb128DecodePolicy,
{
}

#[test]
fn test_builtin_markers_implement_leb128_decode_policy() {
    require_sealed_policy::<NonStrict>();
    require_sealed_policy::<Strict>();
}
