use core::num::NonZeroUsize;

mod binary_codec_tests;
mod decode_policy_tests;
mod leb128_codec_tests;
mod leb128_decode_error_kind_tests;
mod leb128_decode_error_tests;
mod non_strict_tests;
mod strict_tests;
mod zig_zag_codec_tests;

/// Compares a decoded value and its non-zero consumed unit count.
fn assert_decoded_eq<T>(expected: (T, usize), actual: (T, NonZeroUsize))
where
    T: core::fmt::Debug + PartialEq,
{
    let (expected_value, expected_consumed) = expected;
    let (actual_value, actual_consumed) = actual;
    assert_eq!(expected_value, actual_value);
    assert_eq!(expected_consumed, actual_consumed.get());
}
