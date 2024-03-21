mod common;

use std::error::Error;
use acars_vdlm2_parser::hfdl::{NewHfdlMessage, HfdlMessage};
use crate::common::{combine_files_of_message_type, compare_errors, load_files_of_message_type, MessageType, process_file_as_hfdl};

/// This test will ingest contents from the hfdl sample files as a message per line to a `Vec<String>`.
/// It combines the two files together into a single `Vec<String>` for iterating through.
/// Then it will cycle them into `Vec<HfdlMessage>` and back to `String`.
/// It validates that there are no errors going `String` -> `HfdlMessage` and `HfdlMessage` -> `String`.
#[test]
fn test_hfdl_parsing() -> Result<(), Box<dyn Error>> {
    match combine_files_of_message_type(MessageType::Hfdl) {
        Err(load_failed) => Err(load_failed),
        Ok(hfdl_messages) => {
            let mut valid_hfdl_messages: Vec<HfdlMessage> = Vec::new();
            let mut failed_decodes: Vec<String> = Vec::new();
            for line in hfdl_messages {
                match line.to_hfdl() {
                    Err(_) => failed_decodes.push(line),
                    Ok(valid_entry) => valid_hfdl_messages.push(valid_entry),
                }
            }
            for message in valid_hfdl_messages {
                assert!(message.to_string().as_ref().err().is_none());
            }
            for line in failed_decodes {
                compare_errors(line.to_hfdl().err(), serde_json::from_str(&line), &line);
            }
            Ok(())
        }
    }
}

/// Test for displaying the per-item result for hfdl messages, helpful when diagnosing parsing issues.
/// Marked as `#[ignore]` so it can be run separately as required.
#[test]
#[ignore]
fn show_hfdl_ingest() -> Result<(), Box<dyn Error>> {
    println!("Showing hfdl ingest errors");
    match load_files_of_message_type(MessageType::Hfdl) {
        Err(load_failed) => Err(load_failed),
        Ok(hfdl_files) => {
            for file in hfdl_files {
                println!("Testing the contents from file: {}", file.name);
                process_file_as_hfdl(&file.contents);
            }

            Ok(())
        }
    }
}