mod common;

use std::error::Error;
use std::sync::{Arc, Mutex, MutexGuard};
use rand::prelude::SliceRandom;
use rand::rngs::ThreadRng;
use rand::thread_rng;
use acars_vdlm2_parser::{AcarsVdlm2Message, DecodeMessage, MessageResult};
use crate::common::{combine_files_of_message_type, ContentDuplicator, StopwatchType, MessageType, RunDurations, SpeedTestType, Stopwatch, test_enum_serialisation, TestRun, test_value_serialisation, SpeedTestComparisons};
use rayon::prelude::*;
use serde_json::Value;

#[test]
#[ignore]
fn test_serialisation_deserialisation_speed() {
    println!();
    round_results(100.iterating_rounds_library(), 100.iterating_rounds_value());
    round_results(500.iterating_rounds_library(), 500.iterating_rounds_value());
    large_queue_results(1_000.large_queue_library(), 1_000.large_queue_value());
    large_queue_results(5_000.large_queue_library(), 5_000.large_queue_value());
    large_queue_results(10_000.large_queue_library(), 10_000.large_queue_value());
    large_queue_results(20_000.large_queue_library(), 20_000.large_queue_value());
}

/// Trait for performing speed tests.
pub(crate) trait SpeedTest {
    fn iterating_rounds_library(&self) -> Result<RunDurations, Box<dyn Error>>;
    fn large_queue_library(&self) -> Result<RunDurations, Box<dyn Error>>;
    fn iterating_rounds_value(&self) -> Result<RunDurations, Box<dyn Error>>;
    fn large_queue_value(&self) -> Result<RunDurations, Box<dyn Error>>;
}

/// `SpeedTest` implemented for `i32`
///
/// Run x iterations, invoked as `int.speed_test()`
impl SpeedTest for i64 {
    fn iterating_rounds_library(&self) -> Result<RunDurations, Box<dyn Error>> {
        println!("Starting a speed test of {} rounds using the library", self);
        let load_all_messages: Result<Vec<String>, Box<dyn Error>> =
            combine_files_of_message_type(MessageType::All);
        match load_all_messages {
            Err(load_error) => Err(load_error),
            Ok(all_messages) => {
                println!("Loaded data successfully");
                let mut run_durations: RunDurations = RunDurations::new();
                let mut rng: ThreadRng = thread_rng();
                run_durations.total_test_runs = *self;
                let mut processed_items: usize = usize::default();
                println!("Running tests");
                let mut total_run_stopwatch: Stopwatch = Stopwatch::start(StopwatchType::TotalRun);
                for run in 0..*self {
                    let run_count: i64 = run + 1;
                    let mut test_run: TestRun = TestRun::new(&run_count);
                    let mut run_messages: Vec<String> = all_messages.to_vec().duplicate_contents(&run_count);
                    test_run.run_items = run_messages.len();
                    processed_items = processed_items + run_messages.len();
                    run_messages.shuffle(&mut rng);
                    let run_deserialisation_successful_items: Arc<Mutex<Vec<AcarsVdlm2Message>>> = Arc::new(Mutex::new(Vec::new()));
                    let mut deserialisation_run_stopwatch: Stopwatch = Stopwatch::start(StopwatchType::AllDeser);
                    run_messages.par_iter().for_each(|entry| {
                        let parsed_message: MessageResult<AcarsVdlm2Message> = entry.decode_message();
                        match parsed_message {
                            Err(_) => {}
                            Ok(decoded_message) => {
                                run_deserialisation_successful_items.lock().unwrap().push(decoded_message.clone());
                            }
                        }
                    });
                    deserialisation_run_stopwatch.stop();
                    let mut deserialisation_run: Vec<AcarsVdlm2Message> = run_deserialisation_successful_items.lock().unwrap().to_vec();
                    test_run.update_run_durations(&deserialisation_run_stopwatch);
                    deserialisation_run.shuffle(&mut rng);
                    let mut serialisation_run_stopwatch: Stopwatch = Stopwatch::start(StopwatchType::AllSer);
                    deserialisation_run.par_iter().for_each(|message| {
                        test_enum_serialisation(message);
                    });
                    serialisation_run_stopwatch.stop();
                    test_run.update_run_durations(&serialisation_run_stopwatch);
                    run_durations.test_runs.push(test_run);
                }
                total_run_stopwatch.stop();
                run_durations.update_run_durations(&total_run_stopwatch);
                run_durations.run_processed_items = processed_items;
                println!("Speed test completed, storing results.");
                Ok(run_durations)
            }
        }
    }
    
    fn large_queue_library(&self) -> Result<RunDurations, Box<dyn Error>> {
        println!("Starting a speed test of large queue processing using the library");
        let load_all_messages: Result<Vec<String>, Box<dyn Error>> =
            combine_files_of_message_type(MessageType::All);
        match load_all_messages {
            Err(load_error) => Err(load_error),
            Ok(all_messages) => {
                let mut run_durations: RunDurations = RunDurations::new();
                println!("Loaded data successfully");
                let mut rng: ThreadRng = thread_rng();
                println!("Increasing queue size by a factor of {}", self);
                let mut test_message_queue: Vec<String> = all_messages.duplicate_contents(self);
                println!("Queue contains {} messages", test_message_queue.len());
                run_durations.run_processed_items = test_message_queue.len();
                let successfully_decoded_items: Arc<Mutex<Vec<AcarsVdlm2Message>>> = Arc::new(Mutex::new(Vec::new()));
                let mut total_run_stopwatch: Stopwatch = Stopwatch::start(StopwatchType::TotalRun);
                println!("Shuffling the queue");
                test_message_queue.shuffle(&mut rng);
                println!("Deserialising the queue");
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
                println!("Deserialisation completed, shuffling the successful results");
                successfully_decoded_items_lock.shuffle(&mut rng);
                println!("Serialising the queue");
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
    
    fn iterating_rounds_value(&self) -> Result<RunDurations, Box<dyn Error>> {
        println!("Starting a speed test of {} rounds using serde Value", self);
        let load_all_messages: Result<Vec<String>, Box<dyn Error>> =
            combine_files_of_message_type(MessageType::All);
        match load_all_messages {
            Err(load_error) => Err(load_error),
            Ok(all_messages) => {
                println!("Loaded data successfully");
                let mut run_durations: RunDurations = RunDurations::new();
                let mut rng: ThreadRng = thread_rng();
                let mut processed_items: usize = usize::default();
                println!("Running tests");
                let mut total_run_stopwatch: Stopwatch = Stopwatch::start(StopwatchType::TotalRun);
                run_durations.total_test_runs = *self;
                for run in 0..*self {
                    let run_count: i64 = run + 1;
                    let mut test_run: TestRun = TestRun::new(&run_count);
                    let mut run_messages: Vec<String> = all_messages.to_vec().duplicate_contents(&run_count);
                    processed_items = processed_items + run_messages.len();
                    test_run.run_items = run_messages.len();
                    run_messages.shuffle(&mut rng);
                    let run_deserialisation_successful_items: Arc<Mutex<Vec<Value>>> = Arc::new(Mutex::new(Vec::new()));
                    let mut deserialisation_run_stopwatch: Stopwatch = Stopwatch::start(StopwatchType::AllDeser);
                    run_messages.par_iter().for_each(|entry| {
                        let parsed_message: MessageResult<Value> = serde_json::from_str(&entry);
                        match parsed_message {
                            Err(_) => {}
                            Ok(decoded_message) => {
                                run_deserialisation_successful_items.lock().unwrap().push(decoded_message.clone());
                            }
                        }
                    });
                    deserialisation_run_stopwatch.stop();
                    let mut deserialisation_run: Vec<Value> = run_deserialisation_successful_items.lock().unwrap().to_vec();
                    test_run.update_run_durations(&deserialisation_run_stopwatch);
                    deserialisation_run.shuffle(&mut rng);
                    let mut serialisation_run_stopwatch: Stopwatch = Stopwatch::start(StopwatchType::AllSer);
                    deserialisation_run.par_iter().for_each(|message| {
                        test_value_serialisation(message);
                    });
                    serialisation_run_stopwatch.stop();
                    test_run.update_run_durations(&serialisation_run_stopwatch);
                    run_durations.test_runs.push(test_run);
                };
                total_run_stopwatch.stop();
                run_durations.update_run_durations(&total_run_stopwatch);
                run_durations.run_processed_items = processed_items;
                println!("Speed test completed, storing results.");
                Ok(run_durations)
            }
        }
    }
    
    fn large_queue_value(&self) -> Result<RunDurations, Box<dyn Error>> {
        println!("Starting a speed test of large queue processing using serde Value");
        let load_all_messages: Result<Vec<String>, Box<dyn Error>> =
            combine_files_of_message_type(MessageType::All);
        match load_all_messages {
            Err(load_error) => Err(load_error),
            Ok(all_messages) => {
                let mut run_durations: RunDurations = RunDurations::new();
                println!("Loaded data successfully");
                let mut rng: ThreadRng = thread_rng();
                println!("Increasing queue size by a factor of {}", self);
                let mut test_message_queue: Vec<String> = all_messages.duplicate_contents(self);
                println!("Queue contains {} messages", test_message_queue.len());
                run_durations.run_processed_items = test_message_queue.len();
                let successfully_decoded_items: Arc<Mutex<Vec<Value>>> = Arc::new(Mutex::new(Vec::new()));
                let mut total_run_stopwatch: Stopwatch = Stopwatch::start(StopwatchType::TotalRun);
                println!("Shuffling the queue");
                test_message_queue.shuffle(&mut rng);
                println!("Deserialising the queue");
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
                println!("Deserialisation completed, shuffling the successful results");
                successfully_decoded_items_lock.shuffle(&mut rng);
                println!("Serialising the queue");
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

fn round_results(library_result: Result<RunDurations, Box<dyn Error>>, value_result: Result<RunDurations, Box<dyn Error>>) {
    match (library_result, value_result) {
        (Err(library_error), _) => println!("Library test had an error: {}", library_error),
        (_, Err(value_error)) => println!("Value test had an error: {}", value_error),
        (Ok(library), Ok(value)) => {
            let comparison: SpeedTestComparisons = SpeedTestComparisons {
                test_one_type: SpeedTestType::IteratingRoundsLibrary,
                test_one_results: library,
                test_two_type: SpeedTestType::IteratingRoundsValue,
                test_two_results: value
            };
            comparison.compare_rounds();
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