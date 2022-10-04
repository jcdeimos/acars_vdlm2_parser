mod common;

use std::error::Error;
use serde_json::Value;
use acars_vdlm2_parser::MessageResult;
use acars_vdlm2_parser::vdlm2::{NewVdlm2Message, Vdlm2Message};
use crate::common::{combine_files_of_message_type, compare_errors, load_files_of_message_type, MessageType, process_file_as_vdlm2, TestFile};

/// This test will ingest contents from the vdlm2 sample files as a message per line to a `Vec<String>`.
/// It combines the two files together into a single `Vec<String>` for iterating through.
/// Then it will cycle them into `Vec<Vdlm2Message>` and back to `String`.
/// It validates that there are no errors going `String` -> `Vdlm2Message` and `Vdlm2Message` -> `String`.
#[test]
fn test_vdlm2_parsing() -> Result<(), Box<dyn Error>> {
    let load_vdlm_messages: Result<Vec<String>, Box<dyn Error>> =
        combine_files_of_message_type(MessageType::Vdlm2);
    match load_vdlm_messages {
        Err(load_failed) => Err(load_failed),
        Ok(vdlm2_messages) => {
            let mut valid_vdlm2_messages: Vec<Vdlm2Message> = Vec::new();
            let mut failed_decodes: Vec<String> = Vec::new();
            for line in vdlm2_messages {
                let parse_line: MessageResult<Vdlm2Message> = line.to_vdlm2();
                match parse_line {
                    Err(_) => failed_decodes.push(line),
                    Ok(valid_entry) => valid_vdlm2_messages.push(valid_entry),
                }
            }
            for message in valid_vdlm2_messages {
                let vdlm2_to_string: MessageResult<String> = message.to_string();
                assert!(vdlm2_to_string.as_ref().err().is_none());
            }
            for line in failed_decodes {
                let library_parse_error: Option<serde_json::Error> = line.to_vdlm2().err();
                let serde_value_error: Result<Value, serde_json::Error> =
                    serde_json::from_str(&line);
                compare_errors(library_parse_error, serde_value_error, &line);
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
    let load_vdlm2_files: Result<Vec<TestFile>, Box<dyn Error>> =
        load_files_of_message_type(MessageType::Vdlm2);
    match load_vdlm2_files {
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