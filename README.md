
# Audio Codec Algorithms

[![Cross-platform tests](https://github.com/karip/audio-codec-algorithms/actions/workflows/cross-test.yml/badge.svg)](https://github.com/karip/audio-codec-algorithms/actions/workflows/cross-test.yml)

Simple audio codecs implemented in Rust. Supported codecs are:

 - [G.711 A-law](https://en.wikipedia.org/wiki/G.711#A-law)
 - [G.711 μ-law](https://en.wikipedia.org/wiki/G.711#μ-law)

Features:

 - supports no_std
 - no heap memory allocations
 - no unsafe code
 - no panicking
 - no dependencies (except dev-dependency to criterion for benchmarking)

## Running the example

Try out decoding and encoding:

    cargo run --example codec-tester decode ulaw 3 130 221

    cargo run --example codec-tester encode alaw 10 -5430 3263

## Running tests

Run:

    cargo test
    cargo clippy

Performance testing (requires Rust nightly):

    cargo bench

## License

Public Domain. Creative Commons Zero (CC0-1.0).
