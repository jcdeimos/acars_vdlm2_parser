use std::error::Error;
use std::ffi::OsStr;
use std::fs::File;
use std::{fmt, io};
use std::fmt::Formatter;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::time::Duration;
use humantime::format_duration;
use chrono::{DateTime, Utc};
use glob::{glob, GlobResult, Paths, PatternError};
use prettytable::{row, Table, cell};
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde_json::Value;
use acars_vdlm2_parser::{AcarsVdlm2Message, DecodeMessage, MessageResult};
use acars_vdlm2_parser::acars::{AcarsMessage, NewAcarsMessage};
use acars_vdlm2_parser::vdlm2::{NewVdlm2Message, Vdlm2Message};

/// Enum for indicating test data type.
pub(crate) enum MessageType {
    Acars,
    Vdlm2,
    All,
}

pub(crate) enum SpeedTestType {
    IteratingRoundsLibrary,
    LargeQueueLibrary,
    IteratingRoundsValue,
    LargeQueueValue
}

impl fmt::Display for SpeedTestType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            SpeedTestType::IteratingRoundsLibrary => write!(f, "Iterating Rounds Library"),
            SpeedTestType::LargeQueueLibrary => write!(f, "Large Queue Library"),
            SpeedTestType::IteratingRoundsValue => write!(f, "Iterating Rounds Value"),
            SpeedTestType::LargeQueueValue => write!(f, "Large Queue Value")
        }
    }
}

pub(crate) enum StopwatchType {
    AllDeser,
    AllSer,
    LargeQueueSer,
    LargeQueueDeser,
    TotalRun
}

pub(crate) enum StatType {
    AllDeser,
    AllSer
}

/// Struct for storing test information for the tests that just display error information.
pub(crate) struct TestFile {
    pub(crate) name: String,
    pub(crate) contents: Vec<String>,
}

pub(crate) struct Stopwatch {
    pub(crate) start_time: Option<DateTime<Utc>>,
    pub(crate) stop_time: Option<DateTime<Utc>>,
    pub(crate) duration_ms: i64,
    pub(crate) duration_ns: i64,
    pub(crate) stopwatch_type: StopwatchType
}

impl Stopwatch {
    pub(crate) fn start(stopwatch_type: StopwatchType) -> Self {
        Self {
            start_time: Some(Utc::now()),
            stop_time: None,
            duration_ms: i64::default(),
            duration_ns: i64::default(),
            stopwatch_type
        }
    }
    pub(crate) fn stop(&mut self) {
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

// #[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub(crate) struct RunDurations {
    pub(crate) run_processed_items: usize,
    pub(crate) total_test_runs: i64,
    pub(crate) test_runs: Vec<TestRun>,
    pub(crate) large_queue_ser_ms: i64,
    pub(crate) large_queue_ser_ns: i64,
    pub(crate) large_queue_deser_ms: i64,
    pub(crate) large_queue_deser_ns: i64,
    pub(crate) total_run_ms: i64,
    pub(crate) total_run_ns: i64
    
}

#[derive(Debug, Clone, Default)]
pub(crate) struct TestRun {
    pub(crate) run_number: i64,
    pub(crate) run_items: usize,
    pub(crate) deser_ms: i64,
    pub(crate) deser_ns: i64,
    pub(crate) ser_ms: i64,
    pub(crate) ser_ns: i64
}

impl TestRun {
    pub(crate) fn new(run_number: &i64) -> Self {
        Self {
            run_number: *run_number,
            run_items: usize::default(),
            deser_ms: i64::default(),
            deser_ns: i64::default(),
            ser_ms: i64::default(),
            ser_ns: i64::default()
        }
    }
    
    pub(crate) fn update_run_durations(&mut self, stopwatch: &Stopwatch) {
        match stopwatch.stopwatch_type {
            StopwatchType::AllDeser => {
                self.deser_ms = stopwatch.duration_ms;
                self.deser_ns = stopwatch.duration_ns;
            }
            StopwatchType::AllSer => {
                self.ser_ms = stopwatch.duration_ms;
                self.ser_ns = stopwatch.duration_ms;
            }
            StopwatchType::LargeQueueSer => {}
            StopwatchType::LargeQueueDeser => {}
            StopwatchType::TotalRun => {}
        }
    }
}



// #[allow(dead_code)]
impl RunDurations {
    pub(crate) fn new() -> Self {
        Self {
            run_processed_items: usize::default(),
            total_test_runs: i64::default(),
            test_runs: Vec::new(),
            large_queue_ser_ms: i64::default(),
            large_queue_ser_ns: i64::default(),
            large_queue_deser_ms: i64::default(),
            large_queue_deser_ns: i64::default(),
            total_run_ms: i64::default(),
            total_run_ns: i64::default()
        }
    }
    pub(crate) fn update_run_durations(&mut self, stopwatch: &Stopwatch) {
        match stopwatch.stopwatch_type {
            StopwatchType::AllDeser => {}
            StopwatchType::AllSer => {}
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
}

pub(crate) struct SpeedTestComparisons {
    pub(crate) test_one_type: SpeedTestType,
    pub(crate) test_one_results: RunDurations,
    pub(crate) test_two_type: SpeedTestType,
    pub(crate) test_two_results: RunDurations
}

impl SpeedTestComparisons {
    pub(crate) fn compare_rounds(self) {
        let mut comparison_table: Table = Table::new();
        let mut test_one: RunDurations = self.test_one_results;
        let mut test_two: RunDurations = self.test_two_results;
        let test_one_ser: SpeedTestDurations = test_one.test_runs.get_run_stats(StatType::AllSer);
        let test_one_deser: SpeedTestDurations = test_one.test_runs.get_run_stats(StatType::AllDeser);
        let test_two_ser: SpeedTestDurations = test_two.test_runs.get_run_stats(StatType::AllSer);
        let test_two_deser: SpeedTestDurations = test_two.test_runs.get_run_stats(StatType::AllDeser);
        let test_one_duration = Duration::from_millis(*&test_one.total_run_ms as u64);
        let test_two_duration = Duration::from_millis(*&test_two.total_run_ms as u64);
        comparison_table.add_row(row!["Result", self.test_one_type, self.test_two_type]);
        comparison_table.add_row(row!["Rounds", test_one.total_test_runs, test_two.total_test_runs]);
        comparison_table.add_row(row!["Total processed items", test_one.run_processed_items, test_two.run_processed_items]);
        if let (
            Some(test_one_ms),
            Some(test_one_ns),
            Some(test_two_ms),
            Some(test_two_ns)) = (
            test_one_ser.shortest_duration_ms,
            test_one_ser.shortest_duration_ns,
            test_two_ser.shortest_duration_ms,
            test_two_ser.shortest_duration_ns) {
            comparison_table.add_row(row![
                    "Shortest Serialisation",
                    format!("{}ms ({}ns), Run {}", test_one_ms, test_one_ns, test_one_ser.shortest_run_number),
                    format!("{}ms ({}ns), Run {}", test_two_ms, test_two_ns, test_two_ser.shortest_run_number)
                ]);
        }
        if let (
            Some(test_one_ms),
            Some(test_one_ns),
            Some(test_two_ms),
            Some(test_two_ns)) = (
            test_one_ser.longest_duration_ms,
            test_one_ser.longest_duration_ns,
            test_two_ser.longest_duration_ms,
            test_two_ser.longest_duration_ns) {
            comparison_table.add_row(row![
                "Longest Serialisation",
                format!("{}ms ({}ns), Run {}", test_one_ms, test_one_ns, test_one_ser.longest_run_number),
                format!("{}ms ({}ns), Run {}", test_two_ms, test_two_ns, test_two_ser.longest_run_number)
                ]);
        }
        comparison_table.add_row(row![
                "Average Serialisation",
                format!("{}ms ({}ns)", test_one_ser.average_duration_ms, test_one_ser.average_duration_ns),
                format!("{}ms ({}ns)",test_two_ser.average_duration_ms, test_two_ser.average_duration_ns)
                ]);
        if let (
            Some(test_one_ms),
            Some(test_one_ns),
            Some(test_two_ms),
            Some(test_two_ns)) = (
            test_one_deser.shortest_duration_ms,
            test_one_deser.shortest_duration_ns,
            test_two_deser.shortest_duration_ms,
            test_two_deser.shortest_duration_ns) {
            comparison_table.add_row(row![
                "Shortest Deserialisation",
                format!("{}ms ({}ns), Run {}", test_one_ms, test_one_ns, test_one_deser.shortest_run_number),
                format!("{}ms ({}ns), Run {}", test_two_ms, test_two_ns, test_two_deser.shortest_run_number)
                ]);
        }
        if let (
            Some(test_one_ms),
            Some(test_one_ns),
            Some(test_two_ms),
            Some(test_two_ns)) = (
            test_one_deser.longest_duration_ms,
            test_one_deser.longest_duration_ns,
            test_two_deser.longest_duration_ms,
            test_two_deser.longest_duration_ns) {
            comparison_table.add_row(row![
                "Longest Deserialisation",
                format!("{}ms ({}ns), Run {}", test_one_ms, test_one_ns, test_one_deser.longest_run_number),
                format!("{}ms ({}ns), Run {}", test_two_ms, test_two_ns, test_two_deser.longest_run_number)
                ]);
        }
        comparison_table.add_row(
            row![
                "Average Deserialisation",
                format!("{}ms  ({}ns)", test_one_deser.average_duration_ms, test_one_deser.average_duration_ns),
                format!("{}ms ({}ns)", test_two_deser.average_duration_ms, test_two_deser.average_duration_ns)
                ]);
        comparison_table.add_row(row![
            "Total Runtime",
            format!("{} ({}ms) ({}ns)", format_duration(test_one_duration).to_string(), test_one.total_run_ms, test_one.total_run_ns),
            format!("{} ({}ms) ({}ns)", format_duration(test_two_duration).to_string(), test_two.total_run_ms, test_two.total_run_ns)
        ]);
        comparison_table.printstd();
    }
    pub(crate) fn compare_large_queue(self) {
        let mut comparison_table: Table = Table::new();
        let test_one: RunDurations = self.test_one_results;
        let test_two: RunDurations = self.test_two_results;
        let test_one_duration = Duration::from_millis(*&test_one.total_run_ms as u64);
        let test_two_duration = Duration::from_millis(*&test_two.total_run_ms as u64);
        comparison_table.add_row(row!["Result", self.test_one_type, self.test_two_type]);
        comparison_table.add_row(row!["Processed items", test_one.run_processed_items, test_two.run_processed_items]);
        comparison_table.add_row(row![
            "Serialisation",
            format!("{}ms ({}ns)", test_one.large_queue_ser_ms, test_one.large_queue_ser_ns),
            format!("{}ms ({}ns)", test_two.large_queue_ser_ms, test_two.large_queue_ser_ns)
        ]);
        comparison_table.add_row(row![
            "Deserialisation",
            format!("{}ms ({}ns)",test_one.large_queue_deser_ms, test_one.large_queue_deser_ns),
            format!("{}ms ({}ns)",test_two.large_queue_deser_ms, test_two.large_queue_deser_ns)
        ]);
        comparison_table.add_row(row![
            "Total Runtime",
            format!("{} ({}ms) ({}ns)", format_duration(test_one_duration).to_string(), test_one.total_run_ms, test_one.total_run_ns),
            format!("{} ({}ms) ({}ns)", format_duration(test_two_duration).to_string(), test_two.total_run_ms, test_two.total_run_ns)
        ]);
        comparison_table.printstd();
    }
}

pub(crate) struct SpeedTestDurations {
    pub(crate) shortest_duration_ms: Option<i64>,
    pub(crate) shortest_duration_ns: Option<i64>,
    pub(crate) shortest_run_number: i64,
    pub(crate) longest_duration_ms: Option<i64>,
    pub(crate) longest_duration_ns: Option<i64>,
    pub(crate) longest_run_number: i64,
    pub(crate) average_duration_ms: i64,
    pub(crate) average_duration_ns: i64
}

pub(crate) trait BuildSpeedTestDurations {
    fn get_run_stats(&mut self, stat_type: StatType) -> SpeedTestDurations;
}

impl BuildSpeedTestDurations for Vec<TestRun> {
    fn get_run_stats(&mut self, stat_type: StatType) -> SpeedTestDurations {
        let middle: usize = self.len() / 2;
        let duration = match stat_type {
            StatType::AllDeser => {
                self.sort_by(|a, b| a.deser_ns.cmp(&b.deser_ns));
                let first = self.first();
                let last = self.last();
                match (first, last) {
                    (Some(first), Some(last)) => {
                        SpeedTestDurations {
                            shortest_duration_ms: Some(first.deser_ms),
                            shortest_duration_ns: Some(first.deser_ns),
                            shortest_run_number: first.run_number,
                            longest_duration_ms: Some(last.deser_ms),
                            longest_duration_ns: Some(last.deser_ns),
                            longest_run_number: last.run_number,
                            average_duration_ms: self[middle].deser_ms,
                            average_duration_ns: self[middle].deser_ns
                        }
                    }
                    (_,_) => SpeedTestDurations {
                        shortest_duration_ms: None,
                        shortest_duration_ns: None,
                        shortest_run_number: 0,
                        longest_duration_ms: None,
                        longest_duration_ns: None,
                        longest_run_number: 0,
                        average_duration_ms: 0,
                        average_duration_ns: 0
                    }
                }
            }
            StatType::AllSer => {
                self.sort_by(|a, b| a.ser_ns.cmp(&b.ser_ns));
                let first = self.first();
                let last = self.last();
                match (first, last) {
                    (Some(first), Some(last)) => {
                        SpeedTestDurations {
                            shortest_duration_ms: Some(first.ser_ms),
                            shortest_duration_ns: Some(first.ser_ns),
                            shortest_run_number: first.run_number,
                            longest_duration_ms: Some(last.ser_ms),
                            longest_duration_ns: Some(last.ser_ns),
                            longest_run_number: last.run_number,
                            average_duration_ms: self[middle].ser_ms,
                            average_duration_ns: self[middle].ser_ns
                        }
                    }
                    (_,_) => SpeedTestDurations {
                        shortest_duration_ms: None,
                        shortest_duration_ns: None,
                        shortest_run_number: 0,
                        longest_duration_ms: None,
                        longest_duration_ns: None,
                        longest_run_number: 0,
                        average_duration_ms: 0,
                        average_duration_ns: 0
                    }
                }
            }
        };
        duration
    }
}

/// Trait for appending data.
///
/// Using a trait to allow for implementation against `Vec<TestFile>`.
pub(crate) trait AppendData {
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
pub(crate) fn read_test_file(filepath: impl AsRef<Path>) -> io::Result<Vec<String>> {
    BufReader::new(File::open(filepath)?).lines().collect()
}

/// Assistane function to combine contents of test files into a `Vec<String>`.
///
/// This is used for combining the contents of multiple files into a single `Vec<String>` for testing.
pub(crate) fn combine_found_files(
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
pub(crate) fn load_found_files(
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
pub(crate) fn append_lines(
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
pub(crate) fn combine_files_of_message_type(
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
pub(crate) fn load_files_of_message_type(
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
pub(crate) fn process_file_as_vdlm2(contents: &[String]) {
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
pub(crate) fn process_file_as_acars(contents: &[String]) {
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
pub(crate) fn compare_errors(
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

pub(crate) fn test_enum_serialisation(message: &AcarsVdlm2Message) {
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

pub(crate) fn test_value_serialisation(message: &Value) {
    let encoded_string: MessageResult<String> = serde_json::to_string(&message);
    assert_eq!(
        encoded_string.as_ref().err().is_none(),
        true,
        "Parsing data {:?} to String failed: {:?}",
        message,
        encoded_string.as_ref().err()
    );
    let encoded_bytes: MessageResult<Vec<u8>> = serde_json::to_vec(&message);
    assert_eq!(
        encoded_bytes.as_ref().err().is_none(),
        true,
        "Parsing data {:?} to bytes failed: {:?}",
        message,
        encoded_bytes.as_ref().err()
    );
}

pub(crate) trait ContentDuplicator {
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