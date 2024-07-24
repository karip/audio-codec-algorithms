
#[cfg(feature = "internal-no-panic")]
use no_panic::no_panic;

use crate::Error;

/// State values for the IMA ADPCM encoder and decoder.
///
/// The values should be initialized to zeros or to values from the audio stream.
#[derive(Debug, Clone, PartialEq)]
pub struct AdpcmImaState {
    pub predictor: i16,
    pub step_index: u8,
}

impl AdpcmImaState {
    /// Creates a new AdpcmState with zero values.
    pub fn new() -> AdpcmImaState {
        AdpcmImaState {
            predictor: 0,
            step_index: 0,
        }
    }
}

impl Default for AdpcmImaState {
    fn default() -> Self {
        Self::new()
    }
}

const IMA_INDEX_TABLE: &[i8; 16] = &[
    -1, -1, -1, -1, 2, 4, 6, 8,
    -1, -1, -1, -1, 2, 4, 6, 8
];

const IMA_STEP_TABLE: &[i16; 89] = &[
    7, 8, 9, 10, 11, 12, 13, 14, 16, 17,
    19, 21, 23, 25, 28, 31, 34, 37, 41, 45,
    50, 55, 60, 66, 73, 80, 88, 97, 107, 118,
    130, 143, 157, 173, 190, 209, 230, 253, 279, 307,
    337, 371, 408, 449, 494, 544, 598, 658, 724, 796,
    876, 963, 1060, 1166, 1282, 1411, 1552, 1707, 1878, 2066,
    2272, 2499, 2749, 3024, 3327, 3660, 4026, 4428, 4871, 5358,
    5894, 6484, 7132, 7845, 8630, 9493, 10442, 11487, 12635, 13899,
    15289, 16818, 18500, 20350, 22385, 24623, 27086, 29794, 32767
];

/// Decodes a 4-bit encoded IMA ADPCM value to a linear 16-bit signed integer sample value.
///
/// Only the lowest 4 bits of `encoded_nibble` are used and the top-most bits are ignored.
///
/// The `state` parameter should be initialized to zero or to values from the audio stream
/// (depending on how the format has specified it). This method updates `state`
/// with new values. Subsequent calls should pass in the state values from the previous call.
#[cfg_attr(feature = "internal-no-panic", no_panic)]
#[inline(always)]
pub fn decode_adpcm_ima(encoded_nibble: u8, state: &mut AdpcmImaState) -> i16 {
    let nibble = encoded_nibble & 0x0f; // ensure nibble is in the range 0..=15
    state.step_index = state.step_index.min(88); // validate step_index

    // calculate the sample value (predictor) from the previous predictor, step and nibble
    let step_size = i32::from(IMA_STEP_TABLE[usize::from(state.step_index)]);
    let mut diff = step_size >> 3;
    if (nibble & 4) != 0 { diff += step_size; }
    if (nibble & 2) != 0 { diff += step_size >> 1; }
    if (nibble & 1) != 0 { diff += step_size >> 2; }
    let mut predictor = i32::from(state.predictor);
    if (nibble & 8) != 0 {
        predictor -= diff;
    } else {
        predictor += diff;
    }
    // store predictor for the next round, clamped to i16
    #[allow(clippy::cast_possible_truncation)] // value is clamped so truncation never happens
    {
    state.predictor = predictor.clamp(-32768, 32767) as i16;
    }
    // adjust step index, clamped to 0..=88
    state.step_index = state.step_index
        .saturating_add_signed(IMA_INDEX_TABLE[usize::from(nibble)])
        .min(88);
    // predictor is the decoded sample value
    state.predictor
}

/// Decodes an AIFF-C / QT "ima4" compressed block to 16-bit signed integer samples.
///
/// `buf` should contain 2 header bytes (predictor and step index) and 32 bytes of 4-bit samples.
///
/// The `state` parameter should be initialized to zero for the first call and subsequent calls
/// should pass in the state values from the previous call.
///
/// This function outputs 64 decoded samples to `out_samples`.
#[cfg_attr(feature = "internal-no-panic", no_panic)]
#[inline(always)]
pub fn decode_adpcm_ima_ima4(buf: &[u8; 34], state: &mut AdpcmImaState,
    out_samples: &mut [i16; 64]) {

    // the first two bytes are the initial state: pppppppp piiiiiii
    let predictor = i16::from_be_bytes([ buf[0], buf[1] & 0b1000_0000 ]);
    // clamp to the same range as macOS AudioToolbox Framework (0..=88)
    let step_index = (buf[1] & 0b0111_1111).min(88);
    // use the previous block's last sample value as the predictor instead of
    // block header's predictor, if the last sample value is close enough the block header's
    // predictor - this increases the decoding accuracy.
    // note that this means that the sample values of the previous block will affect later blocks.
    // this implementation seems to match Audio Toolbox framework.
    if state.step_index != step_index ||
        state.predictor < predictor.saturating_sub(127) ||
        state.predictor > predictor.saturating_add(127) {
        state.predictor = predictor;
        state.step_index = step_index;
    }
    // decode the rest of the block as nibbles
    let mut sample_index = 0;
    for b in &buf[2..] {
        let s0 = decode_adpcm_ima(*b & 0x0f, state);
        out_samples[sample_index] = s0;
        sample_index += 1;
        let s1 = decode_adpcm_ima(*b >> 4, state);
        out_samples[sample_index] = s1;
        sample_index += 1;
    }
}

/// Decodes WAV / MS IMA ADPCM (wav format 0x0011) compressed block to
/// 16-bit signed integer samples.
///
/// `buf` should contain header bytes (predictor and step index) and bytes of 4-bit encoded
/// samples. For 1 channel audio, the `buf` length must be at least 4. For 2 channel audio,
/// the `buf` length must be at least 8 and it must be divisible by 8.
/// The `buf` length must always be less than 65536.
///
/// `is_stereo` should be `false` for 1 channel (mono) audio and `true` for
/// 2 channel (stereo) audio.
///
/// This function outputs decoded samples to `out_samples`. The `out_samples` length must be
/// `2 * buf.len() - 7` for 1 channel audio and `2 * buf.len() - 14` for 2 channel audio.
/// Samples are interleaved for 2 channel audio.
///
/// An error is returned if the `buf` or `out_samples` length isn't correct.
/// If an error is returned, `out_samples` is left unmodified.
pub fn decode_adpcm_ima_ms(buf: &[u8], is_stereo: bool, out_samples: &mut [i16])
    -> Result<(), Error> {

    let channels = if is_stereo {
        2
    } else {
        1
    };
    // check buf length
    if (channels == 1 && buf.len() < 4) ||
        (channels == 2 && (buf.len() < 8 || buf.len() % 8 != 0)) {
        return Err(Error::InvalidBufferSize);
    }
    if buf.len() > 0xffff {
        return Err(crate::Error::InvalidBufferSize);
    }
    // check that the length of the input buffer and output buffer match
    let expected_sample_len = (buf.len() - 4 * channels)
        .checked_mul(2)
        .and_then(|v| v.checked_add(channels))
        .ok_or(Error::InvalidBufferSize)?;
    if expected_sample_len != out_samples.len() {
        return Err(Error::InvalidBufferSize);
    }
    let mut states = [ AdpcmImaState::new(), AdpcmImaState::new() ];
    // the first channels*4 bytes are the initial state (every fourth byte is ignored)
    for ch in 0..channels {
        states[ch].predictor = i16::from_le_bytes([ buf[ch*4], buf[ch*4+1] ]);
        // Windows 10 acmStreamConvert() refuses to convert blocks which have step index > 88
        // and Windows Media Player ignores such blocks.
        // macOS and Audacity clamp step index to 0..=88. Let's copy that behavior here so that
        // something is decoded.
        states[ch].step_index = buf[ch*4+2].min(88);
        out_samples[ch] = states[ch].predictor;
    }
    // decode the rest of the block from nibbles to interleaved samples
    let mut out_index = 0;
    let mut ch = 0;
    let mut out_subindex = 0;
    for b in &buf[4*channels..] {
        let pos = channels + out_index*4*channels*channels + out_subindex*channels + ch;
        out_samples[pos] = decode_adpcm_ima(*b & 0x0f, &mut states[ch]);
        out_samples[pos + channels] = decode_adpcm_ima(*b >> 4, &mut states[ch]);
        out_subindex += 2;
        if out_subindex == 4*channels {
            out_subindex = 0;
            ch += 1;
            if ch == channels {
                ch = 0;
                out_index += 1;
            }
        }
    }
    Ok(())
}

/// Encodes a linear 16-bit signed integer sample value to a 4-bit encoded IMA ADPCM value.
///
/// The `state` parameter should be initialized to zero or to values from the audio stream
/// (depending on how the format has specified it). This method updates `state`
/// with new values. Subsequent calls should pass in the state values from the previous call.
#[cfg_attr(feature = "internal-no-panic", no_panic)]
#[inline(always)]
pub fn encode_adpcm_ima(sample_value: i16, state: &mut AdpcmImaState) -> u8 {
    state.step_index = state.step_index.min(88); // validate step_index

    // calculate the output nibble using the sample value, previous predictor value and step

    let mut diff = i32::from(sample_value) - i32::from(state.predictor);
    let mut nibble: u8;
    if diff >= 0 {
        nibble = 0;
    } else {
        nibble = 8;
        diff = -diff;
    }
    // calculate nibble and predictor_diff
    // nibble bit 4, predictor_diff step_size
    let step_size = i32::from(IMA_STEP_TABLE[usize::from(state.step_index)]);
    let mut predictor_diff: i32 = step_size >> 3;
    let mut temp_step_size = step_size;
    if diff >= temp_step_size {
        nibble |= 4;
        predictor_diff += step_size;
        diff -= temp_step_size;
    }
    // nibble bit 2, predictor_diff step_size/2
    temp_step_size >>= 1;
    if diff >= temp_step_size {
        nibble |= 2;
        predictor_diff += step_size >> 1;
        diff -= temp_step_size;
    }
    // nibble bit 1, predictor_diff step_size/4
    temp_step_size >>= 1;
    if diff >= temp_step_size {
        nibble |= 1;
        predictor_diff += step_size >> 2;
    }

    // update the predicted sample (predictor)
    let mut predictor = i32::from(state.predictor);
    if (nibble & 8) == 8 {
        predictor -= predictor_diff;
    } else {
        predictor += predictor_diff;
    }
    // store predictor for the next round, clamped to i16
    #[allow(clippy::cast_possible_truncation)] // value is clamped so truncation never happens
    {
    state.predictor = predictor.clamp(-32768, 32767) as i16;
    }
    // adjust step index, clamped to 0..=88
    state.step_index = state.step_index
        .saturating_add_signed(IMA_INDEX_TABLE[usize::from(nibble)])
        .min(88);
    // nibble is the encoded value
    nibble
}

/// Encodes 16-bit signed integer samples to an AIFF-C / QT "ima4" compressed block.
///
/// The `state` parameter should be initialized to zero for the first call and subsequent calls
/// should pass in the state values from the previous call.
///
/// This function outputs 34 encoded bytes to `out_buf`: 2 header bytes (predictor and step index)
/// and 32 bytes of 4-bit samples.
#[cfg_attr(feature = "internal-no-panic", no_panic)]
#[inline(always)]
pub fn encode_adpcm_ima_ima4(samples: &[i16; 64], state: &mut AdpcmImaState,
    out_buf: &mut [u8; 34]) {

    state.step_index = state.step_index.min(88);
    // the first two bytes are the initial state: pppppppp piiiiiii
    #[allow(clippy::cast_sign_loss)] // sign loss is expected when splitting the values to bytes
    {
    out_buf[0] = (state.predictor >> 8) as u8;
    out_buf[1] = (state.predictor & 0x80) as u8 | state.step_index;
    }
    // encode 64 samples to 64 nibbles (32 bytes)
    let mut sample_index = 0;
    for out_b in &mut out_buf[2..] {
        let nibble0 = encode_adpcm_ima(samples[sample_index], state);
        sample_index += 1;
        let nibble1 = encode_adpcm_ima(samples[sample_index], state);
        sample_index += 1;
        *out_b = nibble1 << 4 | nibble0;
    }
}

/// Encodes 16-bit signed integer samples to a MS / WAV IMA ADPCM (wav format 0x0011)
/// compressed block.
///
/// Only 1 or 2 channel audio data is supported. For 1 channel audio, there must be an odd number
/// of samples (1, 3, 5, ..) and for 2 channel audio, `samples` length must be divisible by 16
/// after subtracting 2 from it (2, 18, 34, 50, 66, ..).
/// Samples must be interleaved for 2 channel audio.
///
/// `states` must contain channel number of `AdpcmImaState` items (1 or 2). The state objects
/// should be initialized to zero for the first call and subsequent calls
/// should pass in the state values from the previous call.
///
/// This function outputs encoded bytes to `out_buf`. The `out_buf` length must be
/// `((samples.len() - states.len()) / 2) + states.len()*4` and less than 65536.
///
/// Usually, for 1 channel (mono) audio, the `out_buf` length is 1024 and
/// the `samples` length is 2041.
/// For 2 channel (stereo) audio, the `out_buf` length is 2048 and the `samples` length is 4082.
///
/// An error is returned if `states` has an invalid number of state objects or
/// if the `samples` or `out_buf` length isn't correct.
/// If an error is returned, `out_buf` is left unmodified.
pub fn encode_adpcm_ima_ms(samples: &[i16], states: &mut [AdpcmImaState], out_buf: &mut [u8])
    -> Result<(), Error> {
    let channels = states.len();
    if channels < 1 || channels > 2 {
        return Err(crate::Error::InvalidChannels);
    }
    // check samples length
    if (channels == 1 && samples.len() & 1 == 0) ||
        (channels == 2 && (samples.len() < 2 || (samples.len()-2) % 16 != 0)) {
        return Err(crate::Error::InvalidBufferSize);
    }
    // check buf length
    if out_buf.len() > 0xffff {
        return Err(crate::Error::InvalidBufferSize);
    }
    // check that the length of the input buffer and output buffer match
    if ((samples.len() - states.len()) / 2) + states.len()*4 != out_buf.len() {
        return Err(crate::Error::InvalidBufferSize);
    }
    // the first channels*4 bytes are the initial state (every fourth byte is ignored)
    for ch in 0..channels {
        states[ch].predictor = samples[ch];
        //note: the value of states[ch].step_index is used from the function argument
        out_buf[ch*4] = samples[ch].to_le_bytes()[0];
        out_buf[ch*4+1] = samples[ch].to_le_bytes()[1];
        out_buf[ch*4+2] = states[ch].step_index;
        out_buf[ch*4+3] = 0;
    }
    // encode interleaved samples to nibbles
    let mut index = 0;
    let mut ch = 0;
    let mut subindex = 0;
    for b in &mut out_buf[channels*4..] {
        let pos = channels + index*4*channels*channels + subindex*channels + ch;
        let s0 = encode_adpcm_ima(samples[pos], &mut states[ch]);
        let s1 = encode_adpcm_ima(samples[pos+channels], &mut states[ch]);
        *b = s0 | (s1 << 4);
        subindex += 2;
        if subindex == 4*channels {
            subindex = 0;
            ch += 1;
            if ch == channels {
                ch = 0;
                index += 1;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_adpcm_ima() {
        // normal decoding
        let mut state = AdpcmImaState { predictor: 0, step_index: 0 };
        assert_eq!(decode_adpcm_ima(6, &mut state), 10);
        assert_eq!(state, AdpcmImaState { predictor: 10, step_index: 6 });

        // tests that resulting step index is clamped to 0
        let mut state = AdpcmImaState { predictor: 200, step_index: 0 };
        assert_eq!(decode_adpcm_ima(3, &mut state), 204);
        assert_eq!(state, AdpcmImaState { predictor: 204, step_index: 0 });

        // tests that resulting step index is clamped to 88
        let mut state = AdpcmImaState { predictor: 20200, step_index: 84 };
        assert_eq!(decode_adpcm_ima(14, &mut state), -16175);
        assert_eq!(state, AdpcmImaState { predictor: -16175, step_index: 88 });

        // tests that the returned sample is clamped to -32768
        let mut state = AdpcmImaState { predictor: -30123, step_index: 80 };
        assert_eq!(decode_adpcm_ima(14, &mut state), -32768);
        assert_eq!(state, AdpcmImaState { predictor: -32768, step_index: 86 });

        // tests that the returned sample is clamped to 32767
        let mut state = AdpcmImaState { predictor: 30123, step_index: 80 };
        assert_eq!(decode_adpcm_ima(7, &mut state), 32767);
        assert_eq!(state, AdpcmImaState { predictor: 32767, step_index: 88 });

        // check nibble value too large (greater than 15)
        let mut state = AdpcmImaState { predictor: 0, step_index: 0 };
        assert_eq!(decode_adpcm_ima(16, &mut state), 0);
        assert_eq!(state, AdpcmImaState { predictor: 0, step_index: 0 });

        // check input step index too large
        let mut state = AdpcmImaState { predictor: 0, step_index: 89 };
        assert_eq!(decode_adpcm_ima(10, &mut state), -20478);
        assert_eq!(state, AdpcmImaState { predictor: -20478, step_index: 87 });
    }

    #[test]
    fn test_decode_adpcm_ima4() {
        // macOS 14 afconvert has been tested to return the same values

        // simple block
        let mut decoded_buf = [0i16; 64];
        let mut state = AdpcmImaState { predictor: 0, step_index: 0 };
        decode_adpcm_ima_ima4(&[ 0x00, 0x00,
            0x06, 0x08, 0x08, 0x08, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0x08, 0x80, 0x00, 0x80, 0x00, 0x00, 0x00,
            0x01, 0x00, 0x01, 0x11, 0x11, 0x11, 0x22, 0x22,
            0x32, 0x43, 0x33, 0x43, 0x43, 0x42, 0x32, 0x43
        ], &mut state, &mut decoded_buf);
        assert_eq!(decoded_buf, [
            10, 11, 10, 11, 10, 11, 10, 10,
            -1, -31, -94, -230, -523, -1154, -2511, -5421,
            -11657, -25029, -26940, -25203, -23624, -25059, -23754, -22568,
            -21490, -22470, -21579, -20769, -20033, -19364, -18756, -18203,
            -16694, -16237, -15822, -15444, -14414, -14102, -13250, -12476,
            -11773, -11134, -10552, -10024, -9223, -8495, -7833, -7232,
            -6685, -5989, -5356, -4616, -3920, -3287, -2712, -2040,
            -1407, -667, -170, 644, 1191, 1887, 2520, 3260,
        ]);
        assert_eq!(state, AdpcmImaState { predictor: 3260, step_index: 49 });

        // the previous state (0x0cbc, 49) is given to the next call to check that it is used
        decode_adpcm_ima_ima4(&[ 0x0C, 0xB1,
            0x42, 0x32, 0x43, 0x42, 0x32, 0x43, 0x42, 0x32,
            0x43, 0x42, 0x32, 0x43, 0x42, 0x32, 0x33, 0x34,
            0x34, 0x33, 0x34, 0x34, 0x33, 0x34, 0xF5, 0xFF,
            0xEF, 0x80, 0x00, 0x08, 0x80, 0x00, 0x08, 0x80
        ], &mut state, &mut decoded_buf);
        assert_eq!(decoded_buf, [
            3757, 4571, 5118, 5814, 6447, 7187, 7684, 8498,
            9045, 9741, 10374, 11114, 11611, 12425, 12972, 13668,
            14301, 15041, 15538, 16352, 16899, 17595, 18228, 18968,
            19465, 20279, 20826, 21522, 22155, 22730, 23402, 24035,
            24775, 25471, 26104, 26679, 27351, 27984, 28724, 29420,
            30053, 30628, 31300, 31933, 32767, 30963, 27090, 18788,
            990, -32078, -27983, -31707, -28322, -25245, -28043, -25500,
            -23188, -25290, -23379, -21642, -23221, -21786, -20481, -21667,
        ]);
        assert_eq!(state, AdpcmImaState { predictor: -21667, step_index: 74 });

        // passing in zero state with the same block will give different results
        let mut state = AdpcmImaState { predictor: 0, step_index: 0 };
        decode_adpcm_ima_ima4(&[ 0x0C, 0xB1,
            0x42, 0x32, 0x43, 0x42, 0x32, 0x43, 0x42, 0x32,
            0x43, 0x42, 0x32, 0x43, 0x42, 0x32, 0x33, 0x34,
            0x34, 0x33, 0x34, 0x34, 0x33, 0x34, 0xF5, 0xFF,
            0xEF, 0x80, 0x00, 0x08, 0x80, 0x00, 0x08, 0x80
        ], &mut state, &mut decoded_buf);
        assert_eq!(decoded_buf, [
            3697, 4511, 5058, 5754, 6387, 7127, 7624, 8438,
            8985, 9681, 10314, 11054, 11551, 12365, 12912, 13608,
            14241, 14981, 15478, 16292, 16839, 17535, 18168, 18908,
            19405, 20219, 20766, 21462, 22095, 22670, 23342, 23975,
            24715, 25411, 26044, 26619, 27291, 27924, 28664, 29360,
            29993, 30568, 31240, 31873, 32767, 30963, 27090, 18788,
            990, -32078, -27983, -31707, -28322, -25245, -28043, -25500,
            -23188, -25290, -23379, -21642, -23221, -21786, -20481, -21667
        ]);
        assert_eq!(state, AdpcmImaState { predictor: -21667, step_index: 74 });

        // second packet's last sample 127 matches the next packet's predictor 0,
        // which means that 127 is used as the decoded sample value instead of 0
        let mut state = AdpcmImaState { predictor: 0, step_index: 0 };
        decode_adpcm_ima_ima4(&[0, 0,
            182, 179, 195, 180, 178, 196, 179, 179,
            89, 107, 59, 76, 59, 75, 60, 59,
            45, 47, 63, 63, 63, 63, 63, 63,
            63, 63, 63, 60, 59, 76, 59, 43,
        ], &mut state, &mut decoded_buf);
        assert_eq!(decoded_buf, [
            10, 0, 10, 2, 10, 0, 12, 2,
            9, 1, 12, -1, 10, 0, 10, 2,
            -1, 11, 1, 20, 3, 18, -1, 22,
            1, 19, 2, 23, -2, 22, 1, 19,
            -9, 9, -43, -6, -107, -5, -204, -4,
            -395, -3, -768, -2, -1494, -2, -2912, -3,
            -5673, 0, -11050, 4, -21532, 11, -25172, -1473,
            -23016, -3430, -26323, 1377, -24692, -993, -22536, -8546
        ]);
        assert_eq!(state, AdpcmImaState { predictor: -8546, step_index: 83 });
        decode_adpcm_ima_ima4(&[0, 41,
            8, 8, 128, 128, 8, 8, 128, 8,
            128, 8, 128, 128, 128, 128, 8, 9,
            8, 3, 8, 8, 8, 6, 4, 3,
            4, 3, 4, 5, 3, 3, 3, 3,
        ], &mut state, &mut decoded_buf);
        assert_eq!(decoded_buf, [
            -46, -4, -42, -8, 23, -5, 21, -2,
            -23, -4, -21, -5, 9, -4, -16, -5,
            5, -4, -12, -5, 1, -5, 0, -5,
            -1, -5, -2, -5, -8, -6, -13, -11,
            -13, -11, 0, 1, 0, 1, 0, 1,
            0, 0, 10, 11, 24, 25, 35, 36,
            48, 49, 59, 60, 71, 72, 86, 88,
            99, 100, 110, 111, 119, 120, 127, 127
        ]);
        assert_eq!(state, AdpcmImaState { predictor: 127, step_index: 0 });
        decode_adpcm_ima_ima4(&[0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ], &mut state, &mut decoded_buf);
        assert_eq!(decoded_buf, [
            127, 127, 127, 127, 127, 127, 127, 127, 127, 127, 127, 127, 127, 127, 127, 127,
            127, 127, 127, 127, 127, 127, 127, 127, 127, 127, 127, 127, 127, 127, 127, 127,
            127, 127, 127, 127, 127, 127, 127, 127, 127, 127, 127, 127, 127, 127, 127, 127,
            127, 127, 127, 127, 127, 127, 127, 127, 127, 127, 127, 127, 127, 127, 127, 127
        ]);

        // second packet's sample -127 matches the next packet's predictor 0,
        // which means that -127 is used as the decoded sample value instead of 0
        let mut state = AdpcmImaState { predictor: 0, step_index: 0 };
        decode_adpcm_ima_ima4(&[0, 0,
            182, 179, 195, 180, 178, 196, 179, 179, 89, 107, 59, 76, 59, 75, 60, 59,
            45, 47, 63, 63, 63, 63, 63, 63, 63, 63, 63, 60, 59, 76, 59, 43,
        ], &mut state, &mut decoded_buf);
        assert_eq!(decoded_buf, [
            10, 0, 10, 2, 10, 0, 12, 2,
            9, 1, 12, -1, 10, 0, 10, 2,
            -1, 11, 1, 20, 3, 18, -1, 22,
            1, 19, 2, 23, -2, 22, 1, 19,
            -9, 9, -43, -6, -107, -5, -204, -4,
            -395, -3, -768, -2, -1494, -2, -2912, -3,
            -5673, 0, -11050, 4, -21532, 11, -25172, -1473,
            -23016, -3430, -26323, 1377, -24692, -993, -22536, -8546
        ]);
        assert_eq!(state, AdpcmImaState { predictor: -8546, step_index: 83 });
        decode_adpcm_ima_ima4(&[0, 41,
            8, 8, 128, 128, 8, 8, 128, 8, 128, 8, 128, 128, 128, 128, 11, 11,
            11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 10,
        ], &mut state, &mut decoded_buf);
        assert_eq!(decoded_buf, [
            -46, -4, -42, -8, 23, -5, 21, -2,
            -23, -4, -21, -5, 9, -4, -16, -5,
            5, -4, -12, -5, 1, -5, 0, -5,
            -1, -5, -2, -5, -26, -24, -41, -39,
            -53, -51, -62, -61, -71, -70, -78, -77,
            -84, -84, -88, -88, -92, -92, -96, -96,
            -100, -100, -104, -104, -108, -108, -112, -112,
            -116, -116, -120, -120, -124, -124, -127, -127
        ]);
        assert_eq!(state, AdpcmImaState { predictor: -127, step_index: 0 });
        decode_adpcm_ima_ima4(&[0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ], &mut state, &mut decoded_buf);
        assert_eq!(decoded_buf, [
            -127, -127, -127, -127, -127, -127, -127, -127,
            -127, -127, -127, -127, -127, -127, -127, -127,
            -127, -127, -127, -127, -127, -127, -127, -127,
            -127, -127, -127, -127, -127, -127, -127, -127,
            -127, -127, -127, -127, -127, -127, -127, -127,
            -127, -127, -127, -127, -127, -127, -127, -127,
            -127, -127, -127, -127, -127, -127, -127, -127,
            -127, -127, -127, -127, -127, -127, -127, -127
        ]);

        // out-of-bounds step index in buf is clamped
        let mut decoded_buf = [0i16; 64];
        let mut state = AdpcmImaState { predictor: 0, step_index: 0 };
        decode_adpcm_ima_ima4(&[ 0x80, 0x59,
            0x06, 0x08, 0x08, 0x08, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0x08, 0x80, 0x00, 0x80, 0x00, 0x00, 0x00,
            0x01, 0x00, 0x01, 0x11, 0x11, 0x11, 0x22, 0x22,
            0x32, 0x43, 0x33, 0x43, 0x43, 0x42, 0x32, 0x43
        ], &mut state, &mut decoded_buf);
        assert_eq!(decoded_buf, [
            20477, 24572, 20848, 24233, 21156, 23954, 21411, 23723,
            -7810, -32768, -32768, -32768, -32768, -32768, -32768, -32768,
            -32768, -32768, -32768, -29044, -25659, -28736, -25938, -23395,
            -21083, -23185, -21274, -19537, -17958, -16523, -15218, -14032,
            -10797, -9817, -8926, -8116, -5907, -5238, -3413, -1753,
            -244, 1128, 2374, 3508, 5225, 6786, 8206, 9497,
            10670, 12162, 13520, 15107, 16599, 17957, 19190, 20632,
            21990, 23577, 24643, 26389, 27562, 29054, 30412, 31999,
        ]);
        assert_eq!(state, AdpcmImaState { predictor: 31999, step_index: 57 });
    }

    #[test]
    fn test_decode_adpcm_ms() {
        // Windows 10 acmStreamConvert() has been tested to return the same values

        // one channel
        let mut samples = [0i16; 25];
        assert!(decode_adpcm_ima_ms(&[ 0xAE, 0xC8, 0x40, 0x00,
            0x10, 0x10, 0x10, 0x11, 0x21, 0x21, 0x22, 0x32, 0x43, 0x33, 0x43, 0x43
        ], false, &mut samples).is_ok());
        assert_eq!(samples, [
            -14162, -13747, -12613, -12270, -11334, -11050, -10276, -9573, -8934, -8352,
            -7471, -6991, -6263, -5601, -5000, -4453, -3757, -3124, -2384, -1688,
            -1055, -480, 192, 825, 1565
        ]);

        // two channels
        let mut samples = [0i16; 18];
        assert!(decode_adpcm_ima_ms(&[  0x38, 0xB1, 0x47, 0x00,
            0x1A, 0x9B, 0x50, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x08, 0x00, 0x08
        ], true, &mut samples).is_ok());
        assert_eq!(samples, [
            -20168, -25830, -19358, -23919, -18622, -22182, -17953, -23761, -17345, -22326,
            -15685, -21021, -15182, -19835, -14725, -20913, -14310, -19933
        ]);

        // not enough input data for 1 channel audio
        let mut samples = [0i16; 25];
        assert!(matches!(decode_adpcm_ima_ms(&[ 0x38, 0xB1, 0x47
        ], false, &mut samples), Err(Error::InvalidBufferSize)));

        // invalid buf length for 2 channel audio
        let mut samples = [0i16; 4];
        assert!(matches!(decode_adpcm_ima_ms(&[ 0x38, 0xB1, 0x47, 0x00,
            0x38, 0xB1, 0x47, 0x38, 0xB1
        ], true, &mut samples), Err(Error::InvalidBufferSize)));

        // out-of-bounds step index is clamped
        // (acmStreamConvert() doesn't return any result for this, but some other libraries
        // clamp out-of-bounds step indexes and this implementation matches that behavior)
        let mut samples = [0i16; 25];
        assert!(decode_adpcm_ima_ms(&[ 0x11, 0x81, 89, 0x00,
            0x10, 0x10, 0x10, 0x11, 0x21, 0x21, 0x22, 0x32, 0x43, 0x33, 0x43, 0x43
        ], false, &mut samples).is_ok());
        assert_eq!(samples, [
            -32495, -28400, -17228, -13843, -4611, -1813, 5817, 12754, 19060, 24793,
            32767, 32767, 32767, 32767, 32767, 32767, 32767, 32767, 32767, 32767,
            32767, 32767, 32767, 32767, 32767
        ]);

        // invalid out_samples length
        let mut samples = [0i16; 26];
        assert!(matches!(decode_adpcm_ima_ms(&[ 0xAE, 0xC8, 0x40, 0x00,
            0x10, 0x10, 0x10, 0x11, 0x21, 0x21, 0x22, 0x32, 0x43, 0x33, 0x43, 0x43
        ], false, &mut samples), Err(Error::InvalidBufferSize)));

        // 1 channel and buf size 1024 can be decoded to 2041 samples
        let mut samples = [0i16; 2041];
        assert!(decode_adpcm_ima_ms(&[0u8; 1024], false, &mut samples).is_ok());

        // 2 channels and buf size 2048 can be decoded to 4082 samples
        let mut samples = [0i16; 4082];
        assert!(decode_adpcm_ima_ms(&[0u8; 2048], true, &mut samples).is_ok());
    }

    #[test]
    fn test_decode_adpcm_ms_with_different_buf_sizes() {
        let buf_area = [0u8; 4096];
        let mut sample_area = [0i16; 8192];
        // one channel
        for buf_len in 0..=1025 {
            let buf = &buf_area[0..buf_len];
            let sample_len = 2 * buf.len().max(4) - 7 * 1;
            let mut samples = &mut sample_area[0..sample_len];
            if buf_len >= 4 {
                assert!(decode_adpcm_ima_ms(&buf, false, &mut samples).is_ok());
            } else {
                assert!(matches!(decode_adpcm_ima_ms(&buf, false, &mut samples),
                    Err(Error::InvalidBufferSize)));
            }
        }
        // two channels
        for buf_len in 0..=2049 {
            let buf = &buf_area[0..buf_len];
            let sample_len = 2 * buf.len().max(7) - 7 * 2;
            let mut samples = &mut sample_area[0..sample_len];
            if buf_len >= 8 && buf_len % 8 == 0 {
                assert!(decode_adpcm_ima_ms(&buf, true, &mut samples).is_ok());
            } else {
                assert!(matches!(decode_adpcm_ima_ms(&buf, true, &mut samples),
                    Err(Error::InvalidBufferSize)));
            }
        }
    }

    #[test]
    fn test_encode_adpcm_ima() {
        // normal encoding
        let mut state = AdpcmImaState { predictor: 0, step_index: 0 };
        assert_eq!(encode_adpcm_ima(10, &mut state), 6);
        assert_eq!(state, AdpcmImaState { predictor: 10, step_index: 6 });

        // tests that output step index is clamped to 0
        let mut state = AdpcmImaState { predictor: 0, step_index: 0 };
        assert_eq!(encode_adpcm_ima(0, &mut state), 0);
        assert_eq!(state, AdpcmImaState { predictor: 0, step_index: 0 });

        // tests that output step index is clamped to 88
        let mut state = AdpcmImaState { predictor: -30350, step_index: 83 };
        assert_eq!(encode_adpcm_ima(897, &mut state), 6);
        assert_eq!(state, AdpcmImaState { predictor: 2718, step_index: 88 });

        // tests that the returned sample is clamped to -32768
        let mut state = AdpcmImaState { predictor: -32550, step_index: 65 };
        assert_eq!(encode_adpcm_ima(-32697, &mut state), 8);
        assert_eq!(state, AdpcmImaState { predictor: -32768, step_index: 64 });

        // tests that the returned sample is clamped to 32767
        let mut state = AdpcmImaState { predictor: 32700, step_index: 65 };
        assert_eq!(encode_adpcm_ima(32760, &mut state), 0);
        assert_eq!(state, AdpcmImaState { predictor: 32767, step_index: 64 });

        // check passing in a step index with a too large value
        let mut state = AdpcmImaState { predictor: 0, step_index: 89 };
        assert_eq!(encode_adpcm_ima(0, &mut state), 0);
        assert_eq!(state, AdpcmImaState { predictor: 4095, step_index: 87 });
    }

    #[test]
    fn test_encode_adpcm_ima4() {
        // macOS 14 afconvert has been tested to return the same values

        // simple block with zero initial state values
        let mut state = AdpcmImaState { predictor: 0, step_index: 0 };
        let mut encoded_buf = [0u8; 34];
        encode_adpcm_ima_ima4(&[
            10, 10, 10, 10, 10, 10, 10, 10,
            -32768, -32114, -31460, -30806, -30153, -29499, -28845, -28191,
            -27537, -26883, -26229, -25575, -24922, -24268, -23614, -22960,
            -22306, -21652, -20998, -20344, -19691, -19037, -18383, -17729,
            -17075, -16421, -15767, -15113, -14460, -13806, -13152, -12498,
            -11844, -11190, -10536, -9882, -9229, -8575, -7921, -7267,
            -6613, -5959, -5305, -4651, -3998, -3344, -2690, -2036,
            -1382, -728, -74, 580, 1233, 1887, 2541, 3195,
        ], &mut state, &mut encoded_buf);
        assert_eq!(encoded_buf, [ 0x00, 0x00,
            0x06, 0x08, 0x08, 0x08, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0x08, 0x80, 0x00, 0x80, 0x00, 0x00, 0x00,
            0x01, 0x00, 0x01, 0x11, 0x11, 0x11, 0x22, 0x22,
            0x32, 0x43, 0x33, 0x43, 0x43, 0x42, 0x32, 0x43
        ]);
        assert_eq!(state, AdpcmImaState { predictor: 0x0cbc, step_index: 49 });

        // the previous state (0x0cbc, 49) is given to the next call to check that it is used
        encode_adpcm_ima_ima4(&[
            3849, 4503, 5157, 5811, 6464, 7118, 7772, 8426,
            9080, 9734, 10388, 11042, 11695, 12349, 13003, 13657,
            14311, 14965, 15619, 16273, 16926, 17580, 18234, 18888,
            19542, 20196, 20850, 21504, 22157, 22811, 23465, 24119,
            24773, 25427, 26081, 26735, 27388, 28042, 28696, 29350,
            30004, 30658, 31312, 31966, 32767, -32263, -31609, -30955,
            -30301, -29647, -28993, -28339, -27686, -27032, -26378, -25724,
            -25070, -24416, -23762, -23108, -22455, -21801, -21147, -20493,
        ], &mut state, &mut encoded_buf);
        assert_eq!(encoded_buf, [ 0x0C, 0xB1,
            0x42, 0x32, 0x43, 0x42, 0x32, 0x43, 0x42, 0x32,
            0x43, 0x42, 0x32, 0x43, 0x42, 0x32, 0x33, 0x34,
            0x34, 0x33, 0x34, 0x34, 0x33, 0x34, 0xF5, 0xFF,
            0xEF, 0x80, 0x00, 0x08, 0x80, 0x00, 0x08, 0x80
        ]);
        assert_eq!(state, AdpcmImaState { predictor: -21667, step_index: 74 });

        // large sample values with initial zero state values
        let mut state = AdpcmImaState { predictor: 0, step_index: 0 };
        encode_adpcm_ima_ima4(&[
            16000, 24000, 30000, 32000, 32000, 30000, 24000, 16000,
            8000, 0, -8000, -16000, -24000, -30000, -32000, -32000,
            32000, 32000, 32000, 32000, -32000, -32000, -32000, -32000,
            32000, 32000, 32000, 32000, -32000, -32000, -32000, -32000,
            -32, -16, -8, 0, 8, 16, 32, 16,
            8, 0, -8, -16, -32, -16, -8, 0,
            4, 8, 12, 16, 0, 4, 8, 12,
            16, 12, 8, 4, 0, 16, 8, 4,
        ], &mut state, &mut encoded_buf);
        assert_eq!(encoded_buf, [ 0x00, 0x00,
            0x77, 0x77, 0x77, 0x77, 0xf3, 0xae, 0xab, 0x88,
            0x77, 0x02, 0x9f, 0x08, 0x17, 0x80, 0x9f, 0x08,
            0x04, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08,
            0x08, 0x08, 0x08, 0x88, 0x00, 0x88, 0x00, 0x88,
        ]);
        assert_eq!(state, AdpcmImaState { predictor: -197, step_index: 56 });
    }

    #[test]
    fn test_encode_adpcm_ms() {
        // Windows 10 acmStreamConvert() has been tested to return the same values

        // one channel
        let mut states = [ AdpcmImaState::new() ];
        let mut buf = [0u8; 16];
        assert!(encode_adpcm_ima_ms(&[
            10, 10, 20, 50, 80, 100, 500, 1000, 1500, 2000,
            1500, 800, 500, 300, 100, -100, -300, -500, -800, -1400,
            -3000, -6000, -9000, -12000, -15000
        ], &mut states, &mut buf).is_ok());
        assert_eq!(buf, [
            10, 0, 0, 0, 96, 87, 113, 119, 7, 155, 153, 169, 185, 254, 223, 187
        ]);

        // two channels
        let mut states = [ AdpcmImaState::new(), AdpcmImaState::new() ];
        let mut buf = [0u8; 16];
        assert!(encode_adpcm_ima_ms(&[
            10, 18, 30, 38, 50, 57, 100, 106, 400, 410,
            300, 310, 100, 110, 40, 46, 20, 26
        ], &mut states, &mut buf).is_ok());
        assert_eq!(buf, [
            10, 0, 0, 0, 18, 0, 0, 0, 119, 117, 228, 9, 119, 117, 228, 9
        ]);

        // zero and three channels fail
        let mut states = [];
        let mut buf = [0u8; 16];
        assert!(matches!(encode_adpcm_ima_ms(&[
            10, 18, 30, 38, 50, 57, 100, 106, 400, 410, 300, 310, 100, 110, 40, 46, 20, 26
        ], &mut states, &mut buf), Err(Error::InvalidChannels)));

        let mut states = [ AdpcmImaState::new(), AdpcmImaState::new(), AdpcmImaState::new() ];
        let mut buf = [0u8; 16];
        assert!(matches!(encode_adpcm_ima_ms(&[
            10, 18, 30, 38, 50, 57, 100, 106, 400, 410, 300, 310, 100, 110, 40, 46, 20, 26
        ], &mut states, &mut buf), Err(Error::InvalidChannels)));

        // invalid number of samples
        let mut states = [ AdpcmImaState::new() ];
        let mut buf = [0u8; 5];
        assert!(matches!(encode_adpcm_ima_ms(&[
            10, 18
        ], &mut states, &mut buf), Err(Error::InvalidBufferSize)));

        let mut states = [ AdpcmImaState::new(), AdpcmImaState::new() ];
        let mut buf = [0u8; 16];
        assert!(matches!(encode_adpcm_ima_ms(&[
            10, 18, 20
        ], &mut states, &mut buf), Err(Error::InvalidBufferSize)));

        // invalid out_buf length
        let mut states = [ AdpcmImaState::new() ];
        let mut buf = [0u8; 15];
        assert!(matches!(encode_adpcm_ima_ms(&[
            10, 10, 20, 50, 80, 100, 500, 1000, 1500, 2000,
            1500, 800, 500, 300, 100, -100, -300, -500, -800, -1400,
            -3000, -6000, -9000, -12000, -15000
        ], &mut states, &mut buf), Err(Error::InvalidBufferSize)));

        // 1 channel 2041 samples can be encoded to buf size 1024
        let mut states = [ AdpcmImaState::new() ];
        let mut buf = [0u8; 1024];
        assert!(encode_adpcm_ima_ms(&[0i16; 2041], &mut states, &mut buf).is_ok());

        // 2 channels 4082 samples can be encoded to buf size 2048
        let mut states = [ AdpcmImaState::new(), AdpcmImaState::new() ];
        let mut buf = [0u8; 2048];
        assert!(encode_adpcm_ima_ms(&[0i16; 4082], &mut states, &mut buf).is_ok());
    }

    #[test]
    fn test_encode_adpcm_ms_with_different_buf_sizes() {
        let sample_area = [0i16; 8192];
        let mut buf_area = [0u8; 4096];
        // one channel
        for buf_len in 0..=1025 {
            let mut buf = &mut buf_area[0..buf_len];
            let sample_len = 2 * buf.len().max(4) - 7 * 1;
            let samples = &sample_area[0..sample_len];
            let mut states = [ AdpcmImaState::new() ];
            if buf_len >= 4 {
                assert!(encode_adpcm_ima_ms(&samples, &mut states, &mut buf).is_ok());
            } else {
                assert!(matches!(encode_adpcm_ima_ms(&samples, &mut states, &mut buf),
                    Err(Error::InvalidBufferSize)));
            }
        }
        // two channels
        for buf_len in 0..=2049 {
            let mut buf = &mut buf_area[0..buf_len];
            let sample_len = 2 * buf.len().max(7) - 7 * 2;
            let samples = &sample_area[0..sample_len];
            let mut states = [ AdpcmImaState::new(), AdpcmImaState::new() ];
            if buf_len >= 8 && buf_len % 8 == 0 {
                assert!(encode_adpcm_ima_ms(&samples, &mut states, &mut buf).is_ok());
            } else {
                assert!(matches!(encode_adpcm_ima_ms(&samples, &mut states, &mut buf),
                    Err(Error::InvalidBufferSize)));
            }
        }
    }

}
