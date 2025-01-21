
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {

    // alaw
    c.bench_function("decode_alaw", |b| b.iter(|| {
        for i in 0..=255 {
            black_box(audio_codec_algorithms::decode_alaw(black_box(i)));
        }
    }));
    c.bench_function("encode_alaw", |b| b.iter(|| {
        for i in -32768..=32767 {
            black_box(audio_codec_algorithms::encode_alaw(black_box(i)));
        }
    }));

    // ulaw
    c.bench_function("decode_ulaw", |b| b.iter(|| {
        for i in 0..=255 {
            black_box(audio_codec_algorithms::decode_ulaw(black_box(i)));
        }
    }));
    c.bench_function("encode_ulaw", |b| b.iter(|| {
        for i in -32768..=32767 {
            black_box(audio_codec_algorithms::encode_ulaw(black_box(i)));
        }
    }));

    // adpcm ima
    c.bench_function("decode_adpcm_ima", |b| b.iter(|| {
        let mut state = audio_codec_algorithms::AdpcmImaState::new();
        // only the lowest 4 bits are used by decode_adpcm_ima(), but lets loop over the entire
        // value space [0, 255] to see that it works and so that the time can be compared
        // to other decode benchmarks
        for i in 0..=255 {
            black_box(
                audio_codec_algorithms::decode_adpcm_ima(black_box(i), &mut state));
        }
    }));
    c.bench_function("encode_adpcm_ima", |b| b.iter(|| {
        let mut state = audio_codec_algorithms::AdpcmImaState::new();
        for i in -32768..=32767 {
            black_box(
                audio_codec_algorithms::encode_adpcm_ima(black_box(i), &mut state));
        }
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
