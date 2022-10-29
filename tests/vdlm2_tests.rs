mod common;

use std::error::Error;
use acars_vdlm2_parser::vdlm2::{NewVdlm2Message, Vdlm2Message};
use crate::common::{combine_files_of_message_type, compare_errors, load_files_of_message_type, MessageType, process_file_as_vdlm2};

/// This test will ingest contents from the vdlm2 sample files as a message per line to a `Vec<String>`.
/// It combines the two files together into a single `Vec<String>` for iterating through.
/// Then it will cycle them into `Vec<Vdlm2Message>` and back to `String`.
/// It validates that there are no errors going `String` -> `Vdlm2Message` and `Vdlm2Message` -> `String`.
#[test]
fn test_vdlm2_parsing() -> Result<(), Box<dyn Error>> {
    match combine_files_of_message_type(MessageType::Vdlm2) {
        Err(load_failed) => Err(load_failed),
        Ok(vdlm2_messages) => {
            let mut valid_vdlm2_messages: Vec<Vdlm2Message> = Vec::new();
            let mut failed_decodes: Vec<String> = Vec::new();
            for line in vdlm2_messages {
                match line.to_vdlm2() {
                    Err(_) => failed_decodes.push(line),
                    Ok(valid_entry) => valid_vdlm2_messages.push(valid_entry),
                }
            }
            for message in valid_vdlm2_messages {
                assert!(message.to_string().as_ref().err().is_none());
            }
            for line in failed_decodes {
                compare_errors(line.to_vdlm2().err(), serde_json::from_str(&line), &line);
            }
            Ok(())
        }
    }
}

/// Test for displaying the per-item result for vdlm2 messages, helpful when diagnosing parsing issues.
/// Marked as `#[ignore]` so it can be run separately as required.
#[test]
#[ignore]
fn show_vdlm2_ingest() -> Result<(), Box<dyn Error>> {
    println!("Showing vdlm2 ingest errors");
    match load_files_of_message_type(MessageType::Vdlm2) {
        Err(load_failed) => Err(load_failed),
        Ok(vdlm2_files) => {
            for file in vdlm2_files {
                println!("Testing the contents from file: {}", file.name);
                process_file_as_vdlm2(&file.contents);
            }
            Ok(())
        }
    }
}