// SPDX-License-Identifier: CC0-1.0

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {

    // alaw
    c.bench_function("decode_alaw", |b| b.iter(|| {
        for i in 0..255 {
            audio_codec_algorithms::decode_alaw(black_box(i));
        }
    }));
    c.bench_function("encode_alaw", |b| b.iter(|| {
        for i in -32768..32767 {
            audio_codec_algorithms::encode_alaw(black_box(i));
        }
    }));

    // ulaw
    c.bench_function("decode_ulaw", |b| b.iter(|| {
        for i in 0..255 {
            audio_codec_algorithms::decode_ulaw(black_box(i));
        }
    }));
    c.bench_function("encode_ulaw", |b| b.iter(|| {
        for i in -32768..32767 {
            audio_codec_algorithms::encode_ulaw(black_box(i));
        }
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
