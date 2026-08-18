
#[cfg(feature = "internal-no-panic")]
use no_panic::no_panic;

// decoding table generated using G.191 softare tools at https://www.itu.int/rec/T-REC-G.191/en
const ULAW_VALUES: &[i16; 256] = &[
    -32124, -31100, -30076, -29052, -28028, -27004, -25980, -24956,
    -23932, -22908, -21884, -20860, -19836, -18812, -17788, -16764,
    -15996, -15484, -14972, -14460, -13948, -13436, -12924, -12412,
    -11900, -11388, -10876, -10364, -9852, -9340, -8828, -8316,
    -7932, -7676, -7420, -7164, -6908, -6652, -6396, -6140,
    -5884, -5628, -5372, -5116, -4860, -4604, -4348, -4092,
    -3900, -3772, -3644, -3516, -3388, -3260, -3132, -3004,
    -2876, -2748, -2620, -2492, -2364, -2236, -2108, -1980,
    -1884, -1820, -1756, -1692, -1628, -1564, -1500, -1436,
    -1372, -1308, -1244, -1180, -1116, -1052, -988, -924,
    -876, -844, -812, -780, -748, -716, -684, -652,
    -620, -588, -556, -524, -492, -460, -428, -396,
    -372, -356, -340, -324, -308, -292, -276, -260,
    -244, -228, -212, -196, -180, -164, -148, -132,
    -120, -112, -104, -96, -88, -80, -72, -64,
    -56, -48, -40, -32, -24, -16, -8, 0,
    32124, 31100, 30076, 29052, 28028, 27004, 25980, 24956,
    23932, 22908, 21884, 20860, 19836, 18812, 17788, 16764,
    15996, 15484, 14972, 14460, 13948, 13436, 12924, 12412,
    11900, 11388, 10876, 10364, 9852, 9340, 8828, 8316,
    7932, 7676, 7420, 7164, 6908, 6652, 6396, 6140,
    5884, 5628, 5372, 5116, 4860, 4604, 4348, 4092,
    3900, 3772, 3644, 3516, 3388, 3260, 3132, 3004,
    2876, 2748, 2620, 2492, 2364, 2236, 2108, 1980,
    1884, 1820, 1756, 1692, 1628, 1564, 1500, 1436,
    1372, 1308, 1244, 1180, 1116, 1052, 988, 924,
    876, 844, 812, 780, 748, 716, 684, 652,
    620, 588, 556, 524, 492, 460, 428, 396,
    372, 356, 340, 324, 308, 292, 276, 260,
    244, 228, 212, 196, 180, 164, 148, 132,
    120, 112, 104, 96, 88, 80, 72, 64,
    56, 48, 40, 32, 24, 16, 8, 0,
];

/// Decodes a 8-bit encoded G.711 μ-law value to a linear 16-bit signed integer sample value.
#[cfg_attr(feature = "internal-no-panic", no_panic)]
#[inline(always)]
pub fn decode_ulaw(encoded: u8) -> i16 {
    ULAW_VALUES[usize::from(encoded)]
}

// encoding algorithm is based on "A-Law and mu-Law Companding Implementations Using the TMS320C54x,
// Application Note: SPRA163A", pages 18 and 20: https://www.ti.com/lit/an/spra163a/spra163a.pdf
// the segment (chord) is the position of the highest set bit, so counting leading zeros gives it
// directly, without the binary encoding table on page 13. the application note does the same with
// the EXP instruction, which "allows the extraction of the most significant bits without requiring
// a look-up table" (page 18), giving mchd = (0x19 - T|EXP) << 4 on page 20
// see also https://en.wikipedia.org/wiki/G.711#μ-law

/// Encodes a linear 16-bit signed integer sample value to a 8-bit encoded G.711 μ-law value.
#[cfg_attr(feature = "internal-no-panic", no_panic)]
#[inline(always)]
pub fn encode_ulaw(linear: i16) -> u8 {
    let sign = if linear >= 0 {
        0x00
    } else {
        0x80
    };
    #[allow(clippy::cast_sign_loss)] // sign loss is expected and handled after the cast to u16
    let linear = (linear >> 2) as u16;
    let absval = u32::from(if sign == 0x80 {
        // make a positive value using 1s' complement (a tip from wikipedia)
        linear ^ 0xffff
    } else {
        linear
    });
    let inputval = absval + 33;
    // the number of bits inputval has beyond the first segment, which ends at 0b0000000111111
    let segment = (u32::BITS - inputval.leading_zeros()).saturating_sub(6);
    let compressed_code_word: u32 = if segment >= 8 {
        // inputval is past the last segment
        0b111_1111
    } else {
        (segment << 4) | ((inputval >> (segment + 1)) & 0b1111)
    };
    #[allow(clippy::cast_possible_truncation)] // compressed_code_word is always less than 255
    let result = (sign | compressed_code_word as u8) ^ 0xff;
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_ulaw() {
        // no need to test all values because they are read from a static table
        assert_eq!(decode_ulaw(0), -32124);
        assert_eq!(decode_ulaw(128), 32124);
        assert_eq!(decode_ulaw(255), 0);
    }

    #[test]
    fn test_encode_ulaw() {
        // test against reference values generated for all input values -32768..=32767
        // the reference values were generated with the G.191 software tools
        let buffer = include_bytes!("../test-files/ulaw-reference.bin");
        let mut bi = 0;
        for i in -32768..=32767 {
            let encoded = encode_ulaw(i);
            assert_eq!(buffer[bi], encoded);
            bi += 1;
        }
    }
}
