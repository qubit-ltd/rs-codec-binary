use qubit_codec::DecodeErrorInfo;
use qubit_codec_binary::{
    Leb128DecodeError,
    Leb128DecodeErrorKind,
};

#[test]
fn test_new_stores_kind_and_index() {
    let error = Leb128DecodeError::new(Leb128DecodeErrorKind::Malformed, 3);

    assert_eq!(Leb128DecodeErrorKind::Malformed, error.kind());
    assert_eq!(3, error.index());
    assert_eq!(Some(1), error.consumed());
    assert_eq!(None, error.required());
    assert_eq!(None, error.available());
    assert_eq!("malformed LEB128 integer", error.to_string());
}

#[test]
fn test_incomplete_stores_required_and_available_units() {
    let error = Leb128DecodeError::incomplete(5, 3, 2);

    assert_eq!(Leb128DecodeErrorKind::Incomplete, error.kind());
    assert_eq!(5, error.index());
    assert_eq!(None, error.consumed());
    assert_eq!(Some(3), error.required());
    assert_eq!(Some(2), error.available());
    assert_eq!(Some((3, 2)), error.failure().incomplete());
}

#[test]
fn test_invalid_errors_store_consumed_units() {
    let malformed = Leb128DecodeError::malformed(7, 4);
    let noncanonical = Leb128DecodeError::noncanonical(9, 2);

    assert_eq!(Some(4), malformed.consumed());
    assert_eq!(Some(4), malformed.failure().invalid_consumed());
    assert_eq!(Some(2), noncanonical.consumed());
    assert_eq!(Some(2), noncanonical.failure().invalid_consumed());
}
