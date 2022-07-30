extern crate serde;
extern crate serde_json;

use crate::acars::AcarsMessage;
use crate::vdlm2::Vdlm2Message;
use serde::{Deserialize, Serialize};

pub mod acars;
pub mod vdlm2;

/// Common return type for all serialisation/deserialisation functions.
///
/// This serves as a wrapper for `serde_json::Error` as the Error type.
pub type MessageResult<T> = Result<T, serde_json::Error>;

/// Trait for performing a decode if you wish to apply it to types other than the defaults done in this library.
///
/// The originating data must be in JSON format and have support for providing a `str`, and will not consume the source.
pub trait DecodeMessage {
    fn decode_message(&self) -> MessageResult<AcarsVdlm2Message>;
}

/// Provides functionality for decoding a `String` to `AcarsVdlm2Message`.
///
/// This does not consume the `String`.
impl DecodeMessage for String {
    fn decode_message(&self) -> MessageResult<AcarsVdlm2Message> {
        serde_json::from_str(self)
    }
}

/// Provides functionality for decoding a `str` to `AcarsVdlm2Message`.
///
/// This does not consume the `str`.
impl DecodeMessage for str {
    fn decode_message(&self) -> MessageResult<AcarsVdlm2Message> {
        serde_json::from_str(self)
    }
}

/// Implementation of `AcarsVdlm2Message`.
impl AcarsVdlm2Message {
    /// Converts `AcarsVdlm2Message` to `String`.
    pub fn to_string(&self) -> MessageResult<String> {
        serde_json::to_string(self)
    }

    /// Converts `AcarsVdlm2Message` to a `String` encoded as bytes.
    ///
    /// The output is stored returned as a `Vec<u8>`.
    pub fn to_bytes(&self) -> MessageResult<Vec<u8>> {
        let string_conversion: MessageResult<String> = self.to_string();
        match string_conversion {
            Err(conversion_failed) => Err(conversion_failed),
            Ok(string) => Ok(string.into_bytes()),
        }
    }

    /// Clears a station name that may be set for either `Vdlm2Message` or `AcarsMessage`.
    pub fn clear_station_name(&mut self) {
        match self {
            AcarsVdlm2Message::Vdlm2Message(vdlm2) =>
                vdlm2.clear_station_name(),
            AcarsVdlm2Message::AcarsMessage(acars) =>
                acars.clear_station_name(),
        }
    }

    /// Sets a station name to the provided value for either `Vdlm2Message` or `AcarsMessage`.
    pub fn set_station_name(&mut self, station_name: &str) {
        match self {
            AcarsVdlm2Message::Vdlm2Message(vdlm2) =>
                vdlm2.set_station_name(station_name),
            AcarsVdlm2Message::AcarsMessage(acars) =>
                acars.set_station_name(station_name),
        }
    }

    /// Clears any proxy details that may be set for either `Vdlm2Message` or `AcarsMessage`.
    pub fn clear_proxy_details(&mut self) {
        match self {
            AcarsVdlm2Message::Vdlm2Message(vdlm2) =>
                vdlm2.clear_proxy_details(),
            AcarsVdlm2Message::AcarsMessage(acars) =>
                acars.clear_proxy_details(),
        }
    }

    /// Sets proxy details to the provided details and sets `proxied` to true.
    ///
    /// This invokes `AppDetails::new()` for either `Vdlm2Message` or `AcarsMessage` and updates the record.
    pub fn set_proxy_details(
        &mut self,
        proxied_by: &str,
        acars_router_version: &str,
    ) {
        match self {
            AcarsVdlm2Message::Vdlm2Message(vdlm2) => {
                vdlm2.set_proxy_details(proxied_by, acars_router_version)
            }
            AcarsVdlm2Message::AcarsMessage(acars) => {
                acars.set_proxy_details(proxied_by, acars_router_version)
            }
        }
    }

    pub fn clear_time(&mut self) {
        match self {
            AcarsVdlm2Message::Vdlm2Message(vdlm2) =>
                vdlm2.clear_time(),
            AcarsVdlm2Message::AcarsMessage(acars) =>
                acars.clear_time(),
        }
    }
}

/// This will automagically serialise to either a `Vdlm2Message` or `AcarsMessage`.
///
/// This simplifies the handling of messaging by not needing to identify it first.
/// It handles identification by looking at the provided data and seeing which format matches it best.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
#[serde(untagged)]
pub enum AcarsVdlm2Message {
    Vdlm2Message(Vdlm2Message),
    AcarsMessage(AcarsMessage),
}

/// This struct lives here because it is used by both `Vdlm2Message` and `AcarsMessage`.
///
/// This does not normally exist on `AcarsMessage` and has been added as part of the implementation for the acars_router project.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct AppDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxied: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxied_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acars_router_version: Option<String>,
}

impl AppDetails {
    /// Creates a new instance of `AppDetails` with the provided details.
    /// ```
    /// use acars_vdlm2_parser::AppDetails;
    /// let manual: AppDetails = AppDetails { proxied: Some(true), proxied_by: Some("test".to_string()), acars_router_version: Some("1.0.4".to_string()) };
    /// let generated: AppDetails = AppDetails::new("test", "1.0.4");
    /// assert_eq!(manual, generated);
    /// ```
    pub fn new(proxied_by: &str, acars_router_version: &str) -> Self {
        Self {
            proxied: Some(true),
            proxied_by: Some(proxied_by.to_string()),
            acars_router_version: Some(acars_router_version.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::acars::{AcarsMessage, NewAcarsMessage};
    use crate::vdlm2::{NewVdlm2Message, Vdlm2Message};
    use glob::{glob, GlobResult, Paths, PatternError};
    use rand::rngs::ThreadRng;
    use rand::seq::SliceRandom;
    use rand::thread_rng;
    use serde_json::Value;
    use std::fs::File;
    use std::io;
    use std::io::{BufRead, BufReader};
    use std::path::Path;

    /// Enum for indicating test data type.
    enum MessageType {
        Acars,
        Vdlm2,
        All,
    }

    /// Struct for storing test information for the tests that just display error information.
    struct TestFile {
        name: String,
        contents: Vec<String>,
    }

    /// Trait for appending data.
    ///
    /// Using a trait to allow for implementation against `Vec<TestFile>`.
    trait AppendData {
        fn append_data(&mut self, file: GlobResult) -> Result<(), Box<dyn std::error::Error>>;
    }

    /// Implementing the trait `AppendData` for `Vec<TestFile>`.
    impl AppendData for Vec<TestFile> {
        /// This function exists for taking the contents of a test file and creating a new instance of `TestFile`.
        ///
        /// This is used for running the tests `show_vdlm2_ingest` and `show_acars_ingest`.
        /// These tests are ignored by default and have to be run seperately.
        fn append_data(&mut self, file: GlobResult) -> Result<(), Box<dyn std::error::Error>> {
            match file {
                Err(glob_error) => Err(glob_error.into()),
                Ok(target_file) => {
                    let open_file = File::open(target_file.as_path());
                    match open_file {
                        Err(file_error) => Err(file_error.into()),
                        Ok(file) => {
                            let read_file: io::Result<Vec<String>> =
                                BufReader::new(file).lines().collect();
                            match read_file {
                                Err(read_error) => Err(read_error.into()),
                                Ok(contents) => {
                                    let get_filename = target_file.file_name();
                                    match get_filename {
                                        None => Err("Could not get file name".into()),
                                        Some(file_name) => {
                                            let test_file: TestFile = TestFile {
                                                name: format!("{:?}", file_name),
                                                contents,
                                            };
                                            self.push(test_file);
                                            Ok(())
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Assistance function for tests to read a file, and break it up per line to a `Vec<String>`.
    ///
    /// This allows for tests to iterate through and test each line individually.
    fn read_test_file(filepath: impl AsRef<Path>) -> io::Result<Vec<String>> {
        BufReader::new(File::open(filepath)?).lines().collect()
    }

    /// Assistane function to combine contents of test files into a `Vec<String>`.
    ///
    /// This is used for combining the contents of multiple files into a single `Vec<String>` for testing.
    fn combine_found_files(
        find_files: Result<Paths, PatternError>,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        match find_files {
            Err(pattern_error) => Err(pattern_error.into()),
            Ok(file_paths) => {
                let mut loaded_contents: Vec<String> = Vec::new();
                for file in file_paths {
                    let append_data = append_lines(file, &mut loaded_contents);
                    if let Err(append_failed) = append_data {
                        return Err(append_failed);
                    }
                }
                Ok(loaded_contents.to_vec())
            }
        }
    }

    /// Assistance function for building a `Vec<TestFile>` for use with the tests that show parsing output.
    fn load_found_files(
        find_files: Result<Paths, PatternError>,
    ) -> Result<Vec<TestFile>, Box<dyn std::error::Error>> {
        match find_files {
            Err(pattern_error) => Err(pattern_error.into()),
            Ok(file_paths) => {
                let mut test_files: Vec<TestFile> = Vec::new();
                for file in file_paths {
                    let load_test_file = test_files.append_data(file);
                    if let Err(load_failed) = load_test_file {
                        return Err(load_failed);
                    }
                }
                Ok(test_files)
            }
        }
    }

    /// Assistance function for appending file contents.
    fn append_lines(
        file: GlobResult,
        data: &mut Vec<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match file {
            Err(file_error) => Err(file_error.into()),
            Ok(file_path) => {
                let file_contents: io::Result<Vec<String>> = read_test_file(file_path.as_path());
                match file_contents {
                    Err(read_error) => Err(read_error.into()),
                    Ok(contents) => {
                        for line in contents {
                            data.push(line)
                        }
                        Ok(())
                    }
                }
            }
        }
    }

    /// Assistance function that combines contents of message type test files.
    fn combine_files_of_message_type(
        message_type: MessageType,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        match message_type {
            MessageType::Acars => {
                let find_files: Result<Paths, PatternError> = glob("test_files/acars*");
                combine_found_files(find_files)
            }
            MessageType::Vdlm2 => {
                let find_files: Result<Paths, PatternError> = glob("test_files/vdlm2*");
                combine_found_files(find_files)
            }
            MessageType::All => {
                let find_files: Result<Paths, PatternError> = glob("test_files/*");
                combine_found_files(find_files)
            }
        }
    }

    /// Assistance function that loads contents of individual message type test files and returns them separately instead of combined.
    fn load_files_of_message_type(
        message_type: MessageType,
    ) -> Result<Vec<TestFile>, Box<dyn std::error::Error>> {
        match message_type {
            MessageType::Acars => {
                let find_files: Result<Paths, PatternError> = glob("test_files/acars*");
                load_found_files(find_files)
            }
            MessageType::Vdlm2 => {
                let find_files: Result<Paths, PatternError> = glob("test_files/vdlm2*");
                load_found_files(find_files)
            }
            MessageType::All => {
                let find_files: Result<Paths, PatternError> = glob("test_files/*");
                load_found_files(find_files)
            }
        }
    }

    /// Assistance function for processing the contents of a `&[String]` slice as vdlm2 messages.
    fn process_file_as_vdlm2(contents: &[String]) {
        let contents: Vec<String> = contents.to_vec();
        let mut errors: Vec<String> = Vec::new();
        for (entry, line) in contents.iter().enumerate() {
            let parse_line: MessageResult<Vdlm2Message> = line.to_vdlm2();
            if let Err(parse_error) = parse_line {
                let error_text: String = format!(
                    "Entry {} parse error: {}\nData: {}",
                    entry + 1,
                    parse_error,
                    line
                );
                errors.push(error_text);
            }
        }
        match errors.is_empty() {
            true => println!("No errors found in provided contents"),
            false => {
                println!("Errors found as follows");
                for error in errors {
                    println!("{}", error);
                }
            }
        }
    }

    /// Assistance function for processing the contents of a `&[String]` slice as acars messages.
    fn process_file_as_acars(contents: &[String]) {
        let contents: Vec<String> = contents.to_vec();
        let mut errors: Vec<String> = Vec::new();
        for (entry, line) in contents.iter().enumerate() {
            let parse_line: MessageResult<AcarsMessage> = line.to_acars();
            if let Err(parse_error) = parse_line {
                let error_text: String = format!(
                    "Entry {} parse error: {}\nData: {}",
                    entry + 1,
                    parse_error,
                    line
                );
                errors.push(error_text);
            }
        }
        match errors.is_empty() {
            true => println!("No errors found in provided contents"),
            false => {
                println!("Errors found as follows");
                for error in errors {
                    println!("{}", error);
                }
            }
        }
    }

    /// Assistance function to compare error message strings between Library result and serde `Value` result.
    fn compare_errors(
        error_1: Option<serde_json::Error>,
        error_2: Result<Value, serde_json::Error>,
        line: &str,
    ) {
        if let (Some(library_error), Err(serde_error)) = (error_1, error_2) {
            let serde_error_string: String = serde_error.to_string();
            assert_eq!(
                library_error.to_string(),
                serde_error_string,
                "Errors processing {} do not match between library {} and serde Value {}",
                line,
                library_error.to_string(),
                serde_error_string
            );
        }
    }

    fn test_enum_serialisation(message: AcarsVdlm2Message) {
        let encoded_string: MessageResult<String> = message.to_string();
        assert_eq!(
            encoded_string.as_ref().err().is_none(),
            true,
            "Parsing data {:?} to String failed: {:?}",
            message,
            encoded_string.as_ref().err()
        );
        let encoded_bytes: MessageResult<Vec<u8>> = message.to_bytes();
        assert_eq!(
            encoded_bytes.as_ref().err().is_none(),
            true,
            "Parsing data {:?} to bytes failed: {:?}",
            message,
            encoded_bytes.as_ref().err()
        );
    }

    /// This test will ingest contents from the vdlm2 sample files as a message per line to a `Vec<String>`.
    /// It combines the two files together into a single `Vec<String>` for iterating through.
    /// Then it will cycle them into `Vec<Vdlm2Message>` and back to `String`.
    /// It validates that there are no errors going `String` -> `Vdlm2Message` and `Vdlm2Message` -> `String`.
    #[test]
    fn test_vdlm2_parsing() -> Result<(), Box<dyn std::error::Error>> {
        let load_vdlm_messages: Result<Vec<String>, Box<dyn std::error::Error>> =
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
                    assert_eq!(vdlm2_to_string.as_ref().err().is_none(), true);
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
    fn show_vdlm2_ingest() -> Result<(), Box<dyn std::error::Error>> {
        println!("Showing vdlm2 ingest errors");
        let load_vdlm2_files: Result<Vec<TestFile>, Box<dyn std::error::Error>> =
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

    /// This test will ingest contents from the acars sample files as a message per line to a `Vec<String>`.
    /// It combines the two files together into a single `Vec<String>` for iterating through.
    /// Then it will cycle them into `Vec<AcarsMessage>` and back to `String`.
    /// It validates that there are no errors going `String` -> `AcarsMessage` and `AcarsMessage` -> `String`.
    #[test]
    fn test_acars_parsing() -> Result<(), Box<dyn std::error::Error>> {
        let load_acars_messages: Result<Vec<String>, Box<dyn std::error::Error>> =
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
    fn show_acars_ingest() -> Result<(), Box<dyn std::error::Error>> {
        println!("Showing acars ingest errors");
        let load_acars_files: Result<Vec<TestFile>, Box<dyn std::error::Error>> =
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

    /// This test ingests the contents of all the Acars and Vdlm2 sample files into individual `Vec<String>`.
    /// Then it combines two pairs of known good into a single `Vec<String>` and randomises the ordering.
    /// Then it will cycle them into `Vec<AcarsVdlm2Message>` and back to `String`.
    /// It validates that there are no errors going `String` -> `AcarsVdlm2Message` and `AcarsVdlm2Message` -> `String`.
    /// It then combines two files containing known bad data into a single `Vec<String>` and randomises the ordering.
    /// It validates that it gets errors that it is expecting and the correct number of errors.
    #[test]
    fn test_determining_message() -> Result<(), Box<dyn std::error::Error>> {
        let load_all_messages: Result<Vec<String>, Box<dyn std::error::Error>> =
            combine_files_of_message_type(MessageType::All);
        match load_all_messages {
            Err(load_error) => Err(load_error),
            Ok(mut all_messages) => {
                let mut rng: ThreadRng = thread_rng();
                let mut successfully_decoded_items: Vec<AcarsVdlm2Message> = Vec::new();
                let mut failed_decodes: Vec<String> = Vec::new();
                all_messages.shuffle(&mut rng);
                for entry in all_messages {
                    let parsed_message: MessageResult<AcarsVdlm2Message> = entry.decode_message();
                    match parsed_message {
                        Err(_) =>
                            failed_decodes.push(entry),
                        Ok(decoded_message) =>
                            successfully_decoded_items.push(decoded_message),
                    }
                }
                successfully_decoded_items.shuffle(&mut rng);
                for message in successfully_decoded_items {
                    test_enum_serialisation(message);
                }
                for line in failed_decodes {
                    let library_parse_error: Option<serde_json::Error> =
                        line.decode_message().err();
                    let serde_value_error: Result<Value, serde_json::Error> =
                        serde_json::from_str(&line);
                    compare_errors(library_parse_error, serde_value_error, &line);
                }
                Ok(())
            }
        }
    }
}
