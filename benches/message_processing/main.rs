use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, Throughput, BenchmarkGroup};
use criterion::measurement::WallTime;
use glob::{glob, GlobResult, Paths, PatternError};
use rand::seq::SliceRandom;
use rand::rngs::ThreadRng;
use rand::thread_rng;
use rayon::prelude::*;
use acars_vdlm2_parser::{AcarsVdlm2Message, DecodeMessage};

fn load_data() -> Option<Vec<String>> {
    combine_found_files(glob("test_files/*"))
}

pub fn combine_found_files(find_files: Result<Paths, PatternError>) -> Option<Vec<String>> {
    let Ok(file_paths) = find_files else {
        eprintln!("Failed to load files.");
        return None;
    };
    let mut loaded_contents: Vec<String> = Vec::new();
    for file in file_paths {
        append_lines(file, &mut loaded_contents)
    }
    Some(loaded_contents)
}

fn append_lines(file: GlobResult, data: &mut Vec<String>) {
    let Ok(file_path) = file else {
        return;
    };
    let Some(mut contents) = read_test_file(file_path.as_path()) else {
        return;
    };
    data.append(&mut contents);
}

fn read_test_file(filepath: impl AsRef<Path>) -> Option<Vec<String>> {
    let Ok(open_file) = File::open(filepath) else {
        return None;
    };
    let Ok(lines) = BufReader::new(open_file).lines().collect() else {
        return None;
    };
    Some(lines)
}

fn duplicate_messages(messages: Vec<String>) -> Vec<String> {
    let mut duplication: Vec<String> = messages.to_vec();
    let mut rng: ThreadRng = thread_rng();
    println!("Starting with {} items.", duplication.len());
    for _ in 0..270 {
        let mut more: Vec<String> = messages.to_vec();
        duplication.append(&mut more);
        println!("We now have {} items.", duplication.len());
    }
    duplication.shuffle(&mut rng);
    println!("Finished with {} items.", duplication.len());
    duplication
}

fn duplicate_parsed_messages(messages: Vec<AcarsVdlm2Message>) -> Vec<AcarsVdlm2Message> {
    let mut duplication: Vec<AcarsVdlm2Message> = messages.to_vec();
    let mut rng: ThreadRng = thread_rng();
    println!("Starting with {} items.", duplication.len());
    for _ in 0..270 {
        let mut more: Vec<AcarsVdlm2Message> = messages.to_vec();
        duplication.append(&mut more);
        println!("We now have {} items.", duplication.len());
    }
    duplication.shuffle(&mut rng);
    println!("Finished with {} items.", duplication.len());
    duplication
}

fn ingest(data: &[String]) -> Vec<AcarsVdlm2Message> {
    let ok_messages: Arc<Mutex<Vec<AcarsVdlm2Message>>> = Arc::new(Mutex::new(Vec::new()));
    data.par_iter().for_each(| message: &String | {
        if let Ok(message) = message.decode_message() {
            ok_messages.lock().unwrap().push(message);
        }
    });
    let ok_messages_lock: MutexGuard<Vec<AcarsVdlm2Message>> = ok_messages.lock().unwrap();
    ok_messages_lock.to_vec()
}

fn process_messages_from_string(data: &[String]) {
    let ok_messages: Arc<Mutex<Vec<AcarsVdlm2Message>>> = Arc::new(Mutex::new(Vec::new()));
    data.par_iter().for_each(| message: &String | {
        if let Ok(message) = message.decode_message() {
            ok_messages.lock().unwrap().push(message);
        }
    });
}

fn process_messages_to_string(data: &[AcarsVdlm2Message]) {
    let ok_messages: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    data.par_iter().for_each(| message: &AcarsVdlm2Message | {
        if let Ok(message) = message.to_string() {
            ok_messages.lock().unwrap().push(message);
        }
    });
}

pub fn bench_processing_from_string(c: &mut Criterion) {
    println!("Loading data");
    let Some(loaded_data) = load_data() else {
        eprintln!("Failed to load data.");
        return;
    };
    println!("Duplicating and shuffling data.");
    let duplicated_data: Vec<String> = duplicate_messages(loaded_data);
    println!("Starting on the benching.");
    let mut iter_run: BenchmarkGroup<WallTime> = c.benchmark_group("message_processing_count_from_string");
    iter_run.measurement_time(Duration::from_secs(60));
    iter_run.sample_size(200);
    
    let iter_batch_sizes: Vec<usize> = vec![1, 10, 100, 1_000, 5_000, 10_000, 25_000, 50_000, 75_000, 100_000];
    for batch_size in &iter_batch_sizes {
        let test_snippet: Vec<String> = duplicated_data[0..*batch_size].to_vec();
        iter_run.throughput(Throughput::Elements(iter_batch_sizes.len() as u64));
        iter_run.bench_with_input(BenchmarkId::new("AcarsVdlmMessage", batch_size), &test_snippet, |b, data|  {
            b.iter(|| process_messages_from_string(&data));
        });
    }
}

pub fn bench_processing_to_string(c: &mut Criterion) {
    println!("Loading data");
    let Some(loaded_data) = load_data() else {
        eprintln!("Failed to load data.");
        return;
    };
    let loaded_messages: Vec<AcarsVdlm2Message> = ingest(&loaded_data);
    println!("Duplicating and shuffling data.");
    let duplicated_data: Vec<AcarsVdlm2Message> = duplicate_parsed_messages(loaded_messages);
    println!("Starting on the benching.");
    let mut iter_run: BenchmarkGroup<WallTime> = c.benchmark_group("message_processing_count_to_string");
    iter_run.measurement_time(Duration::from_secs(60));
    iter_run.sample_size(200);
    
    let iter_batch_sizes: Vec<usize> = vec![1, 10, 100, 1_000, 5_000, 10_000, 25_000, 50_000, 75_000, 100_000];
    for batch_size in &iter_batch_sizes {
        let test_snippet: Vec<AcarsVdlm2Message> = duplicated_data[0..*batch_size].to_vec();
        iter_run.throughput(Throughput::Elements(iter_batch_sizes.len() as u64));
        iter_run.bench_with_input(BenchmarkId::new("AcarsVdlmMessage", batch_size), &test_snippet, |b, data|  {
            b.iter(|| process_messages_to_string(&data));
        });
    }
}

criterion_group!(benches, bench_processing_from_string, bench_processing_to_string);
criterion_main!(benches);