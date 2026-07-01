use qubit_codec_binary::{
    Leb128DecodePolicy,
    NonStrict,
    Strict,
};

fn is_strict<P: Leb128DecodePolicy>() -> bool {
    P::STRICT
}

#[test]
fn test_leb128_decode_policy_exposes_strict_flag() {
    assert!(is_strict::<Strict>());
    assert!(!is_strict::<NonStrict>());
}
