
#[cfg(feature = "internal-no-panic")]
use no_panic::no_panic;

// decoding table generated using G.191 softare tools at https://www.itu.int/rec/T-REC-G.191/en
const ALAW_VALUES: &[i16; 256] = &[
    -5504, -5248, -6016, -5760, -4480, -4224, -4992, -4736,
    -7552, -7296, -8064, -7808, -6528, -6272, -7040, -6784,
    -2752, -2624, -3008, -2880, -2240, -2112, -2496, -2368,
    -3776, -3648, -4032, -3904, -3264, -3136, -3520, -3392,
    -22016, -20992, -24064, -23040, -17920, -16896, -19968, -18944,
    -30208, -29184, -32256, -31232, -26112, -25088, -28160, -27136,
    -11008, -10496, -12032, -11520, -8960, -8448, -9984, -9472,
    -15104, -14592, -16128, -15616, -13056, -12544, -14080, -13568,
    -344, -328, -376, -360, -280, -264, -312, -296,
    -472, -456, -504, -488, -408, -392, -440, -424,
    -88, -72, -120, -104, -24, -8, -56, -40,
    -216, -200, -248, -232, -152, -136, -184, -168,
    -1376, -1312, -1504, -1440, -1120, -1056, -1248, -1184,
    -1888, -1824, -2016, -1952, -1632, -1568, -1760, -1696,
    -688, -656, -752, -720, -560, -528, -624, -592,
    -944, -912, -1008, -976, -816, -784, -880, -848,
    5504, 5248, 6016, 5760, 4480, 4224, 4992, 4736,
    7552, 7296, 8064, 7808, 6528, 6272, 7040, 6784,
    2752, 2624, 3008, 2880, 2240, 2112, 2496, 2368,
    3776, 3648, 4032, 3904, 3264, 3136, 3520, 3392,
    22016, 20992, 24064, 23040, 17920, 16896, 19968, 18944,
    30208, 29184, 32256, 31232, 26112, 25088, 28160, 27136,
    11008, 10496, 12032, 11520, 8960, 8448, 9984, 9472,
    15104, 14592, 16128, 15616, 13056, 12544, 14080, 13568,
    344, 328, 376, 360, 280, 264, 312, 296,
    472, 456, 504, 488, 408, 392, 440, 424,
    88, 72, 120, 104, 24, 8, 56, 40,
    216, 200, 248, 232, 152, 136, 184, 168,
    1376, 1312, 1504, 1440, 1120, 1056, 1248, 1184,
    1888, 1824, 2016, 1952, 1632, 1568, 1760, 1696,
    688, 656, 752, 720, 560, 528, 624, 592,
    944, 912, 1008, 976, 816, 784, 880, 848,
];

/// Decodes a 8-bit encoded G.711 A-law value to a linear 16-bit signed integer sample value.
#[cfg_attr(feature = "internal-no-panic", no_panic)]
#[inline(always)]
pub fn decode_alaw(encoded: u8) -> i16 {
    ALAW_VALUES[usize::from(encoded)]
}

// encoding algorithm is based on "A-Law and mu-Law Companding Implementations Using the TMS320C54x,
// Application Note: SPRA163A", page 16: https://www.ti.com/lit/an/spra163a/spra163a.pdf
// see also https://en.wikipedia.org/wiki/G.711#A-law

/// Encodes a linear 16-bit signed integer sample value to a 8-bit encoded G.711 A-law value.
#[cfg_attr(feature = "internal-no-panic", no_panic)]
#[inline(always)]
pub fn encode_alaw(linear: i16) -> u8 {
    let sign = if linear >= 0 {
        0x00
    } else {
        0x80
    };
    #[allow(clippy::cast_sign_loss)] // sign loss is expected and handled after the cast to u16
    let linear = (linear >> 3) as u16;
    let inputval = if sign == 0x80 {
        // make a positive value using 1s' complement (a tip from wikipedia)
        linear ^ 0xffff
    } else {
        linear
    };
    let compressed_code_word: u16 = match inputval {
        #[allow(clippy::identity_op)]
        0b000000000000..=0b000000011111 => 0b000_0000 | (inputval & 0b000000011110) >> 1,
        0b000000100000..=0b000000111111 => 0b001_0000 | (inputval & 0b000000011110) >> 1,
        0b000001000000..=0b000001111111 => 0b010_0000 | (inputval & 0b000000111100) >> 2,
        0b000010000000..=0b000011111111 => 0b011_0000 | (inputval & 0b000001111000) >> 3,
        0b000100000000..=0b000111111111 => 0b100_0000 | (inputval & 0b000011110000) >> 4,
        0b001000000000..=0b001111111111 => 0b101_0000 | (inputval & 0b000111100000) >> 5,
        0b010000000000..=0b011111111111 => 0b110_0000 | (inputval & 0b001111000000) >> 6,
        0b100000000000..=0b111111111111 => 0b111_0000 | (inputval & 0b011110000000) >> 7,
        4096.. => 0b111_1111
    };
    #[allow(clippy::cast_possible_truncation)] // compressed_code_word is always less than 255
    let result = (sign | compressed_code_word as u8) ^ 0xd5;
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_alaw() {
        // no need to test all values because they are read from a static table
        assert_eq!(decode_alaw(0), -5504);
        assert_eq!(decode_alaw(128), 5504);
        assert_eq!(decode_alaw(255), 848);
    }

    #[test]
    fn test_encode_alaw() {
        // test against reference values generated for all input values -32768..=32767
        // the reference values were generated with the G.191 software tools
        let buffer = include_bytes!("../test-files/alaw-reference.bin");
        let mut bi = 0;
        for i in -32768..=32767 {
            let encoded = encode_alaw(i);
            assert_eq!(buffer[bi], encoded);
            bi += 1;
        }
    }
}
