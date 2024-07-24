/*!

Example to encode or decode single values given as command line arguments.

*/

use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Usage: codec-tester {{decode|encode}} {{ulaw|alaw|adpcm_ima}} values...");
        return ExitCode::FAILURE;
    }

    let mut adpcm_state = audio_codec_algorithms::AdpcmImaState::new();
    let command = &args[1];
    let format = &args[2];
    for i in 3..args.len() {
        match (command.as_ref(), format.as_ref()) {
            ("decode", "ulaw") => {
                println!("{}",
                    audio_codec_algorithms::decode_ulaw(args[i].parse::<u8>().expect("bad value")));
            },
            ("decode", "alaw") => {
                println!("{}",
                    audio_codec_algorithms::decode_alaw(args[i].parse::<u8>().expect("bad value")));
            },
            ("decode", "adpcm_ima") => {
                println!("{}", audio_codec_algorithms::decode_adpcm_ima(args[i].parse::<u8>()
                    .expect("bad value"), &mut adpcm_state));
            },
            ("encode", "ulaw") => {
                println!("{}",
                   audio_codec_algorithms::encode_ulaw(args[i].parse::<i16>().expect("bad value")));
            },
            ("encode", "alaw") => {
                println!("{}",
                   audio_codec_algorithms::encode_alaw(args[i].parse::<i16>().expect("bad value")));
            },
            ("encode", "adpcm_ima") => {
                println!("{}", audio_codec_algorithms::encode_adpcm_ima(args[i].parse::<i16>()
                        .expect("bad value"), &mut adpcm_state));
            },
            _ => {
                eprintln!("ERROR: invalid command or format: {}, {}", command, format);
                return ExitCode::FAILURE;
            }
        };
    }
    ExitCode::SUCCESS
}
