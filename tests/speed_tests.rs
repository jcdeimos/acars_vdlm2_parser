mod common;

use std::error::Error;
use rand::prelude::SliceRandom;
use rand::rngs::ThreadRng;
use rand::thread_rng;
use acars_vdlm2_parser::{AcarsVdlm2Message, DecodeMessage, MessageResult};
use crate::common::{combine_files_of_message_type, ContentDuplicator, DisplaySpeedTestResults, StopwatchType, MessageType, RunDurations, SpeedTestDurations, SpeedTestType, Stopwatch, test_enum_serialisation, TestRun};

#[test]
#[ignore]
fn test_serialisation_deserialisation_speed() {
    100.iterating_rounds().display_results(SpeedTestType::IteratingRounds);
    500.iterating_rounds().display_results(SpeedTestType::IteratingRounds);
    1000.large_queue().display_results(SpeedTestType::LargeQueue);
    5000.large_queue().display_results(SpeedTestType::LargeQueue);
    10_000.large_queue().display_results(SpeedTestType::LargeQueue);
}

/// Trait for performing speed tests.
pub(crate) trait SpeedTest {
    fn iterating_rounds(&self) -> Result<RunDurations, Box<dyn Error>>;
    fn large_queue(&self) -> Result<RunDurations, Box<dyn Error>>;
}

/// `SpeedTest` implemented for `i32`
///
/// Run x iterations, invoked as `int.speed_test()`
impl SpeedTest for i64 {
    fn iterating_rounds(&self) -> Result<RunDurations, Box<dyn Error>> {
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
                let mut total_run_stopwatch: Stopwatch = Stopwatch::start(StopwatchType::TotalRun);
                for run in 0..*self {
                    println!("Run {}/{} =>", run + 1, &self);
                    let mut test_run: TestRun = TestRun::new(&run);
                    all_messages.shuffle(&mut rng);
                    let mut run_deserialisation_successful_items: Vec<AcarsVdlm2Message> = Vec::new();
                    let mut deserialisation_run_stopwatch: Stopwatch = Stopwatch::start(StopwatchType::AllDeser);
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
                    println!("Run contained {}/{} successful items", run_deserialisation_successful_items.len(), all_messages.len());
                    test_run.update_run_durations(&deserialisation_run_stopwatch);
                    successfully_decoded_items.shuffle(&mut rng);
                    run_deserialisation_successful_items.shuffle(&mut rng);
                    let mut serialisation_run_stopwatch: Stopwatch = Stopwatch::start(StopwatchType::AllSer);
                    for message in &run_deserialisation_successful_items {
                        test_enum_serialisation(message);
                    }
                    serialisation_run_stopwatch.stop();
                    test_run.update_run_durations(&serialisation_run_stopwatch);
                    println!("Decoded items now contains {} items", successfully_decoded_items.len());
                    let mut additive_serialisation_run_stopwatch: Stopwatch = Stopwatch::start(StopwatchType::AddSer);
                    for message in &successfully_decoded_items {
                        test_enum_serialisation(message);
                    }
                    additive_serialisation_run_stopwatch.stop();
                    test_run.update_run_durations(&additive_serialisation_run_stopwatch);
                    run_durations.test_runs.push(test_run);
                }
                run_durations.run_processed_items = successfully_decoded_items.len();
                successfully_decoded_items.shuffle(&mut rng);
                let mut final_cumulative_serialisation_stopwatch: Stopwatch = Stopwatch::start(StopwatchType::LargeQueueSer);
                for message in &successfully_decoded_items {
                    test_enum_serialisation(message);
                }
                final_cumulative_serialisation_stopwatch.stop();
                total_run_stopwatch.stop();
                run_durations.update_run_durations(&final_cumulative_serialisation_stopwatch);
                run_durations.update_run_durations(&total_run_stopwatch);
                println!("Speed test completed, storing results.");
                Ok(run_durations)
            }
        }
    }
    
    fn large_queue(&self) -> Result<RunDurations, Box<dyn Error>> {
        println!("Starting a speed test of large queue processing");
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
                let mut successfully_decoded_items: Vec<AcarsVdlm2Message> = Vec::new();
                let mut total_run_stopwatch: Stopwatch = Stopwatch::start(StopwatchType::TotalRun);
                println!("Shuffling the queue");
                test_message_queue.shuffle(&mut rng);
                println!("Deserialising the queue");
                let mut deserialisation_run_stopwatch: Stopwatch = Stopwatch::start(StopwatchType::LargeQueueDeser);
                for entry in &test_message_queue {
                    let parsed_message: MessageResult<AcarsVdlm2Message> = entry.decode_message();
                    match parsed_message {
                        Err(_) => {}
                        Ok(decoded_message) => {
                            successfully_decoded_items.push(decoded_message.clone());
                        }
                    }
                }
                deserialisation_run_stopwatch.stop();
                run_durations.update_run_durations(&deserialisation_run_stopwatch);
                println!("Deserialisation completed, shuffling the successful results");
                successfully_decoded_items.shuffle(&mut rng);
                println!("Serialising the queue");
                let mut serialisation_run_stopwatch: Stopwatch = Stopwatch::start(StopwatchType::LargeQueueSer);
                for message in &successfully_decoded_items {
                    test_enum_serialisation(message);
                }
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