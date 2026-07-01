use core::num::NonZeroUsize;

use qubit_codec_binary::{
    Leb128DecodeError,
    Leb128DecodeErrorKind,
};

#[test]
fn test_incomplete_stores_start_error_required_available_and_additional_units()
{
    let required = qubit_io::nz!(3);
    let error = Leb128DecodeError::incomplete(5, required, 2);

    assert_eq!(Leb128DecodeErrorKind::Incomplete, error.kind());
    assert!(error.is_incomplete());
    assert_eq!(5, error.start_index());
    assert_eq!(7, error.error_index());
    assert_eq!(None, error.consumed());
    assert_eq!(Some(required), error.required());
    assert_eq!(Some(2), error.available());
    assert_eq!(NonZeroUsize::new(1), error.additional());
    assert_eq!(
        "incomplete LEB128 integer at byte 5: need at least 3 bytes, only 2 available (next byte boundary 7)",
        error.to_string(),
    );
}

#[test]
fn test_incomplete_rejects_satisfied_required_bound() {
    let required = qubit_io::nz!(2);

    let result = std::panic::catch_unwind(|| {
        Leb128DecodeError::incomplete(5, required, 2)
    });

    assert!(result.is_err());
}

#[test]
fn test_incomplete_rejects_error_boundary_overflow() {
    let required = qubit_io::nz!(2);

    let result = std::panic::catch_unwind(|| {
        Leb128DecodeError::incomplete(usize::MAX, required, 1)
    });

    assert!(result.is_err());
}

#[test]
fn test_invalid_errors_store_start_error_and_consumed_units() {
    let malformed_consumed = qubit_io::nz!(4);
    let noncanonical_consumed = qubit_io::nz!(2);
    let malformed = Leb128DecodeError::malformed(5, 7, malformed_consumed);
    let noncanonical =
        Leb128DecodeError::noncanonical(9, noncanonical_consumed);

    assert!(malformed.is_malformed());
    assert!(noncanonical.is_noncanonical());
    assert_eq!(5, malformed.start_index());
    assert_eq!(7, malformed.error_index());
    assert_eq!(9, noncanonical.start_index());
    assert_eq!(10, noncanonical.error_index());
    assert_eq!(Some(malformed_consumed), malformed.consumed());
    assert_eq!(Some(noncanonical_consumed), noncanonical.consumed());
    assert_eq!(None, malformed.required());
    assert_eq!(None, malformed.additional());
    assert_eq!(None, noncanonical.available());
    assert_eq!(
        "malformed LEB128 integer at byte 5: detected at byte 7 after consuming 4 bytes",
        malformed.to_string(),
    );
    assert_eq!(
        "non-canonical LEB128 integer at byte 9: detected at byte 10 after consuming 2 bytes",
        noncanonical.to_string(),
    );
}

#[test]
fn test_malformed_rejects_error_index_outside_consumed_span() {
    let consumed = qubit_io::nz!(2);

    let before_start = std::panic::catch_unwind(|| {
        Leb128DecodeError::malformed(5, 4, consumed)
    });
    let after_consumed = std::panic::catch_unwind(|| {
        Leb128DecodeError::malformed(5, 7, consumed)
    });

    assert!(before_start.is_err());
    assert!(after_consumed.is_err());
}
