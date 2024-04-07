
# Audio Codec Algorithms

[![Cross-platform tests](https://github.com/karip/audio-codec-algorithms/actions/workflows/cross-test.yml/badge.svg)](https://github.com/karip/audio-codec-algorithms/actions/workflows/cross-test.yml)

Audio decoding and encoding for few basic codecs implemented in Rust. Supported codecs are:

 - [G.711 A-law](https://en.wikipedia.org/wiki/G.711#A-law)
 - [G.711 μ-law](https://en.wikipedia.org/wiki/G.711#μ-law)

Features:

 - supports no_std
 - no heap memory allocations
 - no unsafe code
 - no panicking
 - only dependencies for testing: no-panic and criterion

## Running the example

Try out decoding and encoding values:

    cargo run --example codec-tester decode ulaw 3 130 221
    cargo run --example codec-tester encode alaw 10 -5430 3263

## Running tests

Run:

    # run the tests
    cargo test
    # ensure good code quality
    cargo clippy
    # ensure that the release build never panics
    cargo test --release --features internal-no-panic

Performance testing:

    cargo bench

## License

Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
