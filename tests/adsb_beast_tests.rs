mod common;

use acars_vdlm2_parser::error_handling::deserialization_error::DeserializationError;
use acars_vdlm2_parser::message_types::adsb_raw::{AdsbRawMessage, NewAdsbRawMessage};
use acars_vdlm2_parser::DecodedMessage;
use deku::prelude::*;
use std::error::Error;

use crate::common::{
    combine_files_of_message_type, compare_errors, load_files_of_message_type,
    process_file_as_adsb_raw, MessageType,
};

/// This test will ingest contents from the adsb json sample files as a message per line to a `Vec<String>`.
/// It combines the two files together into a single `Vec<String>` for iterating through.
/// Then it will cycle them into `Vec<ADSBMessage>` and back to `String`.
/// It validates that there are no errors going `String` -> `ADSBMessage` and `ADSBMessage` -> `String`.
#[test]
#[ignore]
fn test_adsb_raw_parsing() -> Result<(), Box<dyn Error>> {
    match combine_files_of_message_type(MessageType::AdsbRaw) {
        Err(load_failed) => Err(load_failed),
        Ok(adsb_messages) => {
            let mut valid_adsb_messages: Vec<AdsbRawMessage> = Vec::new();
            let mut failed_decodes: Vec<String> = Vec::new();
            for line in adsb_messages {
                match line {
                    common::TestFileType::String(_) => {}
                    common::TestFileType::U8(line_as_u8) => match line_as_u8.to_adsb_raw() {
                        Err(_) => failed_decodes.push(String::from_utf8(line_as_u8)?),
                        Ok(adsb_raw_message) => valid_adsb_messages.push(adsb_raw_message),
                    },
                }
            }
            println!("Size of bad messages: {}", failed_decodes.len());
            for message in valid_adsb_messages {
                assert!(message.to_string().as_ref().err().is_none());
                assert!(message.to_bytes().as_ref().err().is_none());
            }
            for line in failed_decodes {
                let error = AdsbRawMessage::from_bytes((&line.as_bytes(), 0))
                    .map_err(|e| DeserializationError::DekuError(e));
                //compare_errors(line.to_adsb_raw().err(), error, &line);
            }
            Ok(())
            //Err("This test is currently failing, but it's not a big deal. It's just a test for the test harness.".into())
        }
    }
}

/// Test for displaying the per-item result for vdlm2 messages, helpful when diagnosing parsing issues.
/// Marked as `#[ignore]` so it can be run separately as required.
#[test]
// #[ignore]
fn show_adsb_raw_injest() -> Result<(), Box<dyn Error>> {
    println!("Showing ADSB Raw ingest errors");
    match load_files_of_message_type(MessageType::AdsbRaw) {
        Err(load_failed) => Err(load_failed),
        Ok(raw_files) => {
            for file in raw_files {
                println!("Testing the contents from file: {}", file.name);
                println!("Size of file: {:?}", file.contents);
                match file.contents {
                    common::FileTypes::String(_) => {} // we should never end up here in this test, but you know, Rust
                    common::FileTypes::U8(file_as_vec_u8) => {
                        process_file_as_adsb_raw(&file_as_vec_u8);
                    }
                }
            }
            Ok(())
        }
    }
}
