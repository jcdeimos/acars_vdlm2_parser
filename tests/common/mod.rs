#![allow(dead_code)]
use acars_vdlm2_parser::error_handling::deserialization_error::DeserializationError;
use acars_vdlm2_parser::helpers::encode_adsb_raw_input::format_adsb_raw_frames_from_bytes;
use acars_vdlm2_parser::message_types::acars::NewAcarsMessage;
use acars_vdlm2_parser::message_types::adsb_json::NewAdsbJsonMessage;
use acars_vdlm2_parser::message_types::adsb_raw::{AdsbRawMessage, NewAdsbRawMessage};
use acars_vdlm2_parser::message_types::vdlm2::NewVdlm2Message;
use acars_vdlm2_parser::DecodedMessage;
use byte_unit::Byte;
use chrono::{DateTime, SecondsFormat, Utc};
use glob::{glob, GlobResult, Paths, PatternError};
use hex;
use humantime::format_duration;
use prettytable::format::Alignment;
use prettytable::{row, Cell, Row, Table};
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde_json::Value;
use std::error::Error;
use std::fmt::Formatter;
use std::fs::File;
use std::io::Read;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::time::Duration;
use std::{fmt, io};
use thousands::Separable;
/// Enum for indicating test data type.
pub enum MessageType {
    Acars,
    Vdlm2,
    AdsbJson,
    AdsbRaw,
    All,
}

pub enum SerialisationTarget {
    String,
    Bytes,
    Both,
}

pub enum SpeedTestType {
    LargeQueueLibrary,
    LargeQueueValue,
}

impl fmt::Display for SpeedTestType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            SpeedTestType::LargeQueueLibrary => write!(f, "Large Queue Library"),
            SpeedTestType::LargeQueueValue => write!(f, "Large Queue Value"),
        }
    }
}

pub enum StopwatchType {
    LargeQueueSer,
    LargeQueueDeser,
    TotalRun,
}

pub enum StatType {
    AllDeser,
    AllSer,
}

#[derive(Debug, Clone)]
pub enum TestFileType {
    String(String),
    U8(Vec<u8>),
}

impl From<String> for TestFileType {
    fn from(v: String) -> Self {
        Self::String(v)
    }
}

impl From<Vec<u8>> for TestFileType {
    fn from(v: Vec<u8>) -> Self {
        Self::U8(v)
    }
}

// FIXME: Does this make sense?
impl IntoIterator for TestFileType {
    type Item = u8;
    type IntoIter = std::vec::IntoIter<u8>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            TestFileType::String(v) => v.into_bytes().into_iter(),
            TestFileType::U8(v) => v.into_iter(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum FileTypes {
    String(Vec<String>),
    U8(Vec<u8>),
}

impl From<String> for FileTypes {
    fn from(v: String) -> Self {
        Self::String(vec![v])
    }
}

impl From<Vec<String>> for FileTypes {
    fn from(v: Vec<String>) -> Self {
        Self::String(v)
    }
}

impl IntoIterator for FileTypes {
    type Item = String;
    type IntoIter = std::vec::IntoIter<String>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            FileTypes::String(v) => v.into_iter(),
            FileTypes::U8(v) => {
                let mut v_string: Vec<String> = Vec::new();
                for i in v {
                    v_string.push(i.to_string());
                }
                v_string.into_iter()
            }
        }
    }
}

impl From<Vec<u8>> for FileTypes {
    fn from(v: Vec<u8>) -> Self {
        Self::U8(v)
    }
}
/// Struct for storing test information for the tests that just display error information.
pub struct TestFile {
    pub name: String,
    pub contents: FileTypes,
}

/// Struct for storing the start, end time and durations for doing elapsed time measurement.
pub struct Stopwatch {
    pub start_time: Option<DateTime<Utc>>,
    pub stop_time: Option<DateTime<Utc>>,
    pub duration_ms: i64,
    pub duration_ns: i64,
    pub stopwatch_type: StopwatchType,
}

impl Stopwatch {
    /// Set the start DateTime for when the call is made and store it.
    ///
    /// Returns an instance of itself
    pub fn start(stopwatch_type: StopwatchType) -> Self {
        Self {
            start_time: Some(Utc::now()),
            stop_time: None,
            duration_ms: i64::default(),
            duration_ns: i64::default(),
            stopwatch_type,
        }
    }
    /// Sets the stop DateTime for when the call is made and stores it.
    ///
    /// Will also calculate the duration in milliseconds and nanoseconds and store them in two i64's
    pub fn stop(&mut self) {
        self.stop_time = Some(Utc::now());
        if let (Some(stop), Some(start)) = (self.stop_time, self.start_time) {
            let duration: chrono::Duration = stop - start;
            self.duration_ms = duration.num_milliseconds();
            if let Some(duration_ns) = duration.num_nanoseconds() {
                self.duration_ns = duration_ns;
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RunDurations {
    pub run_processed_items: usize,
    pub queue_memory_size: Byte,
    pub large_queue_ser_ms: i64,
    pub large_queue_ser_ns: i64,
    pub large_queue_deser_ms: i64,
    pub large_queue_deser_ns: i64,
    pub total_run_ms: i64,
    pub total_run_ns: i64,
}

impl RunDurations {
    pub fn new() -> Self {
        Self {
            run_processed_items: usize::default(),
            queue_memory_size: Default::default(),
            large_queue_ser_ms: i64::default(),
            large_queue_ser_ns: i64::default(),
            large_queue_deser_ms: i64::default(),
            large_queue_deser_ns: i64::default(),
            total_run_ms: i64::default(),
            total_run_ns: i64::default(),
        }
    }
    pub fn update_run_durations(&mut self, stopwatch: &Stopwatch) {
        match stopwatch.stopwatch_type {
            StopwatchType::LargeQueueSer => {
                self.large_queue_ser_ms = stopwatch.duration_ms;
                self.large_queue_ser_ns = stopwatch.duration_ns;
            }
            StopwatchType::LargeQueueDeser => {
                self.large_queue_deser_ms = stopwatch.duration_ms;
                self.large_queue_deser_ns = stopwatch.duration_ns;
            }
            StopwatchType::TotalRun => {
                self.total_run_ms = stopwatch.duration_ms;
                self.total_run_ns = stopwatch.duration_ns;
            }
        }
    }
    pub fn display_run_duration(self, speed_test_type: SpeedTestType) {
        let mut result_table: Table = Table::new();
        let test_one_duration: Duration = Duration::from_millis(self.total_run_ms as u64);
        result_table.add_row(row![
            "Run",
            Utc::now().to_rfc3339_opts(SecondsFormat::Secs, false)
        ]);
        result_table.add_row(row!["Result", speed_test_type]);
        result_table.add_row(row![
            "Processed items",
            format!(
                "{} (Memory size {})",
                self.run_processed_items.separate_with_commas(),
                self.queue_memory_size.get_appropriate_unit(false)
            )
        ]);
        result_table.add_row(row![
            "Serialisation",
            format!(
                "{} ({}ms) ({}ns)",
                format_duration(Duration::from_millis(self.large_queue_ser_ms as u64)),
                self.large_queue_ser_ms,
                self.large_queue_ser_ns
            )
        ]);
        result_table.add_row(row![
            "Deserialisation",
            format!(
                "{} ({}ms) ({}ns)",
                format_duration(Duration::from_millis(self.large_queue_deser_ms as u64)),
                self.large_queue_deser_ms,
                self.large_queue_deser_ns
            )
        ]);
        result_table.add_row(row![
            "Total Runtime",
            format!(
                "{} ({}ms) ({}ns)",
                format_duration(test_one_duration),
                self.total_run_ms,
                self.total_run_ns
            )
        ]);
        result_table.printstd();
    }
}

pub struct SpeedTestComparisons {
    pub test_one_type: SpeedTestType,
    pub test_one_results: RunDurations,
    pub test_two_type: SpeedTestType,
    pub test_two_results: RunDurations,
}

impl SpeedTestComparisons {
    pub fn compare_large_queue(self) {
        let mut comparison_table: Table = Table::new();
        let test_one_duration: Duration =
            Duration::from_millis(self.test_one_results.total_run_ms as u64);
        let test_two_duration: Duration =
            Duration::from_millis(self.test_two_results.total_run_ms as u64);
        let mut date_cell: Cell =
            Cell::new(&Utc::now().to_rfc3339_opts(SecondsFormat::Secs, false)).with_hspan(2);
        date_cell.align(Alignment::CENTER);
        let cells: Vec<Cell> = vec![Cell::new("Run"), date_cell];
        let header_row: Row = Row::new(cells);
        comparison_table.add_row(header_row);
        comparison_table.add_row(row!["Result", &self.test_one_type, &self.test_two_type]);
        comparison_table.add_row(row![
            "Processed items",
            format!(
                "{} (Memory size {})",
                self.test_one_results
                    .run_processed_items
                    .separate_with_commas(),
                self.test_one_results
                    .queue_memory_size
                    .get_appropriate_unit(false)
            ),
            format!(
                "{} (Memory size {})",
                self.test_two_results
                    .run_processed_items
                    .separate_with_commas(),
                self.test_two_results
                    .queue_memory_size
                    .get_appropriate_unit(false)
            ),
        ]);
        comparison_table.add_row(row![
            "Serialisation",
            format!(
                "{} ({}ms) ({}ns)",
                format_duration(Duration::from_millis(
                    self.test_one_results.large_queue_ser_ms as u64
                )),
                self.test_one_results.large_queue_ser_ms,
                self.test_one_results.large_queue_ser_ns
            ),
            format!(
                "{} ({}ms) ({}ns)",
                format_duration(Duration::from_millis(
                    self.test_two_results.large_queue_ser_ms as u64
                )),
                self.test_two_results.large_queue_ser_ms,
                self.test_two_results.large_queue_ser_ns
            )
        ]);
        comparison_table.add_row(row![
            "Deserialisation",
            format!(
                "{} ({}ms) ({}ns)",
                format_duration(Duration::from_millis(
                    self.test_one_results.large_queue_deser_ms as u64
                )),
                self.test_one_results.large_queue_deser_ms,
                self.test_one_results.large_queue_deser_ns
            ),
            format!(
                "{} ({}ms) ({}ns)",
                format_duration(Duration::from_millis(
                    self.test_two_results.large_queue_deser_ms as u64
                )),
                self.test_two_results.large_queue_deser_ms,
                self.test_two_results.large_queue_deser_ns
            )
        ]);
        comparison_table.add_row(row![
            "Total Runtime",
            format!(
                "{} ({}ms) ({}ns)",
                format_duration(test_one_duration),
                self.test_one_results.total_run_ms,
                self.test_one_results.total_run_ns
            ),
            format!(
                "{} ({}ms) ({}ns)",
                format_duration(test_two_duration),
                self.test_two_results.total_run_ms,
                self.test_two_results.total_run_ns
            )
        ]);
        comparison_table.printstd();
    }
}

/// Trait for appending data.
///
/// Using a trait to allow for implementation against `Vec<TestFile>`.
pub trait AppendData {
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
            Ok(target_file) => match File::open(target_file.as_path()) {
                Err(file_error) => Err(file_error.into()),
                Ok(mut file_read) => {
                    let extension = match target_file.as_path().extension() {
                        None => "",
                        Some(extension) => extension.to_str().unwrap_or_default(),
                    };

                    if extension != "bin" {
                        match BufReader::new(file_read)
                            .lines()
                            .collect::<Result<Vec<String>, _>>()
                        {
                            Err(read_error) => Err(read_error.into()),
                            Ok(contents) => match target_file.file_name() {
                                None => Err("Could not get file name".into()),
                                Some(file_name) => {
                                    println!("{}", contents.len());
                                    let test_file: TestFile = TestFile {
                                        name: format!("{:?}", file_name),
                                        contents: contents.into(),
                                    };
                                    self.push(test_file);
                                    Ok(())
                                }
                            },
                        }
                    } else {
                        let mut contents: Vec<u8> = vec![];
                        match file_read.read_to_end(&mut contents) {
                            Err(read_error) => Err(read_error.into()),
                            Ok(_) => {
                                let test_file: TestFile = TestFile {
                                    name: format!("{:?}", target_file.file_name().unwrap()),
                                    /// This reads the entire file in to a `Vec<u8>`. Each individual message is *NOT*
                                    /// seperated out. It is the responsibility of the caller to ensure that each message is separated
                                    contents: contents.into(),
                                };
                                self.push(test_file);
                                Ok(())
                            }
                        }
                    }
                }
            },
        }
    }
}

/// Assistance function for tests to read a file, and break it up per line to a `Vec<String>`.
///
/// This allows for tests to iterate through and test each line individually.
pub fn read_test_file(filepath: impl AsRef<Path>) -> io::Result<Vec<String>> {
    BufReader::new(File::open(filepath)?).lines().collect()
}

/// Assistance function to combine contents of test files into a `Vec<String>`.
///
/// This is used for combining the contents of multiple files into a single `Vec<String>` for testing.
pub fn combine_found_files(
    find_files: Result<Paths, PatternError>,
) -> Result<Vec<TestFileType>, Box<dyn Error>> {
    match find_files {
        Err(pattern_error) => Err(pattern_error.into()),
        Ok(file_paths) => {
            let mut loaded_contents: Vec<TestFileType> = Vec::new();
            for file in file_paths {
                let append_data: Result<(), Box<dyn Error>> =
                    append_lines(file, &mut loaded_contents);
                append_data?
            }
            Ok(loaded_contents.to_vec())
        }
    }
}

/// Assistance function for building a `Vec<TestFile>` for use with the tests that show parsing output.
pub fn load_found_files(
    find_files: Result<Paths, PatternError>,
) -> Result<Vec<TestFile>, Box<dyn Error>> {
    match find_files {
        Err(pattern_error) => Err(pattern_error.into()),
        Ok(file_paths) => {
            let mut test_files: Vec<TestFile> = Vec::new();
            for file in file_paths {
                let load_test_file: Result<(), Box<dyn Error>> = test_files.append_data(file);
                load_test_file?
            }
            Ok(test_files)
        }
    }
}

/// Assistance function for appending file contents.
pub fn append_lines(file: GlobResult, data: &mut Vec<TestFileType>) -> Result<(), Box<dyn Error>> {
    match file {
        Err(file_error) => Err(file_error.into()),
        Ok(file_path) => {
            let extension = match file_path.as_path().extension() {
                None => "",
                Some(extension) => extension.to_str().unwrap_or_default(),
            };

            if extension != "bin" {
                match read_test_file(file_path.as_path()) {
                    Err(read_error) => Err(read_error.into()),
                    Ok(contents) => {
                        for line in contents {
                            data.push(line.into());
                        }
                        Ok(())
                    }
                }
            } else {
                let mut contents: Vec<u8> = vec![];
                let mut reader = File::open(file_path.as_path())?;
                match reader.read_to_end(&mut contents) {
                    Err(read_error) => Err(read_error.into()),
                    Ok(_) => {
                        // The consumer of the data is responsible for seperating the messages in a binary file.
                        // FIXME: we should probably really split these up into individual messages....
                        // But we need to know the actual format of the message to do that.
                        data.push(contents.into());

                        Ok(())
                    }
                }
            }
        }
    }
}

/// Assistance function that combines contents of message type test files.
pub fn combine_files_of_message_type(
    message_type: MessageType,
) -> Result<Vec<TestFileType>, Box<dyn Error>> {
    match message_type {
        MessageType::Acars => combine_found_files(glob("test_files/acars*")),
        MessageType::Vdlm2 => combine_found_files(glob("test_files/vdlm2*")),
        MessageType::AdsbJson => combine_found_files(glob("test_files/adsb_*.json")),
        MessageType::AdsbRaw => combine_found_files(glob("test_files/adsb_raw*.bin")),
        MessageType::All => combine_found_files(glob("test_files/*")),
    }
}

/// Assistance function that loads contents of individual message type test files and returns them separately instead of combined.
pub fn load_files_of_message_type(
    message_type: MessageType,
) -> Result<Vec<TestFile>, Box<dyn Error>> {
    match message_type {
        MessageType::Acars => load_found_files(glob("test_files/acars*")),
        MessageType::Vdlm2 => load_found_files(glob("test_files/vdlm2*")),
        MessageType::AdsbJson => load_found_files(glob("test_files/adsb_*.json")),
        MessageType::AdsbRaw => load_found_files(glob("test_files/adsb_raw*.bin")),
        MessageType::All => load_found_files(glob("test_files/*")),
    }
}

/// Assistance function for processing the contents of a `&[String]` slice as vdlm2 messages.
pub fn process_file_as_vdlm2(contents: &[String]) {
    let contents: Vec<String> = contents.to_vec();
    let mut errors: Vec<String> = Vec::new();
    for (entry, line) in contents.iter().enumerate() {
        if let Err(parse_error) = line.to_vdlm2() {
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
pub fn process_file_as_acars(contents: &[String]) {
    let contents: Vec<String> = contents.to_vec();
    let mut errors: Vec<String> = Vec::new();
    for (entry, line) in contents.iter().enumerate() {
        if let Err(parse_error) = line.to_acars() {
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

/// Assistance function for processing the contents of a `&[String]` slice as adsb_json messages.
pub fn process_file_as_adsb_json(contents: &[String]) {
    let contents: Vec<String> = contents.to_vec();
    let mut errors: Vec<String> = Vec::new();
    for (entry, line) in contents.iter().enumerate() {
        if let Err(parse_error) = line.to_adsb() {
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

pub fn process_file_as_adsb_raw(contents: &[u8]) {
    let frames: Vec<Vec<u8>> = format_adsb_raw_frames_from_bytes(&contents.to_vec());
    let mut errors: Vec<String> = Vec::new();
    let mut entry = 0;

    for frame in frames.iter() {
        if let Err(parse_error) = frame.to_adsb_raw() {
            let error_text: String = format!(
                "Entry {} parse error: {}\nData: {}",
                entry + 1,
                parse_error,
                hex::encode(frame)
            );
            errors.push(error_text);
        }
        entry += 1;
    }

    match errors.is_empty() {
        true => println!(
            "No errors found in provided contents. Total messages: {}",
            entry
        ),
        false => {
            println!(
                "Errors found as follows, total messages: {}, total successful decodes: {}, total errors {}",
                entry,
                entry - errors.len(),
                errors.len()
            );
            for error in errors {
                println!("{}", error);
            }
        }
    }
}

/// Assistance function to compare error message strings between Library result and serde `Value` result.
pub fn compare_deku_errors(
    error_1: Option<DeserializationError>,
    error_2: Result<((&[u8], usize), AdsbRawMessage), DeserializationError>,
    line: &str,
) {
    match (error_1, error_2) {
        (None, Ok(_)) => {}
        (Some(library_error), Ok(value_data)) => {
            assert!(false, "Library {}, Value {:?}", &library_error, &value_data)
        }
        (Some(library_error), Err(value_error)) => assert_eq!(
            library_error.to_string(),
            value_error.to_string(),
            "Errors processing {} do not match between library {} and serde Value {}",
            line,
            library_error,
            value_error.to_string()
        ),
        (None, Err(value_error)) => {
            assert!(false, "Library passed, but Value is {:?}", &value_error)
        }
    }
}

/// Assistance function to compare error message strings between Library result and serde `Value` result.
pub fn compare_serde_errors(
    error_1: Option<DeserializationError>,
    error_2: Result<Value, DeserializationError>,
    line: &str,
) {
    match (error_1, error_2) {
        (None, Ok(_)) => {}
        (Some(library_error), Ok(value_data)) => {
            assert!(false, "Library {}, Value {:?}", &library_error, &value_data)
        }
        (Some(library_error), Err(value_error)) => assert_eq!(
            library_error.to_string(),
            value_error.to_string(),
            "Errors processing {} do not match between library {} and serde Value {}",
            line,
            library_error,
            value_error.to_string()
        ),
        (None, Err(value_error)) => {
            assert!(false, "Library passed, but Value is {:?}", &value_error)
        }
    }
}

pub fn test_enum_serialisation(
    message: &DecodedMessage,
    serialisation_target: SerialisationTarget,
) {
    match serialisation_target {
        SerialisationTarget::String => {
            assert!(
                message.to_string().as_ref().err().is_none(),
                "Parsing data {:?} to String failed: {:?}",
                message,
                message.to_string().as_ref().err()
            );
        }
        SerialisationTarget::Bytes => {
            assert!(
                message.to_bytes().as_ref().err().is_none(),
                "Parsing data {:?} to bytes failed: {:?}",
                message,
                message.to_bytes().as_ref().err()
            );
        }
        SerialisationTarget::Both => {
            assert!(
                message.to_string().as_ref().err().is_none(),
                "Parsing data {:?} to String failed: {:?}",
                message,
                message.to_string().as_ref().err()
            );
            assert!(
                message.to_bytes().as_ref().err().is_none(),
                "Parsing data {:?} to bytes failed: {:?}",
                message,
                message.to_bytes().as_ref().err()
            );
        }
    }
}

pub fn test_value_serialisation(
    message: &DecodedMessage,
    serialisation_target: SerialisationTarget,
) {
    match serialisation_target {
        SerialisationTarget::String => {
            assert!(
                serde_json::to_string(&message).as_ref().err().is_none(),
                "Parsing data {:?} to String failed: {:?}",
                message,
                serde_json::to_string(&message).as_ref().err()
            );
        }
        SerialisationTarget::Bytes => {
            assert!(
                serde_json::to_vec(&message).as_ref().err().is_none(),
                "Parsing data {:?} to bytes failed: {:?}",
                message,
                serde_json::to_vec(&message).as_ref().err()
            );
        }
        SerialisationTarget::Both => {
            assert!(
                serde_json::to_string(&message).as_ref().err().is_none(),
                "Parsing data {:?} to String failed: {:?}",
                message,
                serde_json::to_string(&message).as_ref().err()
            );
            assert!(
                serde_json::to_vec(&message).as_ref().err().is_none(),
                "Parsing data {:?} to bytes failed: {:?}",
                message,
                serde_json::to_vec(&message).as_ref().err()
            );
        }
    }
}

pub trait ContentDuplicator {
    fn duplicate_contents(&self, rounds: &i64) -> Self;
}

impl ContentDuplicator for Vec<String> {
    fn duplicate_contents(&self, rounds: &i64) -> Self {
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

impl ContentDuplicator for Vec<TestFileType> {
    fn duplicate_contents(&self, rounds: &i64) -> Self {
        let mut duplicated_contents: Vec<TestFileType> = Vec::new();
        let mut data: Vec<TestFileType> = self.to_vec();
        let mut rng: ThreadRng = thread_rng();
        for _ in 0..*rounds {
            data.shuffle(&mut rng);
            for entry in &data {
                duplicated_contents.push(entry.to_owned());
            }
        }
        duplicated_contents
    }
}
