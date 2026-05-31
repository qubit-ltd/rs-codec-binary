/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
use core::num::NonZeroUsize;

/// Compares a decoded value and its non-zero consumed unit count.
pub(super) fn assert_decoded_eq<T>(expected: (T, usize), actual: (T, NonZeroUsize))
where
    T: core::fmt::Debug + PartialEq,
{
    let (expected_value, expected_consumed) = expected;
    let (actual_value, actual_consumed) = actual;
    assert_eq!(expected_value, actual_value);
    assert_eq!(expected_consumed, actual_consumed.get());
}
