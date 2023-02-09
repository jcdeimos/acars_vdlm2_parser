mod common;

use acars_vdlm2_parser::adsb_json::{AsdbJsonMessage, NewAsdbJsonMessage};
use std::error::Error;

use crate::common::{
    combine_files_of_message_type, compare_errors, load_files_of_message_type,
    process_file_as_adsb_json, MessageType,
};

/// This test will ingest contents from the adsb json sample files as a message per line to a `Vec<String>`.
/// It combines the two files together into a single `Vec<String>` for iterating through.
/// Then it will cycle them into `Vec<ADSBMessage>` and back to `String`.
/// It validates that there are no errors going `String` -> `ADSBMessage` and `ADSBMessage` -> `String`.
#[test]
fn test_adsb_parsing() -> Result<(), Box<dyn Error>> {
    match combine_files_of_message_type(MessageType::AsdbJson) {
        Err(load_failed) => Err(load_failed),
        Ok(adsb_messages) => {
            let mut valid_adsb_messages: Vec<AsdbJsonMessage> = Vec::new();
            let mut failed_decodes: Vec<String> = Vec::new();
            for line in adsb_messages {
                match line.to_adsb() {
                    Err(_) => failed_decodes.push(line),
                    Ok(adsb_message) => valid_adsb_messages.push(adsb_message),
                }
            }
            println!("Size of bad messages: {}", failed_decodes.len());
            for message in valid_adsb_messages {
                assert!(message.to_string().as_ref().err().is_none());
                assert!(message.to_bytes().as_ref().err().is_none());
            }
            for line in failed_decodes {
                compare_errors(line.to_adsb().err(), serde_json::from_str(&line), &line);
            }
            Ok(())
        }
    }
}

/// Test for displaying the per-item result for vdlm2 messages, helpful when diagnosing parsing issues.
/// Marked as `#[ignore]` so it can be run separately as required.
#[test]
#[ignore]
fn show_adsb_json_injest() -> Result<(), Box<dyn Error>> {
    println!("Showing vdlm2 ingest errors");
    match load_files_of_message_type(MessageType::AsdbJson) {
        Err(load_failed) => Err(load_failed),
        Ok(vdlm2_files) => {
            for file in vdlm2_files {
                println!("Testing the contents from file: {}", file.name);
                process_file_as_adsb_json(&file.contents);
            }
            Ok(())
        }
    }
}
