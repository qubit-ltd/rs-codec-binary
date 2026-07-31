// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec_binary::Strict;

/// Requires a marker type with the traits promised by the public API.
fn require_copy_default<T>()
where
    T: Copy + Default,
{
}

#[test]
fn test_strict_is_copyable_default_marker() {
    let marker = Strict;
    let copied = marker;

    require_copy_default::<Strict>();
    assert_eq!(marker, copied);
    assert_eq!(Strict, Strict);
}
