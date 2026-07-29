// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec_binary::Leb128DecodeErrorKind;

#[test]
fn test_display_formats_decode_error_kinds() {
    assert_eq!(
        "incomplete LEB128 integer",
        Leb128DecodeErrorKind::Incomplete.to_string()
    );
    assert_eq!(
        "malformed LEB128 integer",
        Leb128DecodeErrorKind::Malformed.to_string()
    );
    assert_eq!(
        "non-canonical LEB128 integer",
        Leb128DecodeErrorKind::NonCanonical.to_string()
    );
}
