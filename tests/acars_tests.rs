mod common;

use crate::common::{
    combine_files_of_message_type, compare_errors, load_files_of_message_type,
    process_file_as_acars, MessageType,
};
use acars_vdlm2_parser::acars::{AcarsMessage, NewAcarsMessage};
use acars_vdlm2_parser::DeserializatonError;
use std::error::Error;

/// This test will ingest contents from the acars sample files as a message per line to a `Vec<String>`.
/// It combines the two files together into a single `Vec<String>` for iterating through.
/// Then it will cycle them into `Vec<AcarsMessage>` and back to `String`.
/// It validates that there are no errors going `String` -> `AcarsMessage` and `AcarsMessage` -> `String`.
#[test]
fn test_acars_parsing() -> Result<(), Box<dyn Error>> {
    match combine_files_of_message_type(MessageType::Acars) {
        Err(load_failed) => Err(load_failed),
        Ok(acars_messages) => {
            let mut valid_acars_messages: Vec<AcarsMessage> = Vec::new();
            let mut failed_decodes: Vec<String> = Vec::new();
            for line in acars_messages {
                // determine if line is a string
                match line {
                    common::TestFileType::String(line_as_string) => match line_as_string.to_acars()
                    {
                        Err(_) => failed_decodes.push(line_as_string),
                        Ok(acars_message) => valid_acars_messages.push(acars_message),
                    },
                    common::TestFileType::U8(_) => {}
                }
            }
            for message in valid_acars_messages {
                assert!(message.to_string().as_ref().err().is_none());
                assert!(message.to_bytes().as_ref().err().is_none());
            }
            for line in failed_decodes {
                compare_errors(
                    line.to_acars().err(),
                    serde_json::from_str(&line).map_err(|e| DeserializatonError::SerdeError(e)),
                    &line,
                );
            }
            Ok(())
        }
    }
}

/// Test for displaying the per-item result for acars messages, helpful when diagnosing parsing issues.
/// Marked as `#[ignore]` so it can be run separately as required.
#[test]
#[ignore]
fn show_acars_ingest() -> Result<(), Box<dyn Error>> {
    println!("Showing acars ingest errors");
    match load_files_of_message_type(MessageType::Acars) {
        Err(load_failed) => Err(load_failed),
        Ok(acars_files) => {
            for file in acars_files {
                println!("Testing the contents from file: {}", file.name);
                process_file_as_acars(&file.contents.into_iter().collect::<Vec<String>>());
            }
            Ok(())
        }
    }
}
