#[cfg(feature = "internal-no-panic")]
use no_panic::no_panic;

// conversion table based on the table 3/G.711 in ITU-T Recommendation G.711, page 7.
// (https://www.itu.int/rec/T-REC-G.711 / "G.711 (11/88)")
// see the `test_convert_ulaw_to_alaw()` function for the algorithm to calculate these values.
const ULAW_TO_ALAW: &[u8; 256] = &[
    42, 43, 40, 41, 46, 47, 44, 45, 34, 35, 32, 33, 38, 39, 36, 37,
    58, 59, 56, 57, 62, 63, 60, 61, 50, 51, 48, 49, 54, 55, 52, 53,
    10, 11, 8, 9, 14, 15, 12, 13, 2, 3, 0, 1, 6, 7, 4, 5,
    27, 24, 25, 30, 31, 28, 29, 18, 19, 16, 17, 22, 23, 20, 21, 106,
    104, 105, 110, 111, 108, 109, 98, 99, 96, 97, 102, 103, 100, 101, 122, 120,
    126, 127, 124, 125, 114, 115, 112, 113, 118, 119, 116, 117, 75, 73, 79, 77,
    66, 67, 64, 65, 70, 71, 68, 69, 90, 91, 88, 89, 94, 95, 92, 93,
    82, 82, 83, 83, 80, 80, 81, 81, 86, 86, 87, 87, 84, 84, 85, 85,
    170, 171, 168, 169, 174, 175, 172, 173, 162, 163, 160, 161, 166, 167, 164, 165,
    186, 187, 184, 185, 190, 191, 188, 189, 178, 179, 176, 177, 182, 183, 180, 181,
    138, 139, 136, 137, 142, 143, 140, 141, 130, 131, 128, 129, 134, 135, 132, 133,
    155, 152, 153, 158, 159, 156, 157, 146, 147, 144, 145, 150, 151, 148, 149, 234,
    232, 233, 238, 239, 236, 237, 226, 227, 224, 225, 230, 231, 228, 229, 250, 248,
    254, 255, 252, 253, 242, 243, 240, 241, 246, 247, 244, 245, 203, 201, 207, 205,
    194, 195, 192, 193, 198, 199, 196, 197, 218, 219, 216, 217, 222, 223, 220, 221,
    210, 210, 211, 211, 208, 208, 209, 209, 214, 214, 215, 215, 212, 212, 213, 213,
];

/// Converts a 8-bit encoded G.711 μ-law value to a 8-bit encoded G.711 A-law value.
///
/// The conversion uses a 256 byte look-up-table, which is based on the ITU-T Recommendation
/// G.711 (11/88) conversion tables.
///
/// This conversion is not the same as calling `encode_alaw(decode_ulaw(encoded))` because
/// the G.711 Recommendation defines this conversion with some special properties,
/// such as transparency for bits 1-7 in double conversions (μ-A-μ or A-μ-A).
#[cfg_attr(feature = "internal-no-panic", no_panic)]
#[inline(always)]
pub fn convert_ulaw_to_alaw(encoded: u8) -> u8 {
    ULAW_TO_ALAW[usize::from(encoded)]
}

// conversion table based on the table 4/G.711 in ITU-T Recommendation G.711, page 9.
// (https://www.itu.int/rec/T-REC-G.711 / "G.711 (11/88)")
// see the `test_convert_alaw_to_ulaw()` function for the algorithm to calculate these values.
const ALAW_TO_ULAW: &[u8; 256] = &[
    42, 43, 40, 41, 46, 47, 44, 45, 34, 35, 32, 33, 38, 39, 36, 37,
    57, 58, 55, 56, 61, 62, 59, 60, 49, 50, 48, 48, 53, 54, 51, 52,
    10, 11, 8, 9, 14, 15, 12, 13, 2, 3, 0, 1, 6, 7, 4, 5,
    26, 27, 24, 25, 30, 31, 28, 29, 18, 19, 16, 17, 22, 23, 20, 21,
    98, 99, 96, 97, 102, 103, 100, 101, 93, 93, 92, 92, 95, 95, 94, 94,
    116, 118, 112, 114, 124, 126, 120, 122, 106, 107, 104, 105, 110, 111, 108, 109,
    72, 73, 70, 71, 76, 77, 74, 75, 64, 65, 63, 63, 68, 69, 66, 67,
    86, 87, 84, 85, 90, 91, 88, 89, 79, 79, 78, 78, 82, 83, 80, 81,
    170, 171, 168, 169, 174, 175, 172, 173, 162, 163, 160, 161, 166, 167, 164, 165,
    185, 186, 183, 184, 189, 190, 187, 188, 177, 178, 176, 176, 181, 182, 179, 180,
    138, 139, 136, 137, 142, 143, 140, 141, 130, 131, 128, 129, 134, 135, 132, 133,
    154, 155, 152, 153, 158, 159, 156, 157, 146, 147, 144, 145, 150, 151, 148, 149,
    226, 227, 224, 225, 230, 231, 228, 229, 221, 221, 220, 220, 223, 223, 222, 222,
    244, 246, 240, 242, 252, 254, 248, 250, 234, 235, 232, 233, 238, 239, 236, 237,
    200, 201, 198, 199, 204, 205, 202, 203, 192, 193, 191, 191, 196, 197, 194, 195,
    214, 215, 212, 213, 218, 219, 216, 217, 207, 207, 206, 206, 210, 211, 208, 209,
];

/// Converts a 8-bit encoded G.711 A-law value to a 8-bit encoded G.711 μ-law value.
///
/// The conversion uses a 256 byte look-up-table, which is based on the ITU-T Recommendation
/// G.711 (11/88) conversion tables.
///
/// This conversion is not the same as calling `encode_ulaw(decode_alaw(encoded))` because
/// the G.711 Recommendation defines this conversion with some special properties,
/// such as transparency for bits 1-7 in double conversions (μ-A-μ or A-μ-A).
#[cfg_attr(feature = "internal-no-panic", no_panic)]
#[inline(always)]
pub fn convert_alaw_to_ulaw(encoded: u8) -> u8 {
    ALAW_TO_ULAW[usize::from(encoded)]
}

#[cfg(test)]
mod tests {
    use super::*;

    // variable names used in the tests:
    // - uencoded = u-law encoded value (character signal)
    // - unum = u-law decoder output value number
    // - aencoded = a-law encoded value (character signal after inversion of the even bits)
    // - anum = a-law decoder output value number

    /// Tests ulaw to alaw conversion by comparing it to the conversion table in the spec.
    #[test]
    fn test_convert_ulaw_to_alaw() {
        // tests the entire u-law range
        for uencoded in 0..=255u8 {
            // convert using the conversion table in the spec:
            // - convert ulaw encoded value to ulaw decoder output value number
            // - convert ulaw decoder output value number to alaw decoder output value number
            // - convert alaw decoder output value number to alaw encoded value
            let spec_unum = uencoded_to_unum(uencoded);
            let spec_anum = unum_to_anum(spec_unum);
            let spec_aencoded = anum_to_aencoded(spec_anum);
            // compare the look-up-table value to the spec value
            assert_eq!(convert_ulaw_to_alaw(uencoded), spec_aencoded);

            // enable this line and run `cargo test -- --nocapture` to generate ULAW_TO_ALAW
            //extern crate std; std::print!("{spec_aencoded}, ");
        }
    }

    /// Tests alaw to ulaw conversion by comparing it to the conversion table in the spec.
    #[test]
    fn test_convert_alaw_to_ulaw() {
        // tests the entire a-law range
        for aencoded in 0..=255u8 {
            // convert using the conversion table in the spec:
            // - convert alaw encoded value to alaw decoder output value number
            // - convert alaw decoder output value number to ulaw decoder output value number
            // - convert ulaw decoder output value number to ulaw encoded value
            let spec_anum = aencoded_to_anum(aencoded);
            let spec_unum = anum_to_unum(spec_anum);
            let spec_uencoded = unum_to_uencoded(spec_unum);
            // compare the look-up-table value to the spec value
            assert_eq!(convert_alaw_to_ulaw(aencoded), spec_uencoded);

            // enable this line and run `cargo test -- --nocapture` to generate ALAW_TO_ULAW
            //extern crate std; std::print!("{spec_uencoded}, ");
        }
    }

    /// Tests double conversion μ-A-μ.
    #[test]
    fn test_double_ulaw_to_alaw_to_ulaw() {
        // tests the entire u-law encoded value range
        for uencoded in 0..=255u8 {
            let unum = uencoded_to_unum(uencoded);
            let res_uencoded = convert_alaw_to_ulaw(convert_ulaw_to_alaw(uencoded));
            let res_unum = uencoded_to_unum(res_uencoded);
            // according to ITU-T Recommendation G.711, page 8 note 2:
            // "0, 2, 4, 6, 8, 10, 12, 14 are changed (the numbers being increased by 1)"
            if [ 0, 2, 4, 6, 8, 10, 12, 14 ].contains(&unum.number) {
                // check "(the numbers being increased by 1)"
                assert_eq!(res_unum.number, unum.number + 1);
                // check transparency for bits 1-7:
                // "in these octets, only bit No. 8 (least significant bit in PCM) is changed"
                assert_eq!(res_uencoded, uencoded ^ 0x01);
            } else {
                // not changed
                assert_eq!(res_unum.number, unum.number);
                assert_eq!(res_uencoded, uencoded);
            }
        }
    }

    /// Tests double conversion A-μ-A.
    #[test]
    fn test_double_alaw_to_ulaw_to_alaw() {
        // tests the entire a-law encoded value range
        for aencoded in 0..=255u8 {
            let anum = aencoded_to_anum(aencoded);
            let res_aencoded = convert_ulaw_to_alaw(convert_alaw_to_ulaw(aencoded));
            let res_anum = aencoded_to_anum(res_aencoded);
            // according to ITU-T Recommendation G.711, page 10 note 2:
            // "26, 28, 30, 32, 45, 47, 63 and 80 are changed"
            if [ 26, 28, 30, 32, 45, 47, 63, 80 ].contains(&anum.number) {
                // check transparency for bits 1-7: "Again, only bit 8 is changed"
                assert_eq!(res_aencoded, aencoded ^ 0x01);
            } else {
                // not changed
                assert_eq!(res_anum.number, anum.number);
                assert_eq!(res_aencoded, aencoded);
            }
        }
    }

    // helper structs and functions for tests

    /// Decoder output value number and its sign.
    #[derive(PartialEq, Debug)]
    struct Num {
        /// Sign of the number. 0x00 = positive, 0x80 = negative.
        /// This separate sign field ensures that the ulaw value number 0 can be positive or
        /// negative (spec table 2a or 2b).
        sign_bit: u8,
        /// Decoder output value number, for ulaw 0..=127, for alaw 1..=128.
        number: u8,
    }

    /// Converts ulaw encoded value 0..=255 to ulaw decoder output value number 0..=127.
    /// The `sign_bit` field indicates a positive (0x00) or negative value (0x80).
    fn uencoded_to_unum(uencoded: u8) -> Num {
        let u = uencoded ^ 0xff;
        Num {
            sign_bit: u & 0x80,
            number: u & 0x7f
        }
    }

    #[test]
    fn test_uencoded_to_unum() {
        // According to the spec table 2a column 6 (character signal), value 255 (0b1111_11111)
        // is the decoder output value number 0.
        assert_eq!(uencoded_to_unum(255), Num { sign_bit: 0x00, number: 0 });
        // these can be resolved the same way
        assert_eq!(uencoded_to_unum(254), Num { sign_bit: 0x00, number: 1 });
        assert_eq!(uencoded_to_unum(128), Num { sign_bit: 0x00, number: 127 });
        // the negative values use the spec table 2b, `sign_bit` indicates a negative value
        assert_eq!(uencoded_to_unum(0),Num { sign_bit: 0x80, number: 127 });
        assert_eq!(uencoded_to_unum(15), Num { sign_bit: 0x80, number: 112 });
        assert_eq!(uencoded_to_unum(127), Num { sign_bit: 0x80, number: 0 });
    }

    /// Converts ulaw decoder output value number 0..=127 with the sign bit to
    /// ulaw encoded value 0..=255.
    fn unum_to_uencoded(unum: Num) -> u8 {
        assert!(unum.number <= 127);
        (unum.sign_bit | unum.number) ^ 0xff
    }

    #[test]
    fn test_unum_to_uencoded() {
        // these are the values of `test_uencoded_to_unum()` reversed
        assert_eq!(unum_to_uencoded(Num { sign_bit: 0x00, number: 0 }), 255);
        assert_eq!(unum_to_uencoded(Num { sign_bit: 0x00, number: 1 }), 254);
        assert_eq!(unum_to_uencoded(Num { sign_bit: 0x00, number: 127 }), 128);
        assert_eq!(unum_to_uencoded(Num { sign_bit: 0x80, number: 127 }), 0);
        assert_eq!(unum_to_uencoded(Num { sign_bit: 0x80, number: 112 }), 15);
        assert_eq!(unum_to_uencoded(Num { sign_bit: 0x80, number: 0 }), 127);
    }

    /// Converts alaw encoded value 0..=255 to alaw decoder output value number 1..=128.
    /// The `sign_bit` field indicates a positive (0x00) or negative value (0x80).
    fn aencoded_to_anum(aencoded: u8) -> Num {
        let a = aencoded ^ 0xd5;
        Num {
            sign_bit: a & 0x80,
            number: (a & 0x7f) + 1,
        }
    }

    #[test]
    fn test_aencoded_to_anum() {
        // alaw encoded value 213 is even bit inverted (^ 0x55) to get the value 0b1000_0000.
        // According to the spec table 1a column 6 (character signal before inversion of
        // the even bits), value 0b1000_0000 is the decoder output value number 1.
        assert_eq!(aencoded_to_anum(213), Num { sign_bit: 0x00, number: 1 });
        // these can be resolved the same way
        assert_eq!(aencoded_to_anum(133), Num { sign_bit: 0x00, number: 81 });
        assert_eq!(aencoded_to_anum(170), Num { sign_bit: 0x00, number: 128 });
        // the negative values use the spec table 1b, `sign_bit` indicates a negative value
        assert_eq!(aencoded_to_anum(42), Num { sign_bit: 0x80, number: 128 });
        assert_eq!(aencoded_to_anum(53), Num { sign_bit: 0x80, number: 97 });
        assert_eq!(aencoded_to_anum(85), Num { sign_bit: 0x80, number: 1 });
    }

    /// Converts alaw decoder output value number 1..=128 with the sign bit to
    /// alaw encoded value 0..=255.
    fn anum_to_aencoded(anum: Num) -> u8 {
        assert!(1 <= anum.number && anum.number <= 128);
        (anum.sign_bit | (anum.number - 1)) ^ 0xd5
    }

    #[test]
    fn test_anum_to_aencoded() {
        // these are the values of `test_aencoded_to_anum()` reversed
        assert_eq!(anum_to_aencoded(Num { sign_bit: 0x00, number: 1 }), 213);
        assert_eq!(anum_to_aencoded(Num { sign_bit: 0x00, number: 81 }), 133);
        assert_eq!(anum_to_aencoded(Num { sign_bit: 0x00, number: 128 }), 170);
        assert_eq!(anum_to_aencoded(Num { sign_bit: 0x80, number: 128 }), 42);
        assert_eq!(anum_to_aencoded(Num { sign_bit: 0x80, number: 97 }), 53);
        assert_eq!(anum_to_aencoded(Num { sign_bit: 0x80, number: 1 }), 85);
    }

    /// Converts ulaw decoder output value number to alaw decoder output value number
    /// using the conversion table 3 in ITU-T Recommendation G.711 page 7.
    fn unum_to_anum(unum: Num) -> Num {
        const U_A_CONVERSION: &[u8; 128] = &[
            1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 10, 11, 12, 13, 14,
            15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 27, 29, 31, 33, 34, 35, 36,
            37, 38, 39, 40, 41, 42, 43, 44, 46, 48, 49, 50, 51, 52, 53, 54, 55, 56,
            57, 58, 59, 60, 61, 62, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75,
            76, 77, 78, 79, 81, 82, 83, 84, 85, 86, 87, 88,
            89, 90, 91, 92, 93, 94, 95, 96, 97, 98, 99, 100, 101, 102, 103, 104, 105,
            106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117, 118, 119, 120,
            121, 122, 123, 124, 125, 126, 127, 128
        ];
        assert!(unum.number <= 127);
        Num {
            sign_bit: unum.sign_bit,
            number: U_A_CONVERSION[usize::from(unum.number)]
        }
    }

    /// Converts alaw decoder output value number to ulaw decoder output value number
    /// using the conversion table 4 in ITU-T Recommendation G.711 page 9.
    fn anum_to_unum(anum: Num) -> Num {
        const A_U_CONVERSION: &[u8; 128] = &[
            1, 3, 5, 7, 9, 11, 13, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
            27, 28, 29, 30, 31, 32, 32, 33, 33, 34, 34, 35, 35, 36, 37, 38, 39, 40,
            41, 42, 43, 44, 45, 46, 47, 48, 48, 49, 49, 50, 51, 52, 53, 54, 55, 56,
            57, 58, 59, 60, 61, 62, 63, 64, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73,
            74, 75, 76, 77, 78, 79, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90,
            91, 92, 93, 94, 95, 96, 97,
            98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113,
            114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127
        ];
        assert!(1 <= anum.number && anum.number <= 128);
        Num {
            sign_bit: anum.sign_bit,
            number: A_U_CONVERSION[usize::from(anum.number-1)],
        }
    }
}
