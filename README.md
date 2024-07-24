
# Audio Codec Algorithms

[![Cross-platform tests](https://github.com/karip/audio-codec-algorithms/actions/workflows/cross-test.yml/badge.svg)](https://github.com/karip/audio-codec-algorithms/actions/workflows/cross-test.yml)

Decoding and encoding for few basic audio codecs implemented in Rust:

 - [G.711 A-law](https://en.wikipedia.org/wiki/G.711#A-law)
 - [G.711 μ-law](https://en.wikipedia.org/wiki/G.711#μ-law)
 - [IMA ADPCM](https://en.wikipedia.org/wiki/Interactive_Multimedia_Association)

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
    cargo run --example codec-tester encode adpcm_ima 25 40 60 80 100 160 220
    cargo run --example codec-tester decode adpcm_ima 7 7 2 2 2 7 5

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

There is a GitHub Action called "Cross-platform tests" (cross-test.yml), which automatically
runs `cargo test` for little-endian 64-bit x64_86 and big-endian 32-bit PowerPC.

## License

Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
