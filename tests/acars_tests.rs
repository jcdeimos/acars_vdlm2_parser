mod common;

use std::error::Error;
use serde_json::Value;
use acars_vdlm2_parser::acars::{AcarsMessage, NewAcarsMessage};
use acars_vdlm2_parser::MessageResult;
use crate::common::{combine_files_of_message_type, compare_errors, load_files_of_message_type, MessageType, process_file_as_acars, TestFile};

/// This test will ingest contents from the acars sample files as a message per line to a `Vec<String>`.
/// It combines the two files together into a single `Vec<String>` for iterating through.
/// Then it will cycle them into `Vec<AcarsMessage>` and back to `String`.
/// It validates that there are no errors going `String` -> `AcarsMessage` and `AcarsMessage` -> `String`.
#[test]
fn test_acars_parsing() -> Result<(), Box<dyn Error>> {
    let load_acars_messages: Result<Vec<String>, Box<dyn Error>> =
        combine_files_of_message_type(MessageType::Acars);
    match load_acars_messages {
        Err(load_failed) => Err(load_failed),
        Ok(acars_messages) => {
            let mut valid_acars_messages: Vec<AcarsMessage> = Vec::new();
            let mut failed_decodes: Vec<String> = Vec::new();
            for line in acars_messages {
                let parse_line: MessageResult<AcarsMessage> = line.to_acars();
                match parse_line {
                    Err(_) => failed_decodes.push(line),
                    Ok(acars_message) => valid_acars_messages.push(acars_message),
                }
            }
            for message in valid_acars_messages {
                let acars_to_string: MessageResult<String> = message.to_string();
                assert_eq!(acars_to_string.as_ref().err().is_none(), true);
                let acars_to_bytes: MessageResult<Vec<u8>> = message.to_bytes();
                assert_eq!(acars_to_bytes.as_ref().err().is_none(), true);
            }
            for line in failed_decodes {
                let library_parse_error: Option<serde_json::Error> = line.to_acars().err();
                let serde_value_error: Result<Value, serde_json::Error> =
                    serde_json::from_str(&line);
                compare_errors(library_parse_error, serde_value_error, &line);
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
    let load_acars_files: Result<Vec<TestFile>, Box<dyn Error>> =
        load_files_of_message_type(MessageType::Acars);
    match load_acars_files {
        Err(load_failed) => Err(load_failed),
        Ok(acars_files) => {
            for file in acars_files {
                println!("Testing the contents from file: {}", file.name);
                process_file_as_acars(&file.contents);
            }
            Ok(())
        }
    }
}