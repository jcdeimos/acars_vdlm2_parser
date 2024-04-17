mod common;

use std::error::Error;
use acars_vdlm2_parser::imsl::{NewImslMessage, ImslMessage};
use crate::common::{combine_files_of_message_type, compare_errors, load_files_of_message_type, MessageType, process_file_as_imsl};

/// This test will ingest contents from the imsl sample files as a message per line to a `Vec<String>`.
/// It combines the two files together into a single `Vec<String>` for iterating through.
/// Then it will cycle them into `Vec<ImslMessage>` and back to `String`.
/// It validates that there are no errors going `String` -> `ImslMessage` and `ImslMessage` -> `String`.
#[test]
fn test_imsl_parsing() -> Result<(), Box<dyn Error>> {
    match combine_files_of_message_type(MessageType::Imsl) {
        Err(load_failed) => Err(load_failed),
        Ok(imsl_messages) => {
            let mut valid_imsl_messages: Vec<ImslMessage> = Vec::new();
            let mut failed_decodes: Vec<String> = Vec::new();
            for line in imsl_messages {
                match line.to_imsl() {
                    Err(_) => failed_decodes.push(line),
                    Ok(valid_entry) => valid_imsl_messages.push(valid_entry),
                }
            }
            for message in valid_imsl_messages {
                assert!(message.to_string().as_ref().err().is_none());
            }
            for line in failed_decodes {
                compare_errors(line.to_imsl().err(), serde_json::from_str(&line), &line);
            }
            Ok(())
        }
    }
}

/// Test for displaying the per-item result for imsl messages, helpful when diagnosing parsing issues.
/// Marked as `#[ignore]` so it can be run separately as required.
#[test]
#[ignore]
fn show_imsl_ingest() -> Result<(), Box<dyn Error>> {
    println!("Showing imsl ingest errors");
    match load_files_of_message_type(MessageType::Imsl) {
        Err(load_failed) => Err(load_failed),
        Ok(imsl_files) => {
            for file in imsl_files {
                println!("Testing the contents from file: {}", file.name);
                process_file_as_imsl(&file.contents);
            }

            Ok(())
        }
    }
}
