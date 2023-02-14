use crate::error_handling::adsb_raw_error::ADSBRawError;
use crate::error_handling::deserialization_error::DeserializationError;
use hex;

const ADSB_RAW_START_CHARACTER: u8 = 0x2a; // The adsb raw end charater sequence is is a '0x3b0a', start is '0x2a'
const ADSB_RAW_END_SEQUENCE_FINISH_CHARACTER: u8 = 0x3b;
const ADSB_RAW_END_SEQUENCE_INIT_CHARACTER: u8 = 0x0a;
const ADSB_RAW_FRAME_SMALL: usize = 14;
const ADSB_RAW_FRAME_LARGE: usize = 28;

/// Helper function to format ADSB Raw frames from bytes
/// Expected input is a Vec<u8> of the raw frame(s), including the control characters to start and end the frame
/// Does not consume the input
/// Returns a vector of bytes, with each element of the array being a frame that can be passed in to the ADSB Raw parser

pub fn format_adsb_raw_frames_from_bytes(bytes: &Vec<u8>) -> Vec<Vec<u8>> {
    let mut formatted_frames: Vec<Vec<u8>> = Vec::new();
    let mut current_frame: Vec<u8> = Vec::new();
    let mut errors_found: Vec<DeserializationError> = Vec::new();
    let mut entry = 0;

    // TODO: verify this loop does not skip the very last frame in the sequence
    for byte in bytes.iter() {
        if byte == &ADSB_RAW_END_SEQUENCE_INIT_CHARACTER && entry != 0 {
            match current_frame.len() {
                // The frame size is valid
                ADSB_RAW_FRAME_SMALL | ADSB_RAW_FRAME_LARGE => {
                    // FIXME: there is some kind of stupid read-in issue with the data where I need to do this round-robin
                    // nonesense to convert the data to a string and then back in to a vector of u8s
                    // Maybe I should chunk the input?
                    if let Ok(frame_string) = String::from_utf8(current_frame.clone()) {
                        if let Ok(frame_bytes) = hex::decode(frame_string) {
                            formatted_frames.push(frame_bytes);
                        } else {
                            errors_found.push(DeserializationError::ADSBRawError(
                                ADSBRawError::StringError {
                                    message: "Could not convert the {frame_string} string to bytes"
                                        .to_string(),
                                },
                            ));
                        }
                    } else {
                        errors_found.push(DeserializationError::ADSBRawError(
                            ADSBRawError::StringError {
                                message: "Could not convert the bytes {current_frame} to a string"
                                    .to_string(),
                            },
                        ));
                    }
                }
                // The frame size is invalid
                _ => {
                    errors_found.push(DeserializationError::ADSBRawError(
                        ADSBRawError::ByteSequenceWrong {
                            size: current_frame.len() as u8,
                        },
                    ));
                }
            }
        } else if byte == &ADSB_RAW_START_CHARACTER {
            entry += 1;
            current_frame = Vec::new();
        } else if byte != &ADSB_RAW_END_SEQUENCE_FINISH_CHARACTER
            && byte != &ADSB_RAW_END_SEQUENCE_INIT_CHARACTER
        {
            current_frame.push(byte.clone());
        }
    }

    // If there are any errors, print them out
    if errors_found.len() > 0 {
        println!("Errors found in ADSB Raw frame formatting:");
        for error in errors_found {
            println!("{}", error);
        }
    }

    formatted_frames
}
