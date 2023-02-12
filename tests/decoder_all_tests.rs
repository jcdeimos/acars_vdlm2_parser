use crate::common::{
    combine_files_of_message_type, compare_errors, test_enum_serialisation, MessageType,
    SerialisationTarget,
};
use acars_vdlm2_parser::error_handling::deserialization_error::DeserializationError;
use acars_vdlm2_parser::{DecodeMessage, DecodedMessage};
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
#[ignore]
fn test_determining_message() -> Result<(), Box<dyn Error>> {
    match combine_files_of_message_type(MessageType::All) {
        Err(load_error) => Err(load_error),
        Ok(mut all_messages) => {
            let mut rng: ThreadRng = thread_rng();
            let mut successfully_decoded_items: Vec<DecodedMessage> = Vec::new();
            let mut failed_decodes: Vec<String> = Vec::new();
            all_messages.shuffle(&mut rng);
            for entry in all_messages {
                match entry {
                    common::TestFileType::String(line_as_string) => {
                        match line_as_string.decode_message() {
                            Err(_) => failed_decodes.push(line_as_string),
                            Ok(json_message) => successfully_decoded_items.push(json_message),
                        }
                    }
                    common::TestFileType::U8(line_as_u8) => match line_as_u8.decode_message() {
                        Err(_) => failed_decodes.push(format!("{:?}", line_as_u8)),
                        Ok(bit_message) => successfully_decoded_items.push(bit_message),
                    },
                }
            }
            successfully_decoded_items.shuffle(&mut rng);
            for message in successfully_decoded_items {
                test_enum_serialisation(&message, SerialisationTarget::Both);
            }
            let length = failed_decodes.len();
            for line in failed_decodes {
                println!("{}{}", line, length);
                compare_errors(
                    line.decode_message().err(),
                    serde_json::from_str(&line).map_err(|e| DeserializationError::SerdeError(e)),
                    &line,
                );
            }
            Ok(())
        }
    }
}
