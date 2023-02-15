use crate::common::{
    combine_files_of_message_type, compare_deku_errors, compare_serde_errors,
    test_enum_serialisation, MessageType, SerialisationTarget,
};
use acars_vdlm2_parser::helpers::encode_adsb_raw_input::format_adsb_raw_frames_from_bytes;
use acars_vdlm2_parser::message_types::adsb_raw::{AdsbRawMessage, NewAdsbRawMessage};
use acars_vdlm2_parser::{DecodeMessage, DecodedMessage};
use deku::prelude::*;
use rand::prelude::{SliceRandom, ThreadRng};
use rand::thread_rng;
use std::error::Error;

mod common;

/// This test ingests the contents of all the Acars and Vdlm2 sample files into individual `Vec<String>`.
/// Then it combines two pairs of known good into a single `Vec<String>` and randomises the ordering.
/// Then it will cycle them into `Vec<DecodedMessage>` and back to `String`.
/// It validates that there are no errors going `String` -> `DecodedMessage` and `DecodedMessage` -> `String`.
/// It then combines two files containing known bad data into a single `Vec<String>` and randomises the ordering.
/// It validates that it gets errors that it is expecting and the correct number of errors.
#[test]
fn test_determining_message() -> Result<(), Box<dyn Error>> {
    match combine_files_of_message_type(MessageType::All) {
        Err(load_error) => Err(load_error),
        Ok(mut all_messages) => {
            let mut rng: ThreadRng = thread_rng();
            let mut successfully_decoded_items: Vec<DecodedMessage> = Vec::new();
            let mut failed_decodes: Vec<common::TestFileType> = Vec::new();
            all_messages.shuffle(&mut rng);
            for entry in all_messages {
                match entry.clone() {
                    common::TestFileType::String(line_as_string) => {
                        match line_as_string.decode_json() {
                            Err(e) => failed_decodes.push(entry.clone()),
                            Ok(json_message) => successfully_decoded_items.push(json_message),
                        }
                    }
                    common::TestFileType::U8(line_as_u8) => {
                        let adsb_raw_lines = format_adsb_raw_frames_from_bytes(&line_as_u8);

                        for adsb_raw_line in adsb_raw_lines {
                            match adsb_raw_line.decode_adsb_raw() {
                                Err(_) => failed_decodes.push(entry.clone()),
                                Ok(json_message) => successfully_decoded_items.push(json_message),
                            }
                        }
                    }
                }
            }
            successfully_decoded_items.shuffle(&mut rng);
            for message in &successfully_decoded_items {
                test_enum_serialisation(&message, SerialisationTarget::Both);
            }

            println!(
                "{} messages successfully decoded",
                successfully_decoded_items.len()
            );
            println!("{} messages failed to decode", failed_decodes.len());

            for line in failed_decodes {
                match line.clone() {
                    common::TestFileType::String(line_as_string) => {
                        match line_as_string.decode_json() {
                            Err(_) => {
                                compare_serde_errors(
                                    line_as_string.decode_json().err(),
                                    serde_json::from_str(&line_as_string).map_err(|e| e.into()),
                                    &line_as_string,
                                );
                            }
                            Ok(_) => {
                                panic!("Should not have been able to decode this line");
                            }
                        }
                    }
                    common::TestFileType::U8(line_as_u8) => match line_as_u8.decode_json() {
                        Err(_) => {
                            compare_deku_errors(
                                line_as_u8.to_adsb_raw().err(),
                                AdsbRawMessage::from_bytes((&line_as_u8, 0)).map_err(|e| e.into()),
                                format!("{:?}", line_as_u8).as_str(),
                            );
                        }
                        Ok(_) => {
                            panic!("Should not have been able to decode this line");
                        }
                    },
                }
            }
            Ok(())
        }
    }
}
