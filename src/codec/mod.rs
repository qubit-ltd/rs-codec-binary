/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/

mod binary_codec;
mod leb128_codec;
mod leb128_decode_error;
mod leb128_decode_error_kind;
mod leb128_decode_policy;
mod non_strict;
mod sealed;
mod strict;
mod zig_zag_codec;

pub use binary_codec::BinaryCodec;
pub use leb128_codec::Leb128Codec;
pub use leb128_decode_error::Leb128DecodeError;
pub use leb128_decode_error_kind::Leb128DecodeErrorKind;
pub use leb128_decode_policy::Leb128DecodePolicy;
pub use non_strict::NonStrict;
pub use strict::Strict;
pub use zig_zag_codec::ZigZagCodec;
