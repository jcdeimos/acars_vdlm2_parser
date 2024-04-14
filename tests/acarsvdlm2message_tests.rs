use std::error::Error;
use rand::prelude::{SliceRandom, ThreadRng};
use rand::thread_rng;
use acars_vdlm2_parser::{ReceivedMessage, DecodeMessage};
use crate::common::{combine_files_of_message_type, compare_errors, MessageType, SerialisationTarget, test_enum_serialisation};

mod common;

/// This test ingests the contents of all the Acars and Vdlm2 sample files into individual `Vec<String>`.
/// Then it combines two pairs of known good into a single `Vec<String>` and randomises the ordering.
/// Then it will cycle them into `Vec<AcarsVdlm2Message>` and back to `String`.
/// It validates that there are no errors going `String` -> `AcarsVdlm2Message` and `AcarsVdlm2Message` -> `String`.
/// It then combines two files containing known bad data into a single `Vec<String>` and randomises the ordering.
/// It validates that it gets errors that it is expecting and the correct number of errors.
#[test]
fn test_determining_message() -> Result<(), Box<dyn Error>> {
    match combine_files_of_message_type(MessageType::All) {
        Err(load_error) => Err(load_error),
        Ok(mut all_messages) => {
            let mut rng: ThreadRng = thread_rng();
            let mut successfully_decoded_items: Vec<ReceivedMessage> = Vec::new();
            let mut failed_decodes: Vec<String> = Vec::new();
            all_messages.shuffle(&mut rng);
            for entry in all_messages {
                match entry.decode_message() {
                    Err(_) =>
                        failed_decodes.push(entry),
                    Ok(decoded_message) =>
                        successfully_decoded_items.push(decoded_message),
                }
            }
            successfully_decoded_items.shuffle(&mut rng);
            for message in successfully_decoded_items {
                test_enum_serialisation(&message, SerialisationTarget::Both);
            }
            for line in failed_decodes {
                compare_errors(line.decode_message().err(), serde_json::from_str(&line), &line);
            }
            Ok(())
        }
    }
}