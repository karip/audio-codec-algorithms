
use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Usage: codec-tester {{decode|encode}} {{ulaw|alaw}} values...");
        return ExitCode::FAILURE;
    }

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
            ("encode", "ulaw") => {
                println!("{}",
                   audio_codec_algorithms::encode_ulaw(args[i].parse::<i16>().expect("bad value")));
            },
            ("encode", "alaw") => {
                println!("{}",
                   audio_codec_algorithms::encode_alaw(args[i].parse::<i16>().expect("bad value")));
            },
            _ => {
                eprintln!("ERROR: invalid command or format: {}, {}", command, format);
                return ExitCode::FAILURE;
            }
        };
    }
    ExitCode::SUCCESS
}
