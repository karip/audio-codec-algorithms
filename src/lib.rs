//!
//! This crate contains simple audio codecs. Supported codecs are:
//!  - [G.711 A-law](https://en.wikipedia.org/wiki/G.711#A-law)
//!  - [G.711 μ-law](https://en.wikipedia.org/wiki/G.711#μ-law)
//!  - [IMA ADPCM](https://en.wikipedia.org/wiki/Interactive_Multimedia_Association)
//!

#![no_std]

#![forbid(
    unsafe_code,
    clippy::panic,
    clippy::exit,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unimplemented,
    clippy::todo,
    clippy::unreachable,
)]
#![deny(
    clippy::cast_ptr_alignment,
    clippy::char_lit_as_u8,
    clippy::unnecessary_cast,
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::checked_conversions,
)]
#![allow(clippy::manual_range_contains)]

mod alaw;
pub use alaw::{decode_alaw, encode_alaw};

mod ulaw;
pub use ulaw::{decode_ulaw, encode_ulaw};

mod adpcm_ima;
pub use adpcm_ima::AdpcmImaState;
pub use adpcm_ima::{decode_adpcm_ima, decode_adpcm_ima_ima4, decode_adpcm_ima_ms};
pub use adpcm_ima::{encode_adpcm_ima, encode_adpcm_ima_ima4, encode_adpcm_ima_ms};

/// Error values.
#[derive(Debug)]
pub enum Error {
    /// Invalid number of channels.
    InvalidChannels,

    /// Buffer has an invalid size.
    InvalidBufferSize,
}
