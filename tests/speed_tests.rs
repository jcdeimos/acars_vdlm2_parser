mod common;

use std::error::Error;
use std::sync::{Arc, Mutex, MutexGuard};
use rand::prelude::SliceRandom;
use rand::rngs::ThreadRng;
use rand::thread_rng;
use acars_vdlm2_parser::{AcarsVdlm2Message, DecodeMessage, MessageResult};
use crate::common::{combine_files_of_message_type, ContentDuplicator, DisplaySpeedTestResults, StopwatchType, MessageType, RunDurations, SpeedTestDurations, SpeedTestType, Stopwatch, test_enum_serialisation, TestRun, test_value_serialisation};
use rayon::prelude::*;
use serde_json::Value;

#[test]
#[ignore]
fn test_serialisation_deserialisation_speed() {
    let rounds_100_library: Result<RunDurations, Box<dyn Error>> = 100.iterating_rounds_library();
    let rounds_500_library: Result<RunDurations, Box<dyn Error>> = 500.iterating_rounds_library();
    let large_1000_library: Result<RunDurations, Box<dyn Error>> = 1000.large_queue_library();
    let large_5000_library: Result<RunDurations, Box<dyn Error>> = 5000.large_queue_library();
    let large_10_000_library: Result<RunDurations, Box<dyn Error>> = 10_000.large_queue_library();
    let rounds_library: Vec<Result<RunDurations, Box<dyn Error>>> = vec![rounds_100_library, rounds_500_library];
    let large_queue_library: Vec<Result<RunDurations, Box<dyn Error>>> = vec![large_1000_library, large_5000_library, large_10_000_library];
    for round in rounds_library {
        round.display_results(SpeedTestType::IteratingRoundsLibrary);
    }
    for queue in large_queue_library {
        queue.display_results(SpeedTestType::LargeQueueLibrary);
    }
    let rounds_100_value: Result<RunDurations, Box<dyn Error>> = 100.iterating_rounds_library();
    let rounds_500_value: Result<RunDurations, Box<dyn Error>> = 500.iterating_rounds_library();
    let large_1000_value: Result<RunDurations, Box<dyn Error>> = 1000.large_queue_library();
    let large_5000_value: Result<RunDurations, Box<dyn Error>> = 5000.large_queue_library();
    let large_10_000_value: Result<RunDurations, Box<dyn Error>> = 10_000.large_queue_library();
    let rounds_value: Vec<Result<RunDurations, Box<dyn Error>>> = vec![rounds_100_value, rounds_500_value];
    let large_queue_value: Vec<Result<RunDurations, Box<dyn Error>>> = vec![large_1000_value, large_5000_value, large_10_000_value];
    for round in rounds_value {
        round.display_results(SpeedTestType::IteratingRoundsValue);
    }
    for queue in large_queue_value {
        queue.display_results(SpeedTestType::LargeQueueValue);
    }
    
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
                let run_durations: Arc<Mutex<RunDurations>> = Arc::new(Mutex::new(RunDurations::new()));
                let mut total_run_stopwatch: Stopwatch = Stopwatch::start(StopwatchType::TotalRun);
                (0..*self).into_par_iter().for_each(|run| {
                    let run_count: i64 = run + 1;
                    let mut test_run: TestRun = TestRun::new(&run_count);
                    println!("Running test with a factor of {}.", run_count);
                    let mut run_messages: Vec<String> = all_messages.to_vec().duplicate_contents(&run_count);
                    test_run.run_items = run_messages.len();
                    let mut rng: ThreadRng = thread_rng();
                    run_messages.shuffle(&mut rng);
                    let mut run_deserialisation_successful_items: Vec<AcarsVdlm2Message> = Vec::new();
                    let mut deserialisation_run_stopwatch: Stopwatch = Stopwatch::start(StopwatchType::AllDeser);
                    for entry in &run_messages {
                        let parsed_message: MessageResult<AcarsVdlm2Message> = entry.decode_message();
                        match parsed_message {
                            Err(_) => {}
                            Ok(decoded_message) => {
                                run_deserialisation_successful_items.push(decoded_message.clone());
                            }
                        }
                    }
                    deserialisation_run_stopwatch.stop();
                    println!("Run contained {}/{} successful items", run_deserialisation_successful_items.len(), run_messages.len());
                    test_run.update_run_durations(&deserialisation_run_stopwatch);
                    run_deserialisation_successful_items.shuffle(&mut rng);
                    let mut serialisation_run_stopwatch: Stopwatch = Stopwatch::start(StopwatchType::AllSer);
                    for message in &run_deserialisation_successful_items {
                        test_enum_serialisation(message);
                    }
                    serialisation_run_stopwatch.stop();
                    test_run.update_run_durations(&serialisation_run_stopwatch);
                    run_durations.lock().unwrap().test_runs.push(test_run);
                });
                let mut run_lock: MutexGuard<RunDurations> = run_durations.lock().unwrap();
                total_run_stopwatch.stop();
                run_lock.update_run_durations(&total_run_stopwatch);
                println!("Speed test completed, storing results.");
                Ok(run_lock.clone())
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
                let run_durations: Arc<Mutex<RunDurations>> = Arc::new(Mutex::new(RunDurations::new()));
                let mut total_run_stopwatch: Stopwatch = Stopwatch::start(StopwatchType::TotalRun);
                (0..*self).into_par_iter().for_each(|run| {
                    let run_count: i64 = run + 1;
                    let mut test_run: TestRun = TestRun::new(&run_count);
                    println!("Running test with a factor of {}.", run_count);
                    let mut run_messages: Vec<String> = all_messages.to_vec().duplicate_contents(&run_count);
                    test_run.run_items = run_messages.len();
                    let mut rng: ThreadRng = thread_rng();
                    run_messages.shuffle(&mut rng);
                    let mut run_deserialisation_successful_items: Vec<Value> = Vec::new();
                    let mut deserialisation_run_stopwatch: Stopwatch = Stopwatch::start(StopwatchType::AllDeser);
                    for entry in &run_messages {
                        let parsed_message: MessageResult<Value> = serde_json::from_str(&entry);
                        match parsed_message {
                            Err(_) => {}
                            Ok(decoded_message) => {
                                run_deserialisation_successful_items.push(decoded_message.clone());
                            }
                        }
                    }
                    deserialisation_run_stopwatch.stop();
                    println!("Run contained {}/{} successful items", run_deserialisation_successful_items.len(), run_messages.len());
                    test_run.update_run_durations(&deserialisation_run_stopwatch);
                    run_deserialisation_successful_items.shuffle(&mut rng);
                    let mut serialisation_run_stopwatch: Stopwatch = Stopwatch::start(StopwatchType::AllSer);
                    for message in &run_deserialisation_successful_items {
                        test_value_serialisation(message);
                    }
                    serialisation_run_stopwatch.stop();
                    test_run.update_run_durations(&serialisation_run_stopwatch);
                    run_durations.lock().unwrap().test_runs.push(test_run);
                });
                let mut run_lock: MutexGuard<RunDurations> = run_durations.lock().unwrap();
                total_run_stopwatch.stop();
                run_lock.update_run_durations(&total_run_stopwatch);
                println!("Speed test completed, storing results.");
                Ok(run_lock.clone())
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

trait SpeedTestDisplayRanges {
    fn output_speed_test_ranges(&self, test: &str);
    fn output_speed_test_gaps(&self, test: &str);
}

impl SpeedTestDisplayRanges for SpeedTestDurations {
    fn output_speed_test_ranges(&self, test: &str) {
        if let (Some(shortest_run), Some(longest_run)) = (self.shortest_duration, self.longest_duration) {
            println!("{} run stats:\nShortest: {}ms (Run {})\nLongest: {}ms (Run {})\nAverage: {}ms",
                     test, shortest_run, self.shortest_run_number, longest_run, self.longest_run_number, self.average_duration);
        }
    }
    
    fn output_speed_test_gaps(&self, test: &str) {
        if let (Some(shortest_run), Some(longest_run)) = (self.shortest_duration, self.longest_duration) {
            println!("{} run stats:\nShortest gap: {}ms\nLongest gap: {}ms\nAverage: {}ms",
                     test, shortest_run, longest_run, self.average_duration);
        }
    }
}
