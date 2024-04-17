mod common;

use std::error::Error;
use acars_vdlm2_parser::irdm::{NewIrdmMessage, IrdmMessage};
use crate::common::{combine_files_of_message_type, compare_errors, load_files_of_message_type, MessageType, process_file_as_irdm};

/// This test will ingest contents from the irdm sample files as a message per line to a `Vec<String>`.
/// It combines the two files together into a single `Vec<String>` for iterating through.
/// Then it will cycle them into `Vec<IrdmMessage>` and back to `String`.
/// It validates that there are no errors going `String` -> `IrdmMessage` and `IrdmMessage` -> `String`.
#[test]
fn test_irdm_parsing() -> Result<(), Box<dyn Error>> {
    match combine_files_of_message_type(MessageType::Irdm) {
        Err(load_failed) => Err(load_failed),
        Ok(irdm_messages) => {
            let mut valid_irdm_messages: Vec<IrdmMessage> = Vec::new();
            let mut failed_decodes: Vec<String> = Vec::new();
            for line in irdm_messages {
                match line.to_irdm() {
                    Err(_) => failed_decodes.push(line),
                    Ok(valid_entry) => valid_irdm_messages.push(valid_entry),
                }
            }
            for message in valid_irdm_messages {
                assert!(message.to_string().as_ref().err().is_none());
            }
            for line in failed_decodes {
                compare_errors(line.to_irdm().err(), serde_json::from_str(&line), &line);
            }
            Ok(())
        }
    }
}

/// Test for displaying the per-item result for irdm messages, helpful when diagnosing parsing issues.
/// Marked as `#[ignore]` so it can be run separately as required.
#[test]
#[ignore]
fn show_irdm_ingest() -> Result<(), Box<dyn Error>> {
    println!("Showing irdm ingest errors");
    match load_files_of_message_type(MessageType::Irdm) {
        Err(load_failed) => Err(load_failed),
        Ok(irdm_files) => {
            for file in irdm_files {
                println!("Testing the contents from file: {}", file.name);
                process_file_as_irdm(&file.contents);
            }

            Ok(())
        }
    }
}
