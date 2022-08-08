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
    
    /// Converts `AcarsVdlm2Message` to `String` and appends a `\n` to the end.
    pub fn to_string_newline(&self) -> MessageResult<String> {
        let data = serde_json::to_string(self);
        match data {
            Err(to_string_error) => Err(to_string_error),
            Ok(string) => Ok(format!("{}\n", string))
        }
    }

    /// Converts `AcarsVdlm2Message` to a `String` encoded as bytes.
    ///
    /// The output is returned as a `Vec<u8>`.
    pub fn to_bytes(&self) -> MessageResult<Vec<u8>> {
        let string_conversion: MessageResult<String> = self.to_string();
        match string_conversion {
            Err(conversion_failed) => Err(conversion_failed),
            Ok(string) => Ok(string.into_bytes()),
        }
    }
    
    /// Converts `AcarsVdlm2Message` to a `String` terminated with a `\n` and encoded as bytes.
    ///
    /// The output is returned as a `Vec<u8>`.
    pub fn to_bytes_newline(&self) -> MessageResult<Vec<u8>> {
        let string_conversion: MessageResult<String> = self.to_string_newline();
        match string_conversion {
            Err(conversion_failed) => Err(conversion_failed),
            Ok(string) => Ok(string.into_bytes())
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

    /// Clears the time details from the message.
    pub fn clear_time(&mut self) {
        match self {
            AcarsVdlm2Message::Vdlm2Message(vdlm2) =>
                vdlm2.clear_time(),
            AcarsVdlm2Message::AcarsMessage(acars) =>
                acars.clear_time(),
        }
    }

    /// Retrieves the time information from the message.
    pub fn get_time(&self) -> Option<f64> {
        match self {
            AcarsVdlm2Message::Vdlm2Message(vdlm2) =>
                vdlm2.get_time(),
            AcarsVdlm2Message::AcarsMessage(acars) =>
                acars.get_time(),
        }
    }
    
    /// Clears the `freq_skew` field from a `Vdlm2Message`.
    pub fn clear_freq_skew(&mut self) {
        match self {
            AcarsVdlm2Message::Vdlm2Message(vdlm2) => vdlm2.clear_freq_skew(),
            AcarsVdlm2Message::AcarsMessage(_) => {}
        }
    }
    
    /// Clears the `hdr_bits_fixed` field from a `Vdlm2Message`.
    pub fn clear_hdr_bits_fixed(&mut self) {
        match self {
            AcarsVdlm2Message::Vdlm2Message(vdlm2) => vdlm2.clear_hdr_bits_fixed(),
            AcarsVdlm2Message::AcarsMessage(_) => {}
        }
    }
    
    /// Clears the `noise_level` field from a `Vdlm2Message`.
    pub fn clear_noise_level(&mut self) {
        match self {
            AcarsVdlm2Message::Vdlm2Message(vdlm2) => vdlm2.clear_noise_level(),
            AcarsVdlm2Message::AcarsMessage(_) => {}
        }
    }
    
    /// Clears the `octets_corrected_by_fec` field from a `Vdlm2Message`.
    pub fn clear_octets_corrected_by_fec(&mut self) {
        match self {
            AcarsVdlm2Message::Vdlm2Message(vdlm2) => vdlm2.clear_octets_corrected_by_fec(),
            AcarsVdlm2Message::AcarsMessage(_) => {}
        }
    }
    
    /// Clears the `sig_level` field from a `Vdlm2Message`.
    pub fn clear_sig_level(&mut self) {
        match self {
            AcarsVdlm2Message::Vdlm2Message(vdlm2) => vdlm2.clear_sig_level(),
            AcarsVdlm2Message::AcarsMessage(_) => {}
        }
    }
    
    /// Clears the `channel` field from a `AcarsMessage`.
    pub fn clear_channel(&mut self) {
        match self {
            AcarsVdlm2Message::Vdlm2Message(_) => {}
            AcarsVdlm2Message::AcarsMessage(acars) => acars.clear_channel()
        }
    }
    
    /// Clears the `error` field from a `AcarsMessage`.
    pub fn clear_error(&mut self) {
        match self {
            AcarsVdlm2Message::Vdlm2Message(_) => {}
            AcarsVdlm2Message::AcarsMessage(acars) => acars.clear_error()
        }
    }
    
    /// Clears the `level` field from a `AcarsMessage`.
    pub fn clear_level(&mut self) {
        match self {
            AcarsVdlm2Message::Vdlm2Message(_) => {}
            AcarsVdlm2Message::AcarsMessage(acars) => acars.clear_level()
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
    pub name: String,
    pub ver: String,
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
    /// let manual: AppDetails = AppDetails { name: "".to_string(), ver: "".to_string(),proxied: Some(true), proxied_by: Some("test".to_string()), acars_router_version: Some("1.0.4".to_string()) };
    /// let generated: AppDetails = AppDetails::new("test", "1.0.4");
    /// assert_eq!(manual, generated);
    /// ```
    pub fn new(proxied_by: &str, acars_router_version: &str) -> Self {
        Self {
            name: "".to_string(),
            ver: "".to_string(),
            proxied: Some(true),
            proxied_by: Some(proxied_by.to_string()),
            acars_router_version: Some(acars_router_version.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::ffi::OsStr;
    use super::*;
    use std::fs::File;
    use std::io;
    use std::io::{BufRead, BufReader};
    use std::path::Path;
    use chrono::{DateTime, Duration, Utc};
    use glob::{glob, GlobResult, Paths, PatternError};
    use rand::rngs::ThreadRng;
    use rand::seq::SliceRandom;
    use rand::thread_rng;
    use serde_json::Value;
    use crate::vdlm2::{NewVdlm2Message, Vdlm2Message};
    use crate::acars::{AcarsMessage, NewAcarsMessage};

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
    
    struct Stopwatch {
        timer: String,
        start_time: Option<DateTime<Utc>>,
        stop_time: Option<DateTime<Utc>>,
        duration_ms: Option<i64>,
        duration_ns: Option<i64>
        
    }
    
    impl Stopwatch {
        fn start(timer: &str) -> Self {
            Self {
                timer: timer.to_string(),
                start_time: Some(Utc::now()),
                stop_time: None,
                duration_ms: None,
                duration_ns: None
            }
        }
        fn stop(&mut self) {
            self.stop_time = Some(Utc::now());
            if let (Some(stop), Some(start)) = (self.stop_time, self.start_time) {
                let duration: Duration = stop - start;
                self.duration_ms = Some(duration.num_milliseconds());
                self.duration_ns = duration.num_nanoseconds();
            }
        }
        fn duration_ms(&self) -> Option<i64> {
            match self.duration_ms {
                Some(duration) => Some(duration),
                None => None
            }
        }
        fn duration_ns(&self) -> Option<i64> {
            match self.duration_ns {
                Some(duration) => Some(duration),
                None => None
            }
        }
    }
    
    struct RunDurations {
        all_deser_run_ms: Vec<i64>,
        all_deser_run_ns: Vec<i64>,
        all_ser_run_ms: Vec<i64>,
        all_ser_run_ns: Vec<i64>,
        add_ser_run_ms: Vec<i64>,
        add_ser_run_ns: Vec<i64>
    }
    
    impl RunDurations {
        fn new() -> Self {
            Self {
                all_deser_run_ms: Vec::new(),
                all_deser_run_ns: Vec::new(),
                all_ser_run_ms: Vec::new(),
                all_ser_run_ns: Vec::new(),
                add_ser_run_ms: Vec::new(),
                add_ser_run_ns: Vec::new()
            }
        }
        fn deser_ms(&mut self, duration: &i64) {
            self.all_deser_run_ms.push(*duration);
        }
        fn deser_ns(&mut self, duration: &i64) {
            self.all_deser_run_ns.push(*duration);
        }
        fn ser_ms(&mut self, duration: &i64) {
            self.all_ser_run_ms.push(*duration);
        }
        fn ser_ns(&mut self, duration: &i64) {
            self.all_ser_run_ns.push(*duration);
        }
        fn add_ser_ms(&mut self, duration: &i64) {
            self.add_ser_run_ms.push(*duration);
        }
        fn add_ser_ns(&mut self, duration: &i64) {
            self.add_ser_run_ns.push(*duration);
        }
    }

    /// Trait for appending data.
    ///
    /// Using a trait to allow for implementation against `Vec<TestFile>`.
    trait AppendData {
        fn append_data(&mut self, file: GlobResult) -> Result<(), Box<dyn Error>>;
    }

    /// Implementing the trait `AppendData` for `Vec<TestFile>`.
    impl AppendData for Vec<TestFile> {
        /// This function exists for taking the contents of a test file and creating a new instance of `TestFile`.
        ///
        /// This is used for running the tests `show_vdlm2_ingest` and `show_acars_ingest`.
        /// These tests are ignored by default and have to be run seperately.
        fn append_data(&mut self, file: GlobResult) -> Result<(), Box<dyn Error>> {
            match file {
                Err(glob_error) => Err(glob_error.into()),
                Ok(target_file) => {
                    let open_file: Result<File, io::Error> = File::open(target_file.as_path());
                    match open_file {
                        Err(file_error) => Err(file_error.into()),
                        Ok(file) => {
                            let read_file: Result<Vec<String>, io::Error> =
                                BufReader::new(file).lines().collect();
                            match read_file {
                                Err(read_error) => Err(read_error.into()),
                                Ok(contents) => {
                                    let get_filename: Option<&OsStr> = target_file.file_name();
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
    ) -> Result<Vec<String>, Box<dyn Error>> {
        match find_files {
            Err(pattern_error) => Err(pattern_error.into()),
            Ok(file_paths) => {
                let mut loaded_contents: Vec<String> = Vec::new();
                for file in file_paths {
                    let append_data: Result<(), Box<dyn Error>> = append_lines(file, &mut loaded_contents);
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
    ) -> Result<Vec<TestFile>, Box<dyn Error>> {
        match find_files {
            Err(pattern_error) => Err(pattern_error.into()),
            Ok(file_paths) => {
                let mut test_files: Vec<TestFile> = Vec::new();
                for file in file_paths {
                    let load_test_file: Result<(), Box<dyn Error>> = test_files.append_data(file);
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
    ) -> Result<(), Box<dyn Error>> {
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
    ) -> Result<Vec<String>, Box<dyn Error>> {
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
    ) -> Result<Vec<TestFile>, Box<dyn Error>> {
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

    fn test_enum_serialisation(message: &AcarsVdlm2Message) {
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
    
    trait ContentDuplicator {
        fn duplicate_contents(&self, rounds: &i32) -> Self;
    }
    
    impl ContentDuplicator for Vec<String> {
        fn duplicate_contents(&self, rounds: &i32) -> Self {
            let mut duplicated_contents: Vec<String> = Vec::new();
            let mut data: Vec<String> = self.to_vec();
            let mut rng: ThreadRng = thread_rng();
            for _ in 0..*rounds {
                data.shuffle(&mut rng);
                for entry in &data {
                    duplicated_contents.push(entry.to_string());
                }
            }
            duplicated_contents
        }
    }
    
    /// Trait for performing speed tests.
    trait SpeedTest {
        fn speed_test(&self) -> Result<(), Box<dyn Error>>;
    }
    
    /// `SpeedTest` implemented for `i32`
    ///
    /// Run x iterations, invoked as `int.speed_test()`
    impl SpeedTest for i32 {
        fn speed_test(&self) -> Result<(), Box<dyn Error>> {
            println!("Starting a speed test of {} rounds", self);
            let load_all_messages: Result<Vec<String>, Box<dyn Error>> =
                combine_files_of_message_type(MessageType::All);
            match load_all_messages {
                Err(load_error) => Err(load_error),
                Ok(mut all_messages) => {
                    println!("Loaded data successfully");
                    let mut rng: ThreadRng = thread_rng();
                    let mut successfully_decoded_items: Vec<AcarsVdlm2Message> = Vec::new();
                    let mut run_durations: RunDurations = RunDurations::new();
                    let mut total_run_stopwatch: Stopwatch = Stopwatch::start("Total Run");
                    for run in 0..*self {
                        println!("Run {} =>", run);
                        all_messages.shuffle(&mut rng);
                        let mut run_deserialisation_successful_items: Vec<AcarsVdlm2Message> = Vec::new();
                        let mut deserialisation_run_stopwatch: Stopwatch = Stopwatch::start("Deserialisation");
                        for entry in &all_messages {
                            let parsed_message: MessageResult<AcarsVdlm2Message> = entry.decode_message();
                            match parsed_message {
                                Err(_) => {}
                                Ok(decoded_message) => {
                                    successfully_decoded_items.push(decoded_message.clone());
                                    run_deserialisation_successful_items.push(decoded_message.clone());
                                }
                            }
                        }
                        deserialisation_run_stopwatch.stop();
                        println!("Run added {} successful items", run_deserialisation_successful_items.len());
                        match (deserialisation_run_stopwatch.duration_ms(), deserialisation_run_stopwatch.duration_ns()) {
                            (Some(ms), None) => {
                                println!("{} duration: {}ms", ms, deserialisation_run_stopwatch.timer);
                                run_durations.deser_ms(&ms);
                            },
                            (None, Some(ns)) => {
                                println!("{} duration: {}ns", deserialisation_run_stopwatch.timer, ns);
                                run_durations.deser_ns(&ns);
                            },
                            (Some(ms), Some(ns)) => {
                                println!("{} duration: {}ms ({}ns)", deserialisation_run_stopwatch.timer, ms, ns);
                                run_durations.deser_ms(&ms);
                                run_durations.deser_ns(&ns);
                            },
                            (_, _) => println!("Unknown {} duration for this run", deserialisation_run_stopwatch.timer)
                        }
                        successfully_decoded_items.shuffle(&mut rng);
                        run_deserialisation_successful_items.shuffle(&mut rng);
                        let mut serialisation_run_stopwatch: Stopwatch = Stopwatch::start("Serialisation");
                        for message in &run_deserialisation_successful_items {
                            test_enum_serialisation(message);
                        }
                        serialisation_run_stopwatch.stop();
                        match (serialisation_run_stopwatch.duration_ms(), serialisation_run_stopwatch.duration_ns()) {
                            (Some(ms), None) => {
                                println!("{} duration: {}ms", ms, serialisation_run_stopwatch.timer);
                                run_durations.ser_ms(&ms);
                            },
                            (None, Some(ns)) => {
                                println!("{} duration: {}ns", serialisation_run_stopwatch.timer, ns);
                                run_durations.ser_ns(&ns);
                            },
                            (Some(ms), Some(ns)) => {
                                println!("{} duration: {}ms ({}ns)", serialisation_run_stopwatch.timer, ms, ns);
                                run_durations.ser_ms(&ms);
                                run_durations.ser_ns(&ns);
                            },
                            (_, _) => println!("Unknown {} duration for this run", serialisation_run_stopwatch.timer)
                        }
                        println!("Decoded items now contains {} items", successfully_decoded_items.len());
                        let mut additive_serialisation_run_stopwatch: Stopwatch = Stopwatch::start("Cumulative Run Serialisation");
                        for message in &successfully_decoded_items {
                            test_enum_serialisation(message);
                        }
                        additive_serialisation_run_stopwatch.stop();
                        match (additive_serialisation_run_stopwatch.duration_ms(), additive_serialisation_run_stopwatch.duration_ns()) {
                            (Some(ms), None) => {
                                println!("{} duration: {}ms", ms, additive_serialisation_run_stopwatch.timer);
                                run_durations.add_ser_ms(&ms);
                            },
                            (None, Some(ns)) => {
                                println!("{} duration: {}ns", additive_serialisation_run_stopwatch.timer, ns);
                                run_durations.add_ser_ns(&ns);
                            },
                            (Some(ms), Some(ns)) => {
                                println!("{} duration: {}ms ({}ns)", additive_serialisation_run_stopwatch.timer, ms, ns);
                                run_durations.add_ser_ms(&ms);
                                run_durations.add_ser_ns(&ns);
                            },
                            (_, _) => println!("Unknown {} duration for this run", additive_serialisation_run_stopwatch.timer)
                        }
                    }
                    successfully_decoded_items.shuffle(&mut rng);
                    let mut final_cumulative_serialisation_stopwatch: Stopwatch = Stopwatch::start("Cumulative Serialisation");
                    for message in &successfully_decoded_items {
                        test_enum_serialisation(message);
                    }
                    final_cumulative_serialisation_stopwatch.stop();
                    total_run_stopwatch.stop();
                    run_durations.add_ser_run_ms.sort_by(|a, b | a.cmp(&b));
                    let mut additive_serialisation_run_gaps: Vec<i64> = run_durations.add_ser_run_ms.windows(2).map(|w| w[1] - w[0]).collect::<Vec<i64>>();
                    additive_serialisation_run_gaps.sort_by(|a, b| a.cmp(&b));
                    let shortest_additive_serialisation_run_gaps: Option<&i64> = additive_serialisation_run_gaps.first();
                    let longest_additive_serialisation_run_gaps: Option<&i64> = additive_serialisation_run_gaps.last();
                    let middle_additive_serialisation_run_gaps: usize = additive_serialisation_run_gaps.len() / 2;
                    let average_additive_serialisation_run_gaps: i64 = additive_serialisation_run_gaps[middle_additive_serialisation_run_gaps];
                    run_durations.all_deser_run_ms.sort_by(|a, b | a.cmp(&b));
                    let shortest_deserialisation_run: Option<&i64> = run_durations.all_deser_run_ms.first();
                    let longest_deserialisation_run: Option<&i64> = run_durations.all_deser_run_ms.last();
                    let middle_deserialisation_run: usize = run_durations.all_deser_run_ms.len() / 2;
                    let average_deserialisation_run: i64 = run_durations.all_deser_run_ms[middle_deserialisation_run];
                    run_durations.all_ser_run_ms.sort_by(|a, b| a.cmp(&b));
                    let shortest_serialisation_run: Option<&i64> = run_durations.all_ser_run_ms.first();
                    let longest_serialisation_run: Option<&i64> = run_durations.all_ser_run_ms.last();
                    let middle_serialisation_run: usize = run_durations.all_ser_run_ms.len() / 2;
                    let average_serialisation_run: i64 = run_durations.all_ser_run_ms[middle_serialisation_run];
                    println!("Speed test completed!");
                    println!("Total Serialisation runs: {}", run_durations.add_ser_run_ms.len());
                    output_speed_test_ranges("Serialisation", shortest_serialisation_run, longest_serialisation_run, &average_serialisation_run);
                    println!("Total Serialisation runs: {}", run_durations.all_deser_run_ms.len());
                    output_speed_test_ranges("Deserialisation", shortest_deserialisation_run, longest_deserialisation_run, &average_deserialisation_run);
                    output_speed_test_gaps("Additive Serialisation", shortest_additive_serialisation_run_gaps, longest_additive_serialisation_run_gaps, &average_additive_serialisation_run_gaps);
                    if let Some(duration_ms) = final_cumulative_serialisation_stopwatch.duration_ms() {
                        println!("Last Deserialisation of {} items completed in: {}ms", successfully_decoded_items.len(), duration_ms);
                    }
                    match (total_run_stopwatch.duration_ms(), total_run_stopwatch.duration_ns()) {
                        (Some(ms), None) => println!("{} duration: {}ms", ms, total_run_stopwatch.timer),
                        (None, Some(ns)) => println!("{} duration: {}ns", total_run_stopwatch.timer, ns),
                        (Some(ms), Some(ns)) => println!("{} duration: {}ms ({}ns)", total_run_stopwatch.timer, ms, ns),
                        (_, _) => println!("Unknown {} duration for this run", total_run_stopwatch.timer)
                    }
                    Ok(())
                }
            }
        }
    }
    
    fn output_speed_test_ranges(test_type: &str, shortest: Option<&i64>, longest: Option<&i64>, average: &i64) {
        if let (Some(shortest_run), Some(longest_run)) = (shortest, longest) {
            println!("{} run stats:\nShortest: {}ms\nLongest: {}ms\nAverage: {}ms",
                     test_type, shortest_run, longest_run, average);
        }
    }
    
    fn output_speed_test_gaps(test_type: &str, shortest: Option<&i64>, longest: Option<&i64>, average: &i64) {
        if let (Some(shortest_run), Some(longest_run)) = (shortest, longest) {
            println!("{} run stats:\nShortest gap: {}ms\nLongest gap: {}ms\nAverage: {}ms",
                     test_type, shortest_run, longest_run, average);
        }
    }

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
    fn test_determining_message() -> Result<(), Box<dyn Error>> {
        let load_all_messages: Result<Vec<String>, Box<dyn Error>> =
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
                    test_enum_serialisation(&message);
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
    
    #[test]
    #[ignore]
    fn test_serialisation_deserialisation_speed() -> Result<(), Box<dyn Error>> {
        100.speed_test()
    }
}
