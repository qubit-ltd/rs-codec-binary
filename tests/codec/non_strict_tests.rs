use qubit_codec_binary::NonStrict;

#[test]
fn test_non_strict_is_copyable_default_marker() {
    let marker = NonStrict;

    assert_eq!(marker, NonStrict);
}
