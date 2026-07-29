// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec_binary::{
    Leb128DecodePolicy,
    NonStrict,
    Strict,
};

/// Requires a type to be one of the crate's sealed LEB128 policy markers.
fn require_sealed_policy<P>()
where
    P: Leb128DecodePolicy,
{
}

#[test]
fn test_sealed_policy_trait_is_implemented_by_builtin_markers() {
    require_sealed_policy::<NonStrict>();
    require_sealed_policy::<Strict>();
}
