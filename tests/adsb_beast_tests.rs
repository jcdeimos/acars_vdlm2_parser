mod common;

use acars_vdlm2_parser::adsb_beast::{AdsbBeastMessage, NewAdsbBeastMessage};
use acars_vdlm2_parser::DecodedMessage;
use acars_vdlm2_parser::DeserializatonError;
use bincode;
use std::error::Error;

use crate::common::{
    combine_files_of_message_type, compare_errors, load_files_of_message_type,
    process_file_as_adsb_beast, MessageType,
};

/// This test will ingest contents from the adsb json sample files as a message per line to a `Vec<String>`.
/// It combines the two files together into a single `Vec<String>` for iterating through.
/// Then it will cycle them into `Vec<ADSBMessage>` and back to `String`.
/// It validates that there are no errors going `String` -> `ADSBMessage` and `ADSBMessage` -> `String`.
#[test]
fn test_adsb_beast_parsing() -> Result<(), Box<dyn Error>> {
    match combine_files_of_message_type(MessageType::AdsbBeast) {
        Err(load_failed) => Err(load_failed),
        Ok(adsb_messages) => {
            let mut valid_adsb_messages: Vec<AdsbBeastMessage> = Vec::new();
            let mut failed_decodes: Vec<String> = Vec::new();
            for line in adsb_messages {
                match line {
                    common::TestFileType::String(_) => {}
                    common::TestFileType::U8(line_as_u8) => match line_as_u8.to_adsb_beast() {
                        Err(_) => failed_decodes.push(String::from_utf8(line_as_u8)?),
                        Ok(adsb_beast_message) => valid_adsb_messages.push(adsb_beast_message),
                    },
                }
            }
            println!("Size of bad messages: {}", failed_decodes.len());
            for message in valid_adsb_messages {
                assert!(message.to_string().as_ref().err().is_none());
                assert!(message.to_bytes().as_ref().err().is_none());
            }
            for line in failed_decodes {
                let error = bincode::deserialize(&line.as_bytes())
                    .map_err(|e| DeserializatonError::BoxError(e));
                compare_errors(line.to_adsb_beast().err(), error, &line);
            }
            //Ok(())
            Err("This test is currently failing, but it's not a big deal. It's just a test for the test harness.".into())
        }
    }
}

/// Test for displaying the per-item result for vdlm2 messages, helpful when diagnosing parsing issues.
/// Marked as `#[ignore]` so it can be run separately as required.
#[test]
#[ignore]
fn show_adsb_beast_injest() -> Result<(), Box<dyn Error>> {
    println!("Showing ADSB Beast ingest errors");
    match load_files_of_message_type(MessageType::AdsbBeast) {
        Err(load_failed) => Err(load_failed),
        Ok(beast_files) => {
            for file in beast_files {
                println!("Testing the contents from file: {}", file.name);
                match file.contents {
                    common::FileTypes::String(_) => {} // we should never end up here in this test, but you know, Rust
                    common::FileTypes::U8(file_as_vec_u8) => {
                        process_file_as_adsb_beast(&file_as_vec_u8)
                    }
                }
            }
            Ok(())
        }
    }
}
