mod common;

use std::error::Error;
use std::sync::{Arc, Mutex, MutexGuard};
use rand::prelude::SliceRandom;
use rand::rngs::ThreadRng;
use rand::thread_rng;
use acars_vdlm2_parser::{AcarsVdlm2Message, DecodeMessage, MessageResult};
use crate::common::{combine_files_of_message_type, ContentDuplicator, StopwatchType, MessageType, RunDurations, SpeedTestType, Stopwatch, test_enum_serialisation, test_value_serialisation, SpeedTestComparisons};
use rayon::prelude::*;
use serde_json::Value;

#[test]
#[ignore]
fn test_speed_large_queue() {
    large_queue_results(1_000.large_queue_library(), 1_000.large_queue_value());
    large_queue_results(2_500.large_queue_library(), 2_500.large_queue_value());
    large_queue_results(5_000.large_queue_library(), 5_000.large_queue_value());
    large_queue_results(7_500.large_queue_library(), 7_500.large_queue_value());
    large_queue_results(10_000.large_queue_library(), 10_000.large_queue_value());
    large_queue_results(20_000.large_queue_library(), 20_000.large_queue_value());
    large_queue_results(50_000.large_queue_library(), 50_000.large_queue_value());
    large_queue_results(75_000.large_queue_library(), 75_000.large_queue_value());
}

/// Trait for performing speed tests.
pub(crate) trait SpeedTest {
    fn large_queue_library(&self) -> Result<RunDurations, Box<dyn Error>>;
    fn large_queue_value(&self) -> Result<RunDurations, Box<dyn Error>>;
}

/// `SpeedTest` implemented for `i64`
///
/// Run x iterations, invoked as `int.iterating_rounds_library()`
impl SpeedTest for i64 {
    fn large_queue_library(&self) -> Result<RunDurations, Box<dyn Error>> {
        println!("Starting a speed test of large queue processing using the library");
        println!("Base factor is {}", self);
        let load_all_messages: Result<Vec<String>, Box<dyn Error>> =
            combine_files_of_message_type(MessageType::All);
        match load_all_messages {
            Err(load_error) => Err(load_error),
            Ok(all_messages) => {
                let mut run_durations: RunDurations = RunDurations::new();
                println!("Loaded data successfully");
                let mut rng: ThreadRng = thread_rng();
                let mut test_message_queue: Vec<String> = all_messages.duplicate_contents(self);
                println!("Processing {} messages in a queue", test_message_queue.len());
                run_durations.run_processed_items = test_message_queue.len();
                let successfully_decoded_items: Arc<Mutex<Vec<AcarsVdlm2Message>>> = Arc::new(Mutex::new(Vec::new()));
                println!("Running tests");
                let mut total_run_stopwatch: Stopwatch = Stopwatch::start(StopwatchType::TotalRun);
                test_message_queue.shuffle(&mut rng);
                let mut deserialisation_run_stopwatch: Stopwatch = Stopwatch::start(StopwatchType::LargeQueueDeser);
                test_message_queue.par_iter().for_each(|entry| {
                    let parsed_message: MessageResult<AcarsVdlm2Message> = entry.decode_message();
                    match parsed_message {
                        Err(_) => {}
                        Ok(decoded_message) => {
                            successfully_decoded_items.lock().unwrap().push(decoded_message.clone());
                        }
                    }
                });
                deserialisation_run_stopwatch.stop();
                let mut successfully_decoded_items_lock: MutexGuard<Vec<AcarsVdlm2Message>> = successfully_decoded_items.lock().unwrap();
                run_durations.update_run_durations(&deserialisation_run_stopwatch);
                successfully_decoded_items_lock.shuffle(&mut rng);
                let mut serialisation_run_stopwatch: Stopwatch = Stopwatch::start(StopwatchType::LargeQueueSer);
                successfully_decoded_items_lock.par_iter().for_each(|message| {
                    test_enum_serialisation(message);
                });
                serialisation_run_stopwatch.stop();
                total_run_stopwatch.stop();
                run_durations.update_run_durations(&serialisation_run_stopwatch);
                run_durations.update_run_durations(&total_run_stopwatch);
                println!("Speed test completed, storing results");
                
                Ok(run_durations)
            }
        }
    }
    
    fn large_queue_value(&self) -> Result<RunDurations, Box<dyn Error>> {
        println!("Starting a speed test of large queue processing using serde Value");
        println!("Base factor is {}", self);
        let load_all_messages: Result<Vec<String>, Box<dyn Error>> =
            combine_files_of_message_type(MessageType::All);
        match load_all_messages {
            Err(load_error) => Err(load_error),
            Ok(all_messages) => {
                let mut run_durations: RunDurations = RunDurations::new();
                println!("Loaded data successfully");
                let mut rng: ThreadRng = thread_rng();
                let mut test_message_queue: Vec<String> = all_messages.duplicate_contents(self);
                println!("Processing {} messages in a queue", test_message_queue.len());
                run_durations.run_processed_items = test_message_queue.len();
                let successfully_decoded_items: Arc<Mutex<Vec<Value>>> = Arc::new(Mutex::new(Vec::new()));
                println!("Running tests");
                let mut total_run_stopwatch: Stopwatch = Stopwatch::start(StopwatchType::TotalRun);
                test_message_queue.shuffle(&mut rng);
                let mut deserialisation_run_stopwatch: Stopwatch = Stopwatch::start(StopwatchType::LargeQueueDeser);
                test_message_queue.par_iter().for_each(|entry| {
                    let parsed_message: MessageResult<Value> = serde_json::from_str(&entry);
                    match parsed_message {
                        Err(_) => {}
                        Ok(decoded_message) => {
                            successfully_decoded_items.lock().unwrap().push(decoded_message.clone());
                        }
                    }
                });
                deserialisation_run_stopwatch.stop();
                let mut successfully_decoded_items_lock: MutexGuard<Vec<Value>> = successfully_decoded_items.lock().unwrap();
                run_durations.update_run_durations(&deserialisation_run_stopwatch);
                successfully_decoded_items_lock.shuffle(&mut rng);
                let mut serialisation_run_stopwatch: Stopwatch = Stopwatch::start(StopwatchType::LargeQueueSer);
                successfully_decoded_items_lock.par_iter().for_each(|message| {
                    test_value_serialisation(message);
                });
                serialisation_run_stopwatch.stop();
                total_run_stopwatch.stop();
                run_durations.update_run_durations(&serialisation_run_stopwatch);
                run_durations.update_run_durations(&total_run_stopwatch);
                println!("Speed test completed, storing results");
            
                Ok(run_durations)
            }
        }
    }
}

fn large_queue_results(library_result: Result<RunDurations, Box<dyn Error>>, value_result: Result<RunDurations, Box<dyn Error>>) {
    match (library_result, value_result) {
        (Err(library_error), _) => println!("Library test had an error: {}", library_error),
        (_, Err(value_error)) => println!("Value test had an error: {}", value_error),
        (Ok(library), Ok(value)) => {
            let comparison: SpeedTestComparisons = SpeedTestComparisons {
                test_one_type: SpeedTestType::LargeQueueLibrary,
                test_one_results: library,
                test_two_type: SpeedTestType::LargeQueueValue,
                test_two_results: value
            };
            comparison.compare_large_queue();
        }
    }
}