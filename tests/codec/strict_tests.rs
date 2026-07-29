// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec_binary::Strict;

#[test]
fn test_strict_is_copyable_default_marker() {
    let marker = Strict;

    assert_eq!(marker, Strict);
}
