// take in two command line argument of server:port to connect to
// and then connect to the server and port and print out the response

extern crate acars_vdlm2_parser;
extern crate clap as clap;
extern crate hex;

use acars_vdlm2_parser::helpers::encode_adsb_raw_input::format_adsb_raw_frame_from_str;
use acars_vdlm2_parser::DecodeMessage;
use acars_vdlm2_parser::ExpectedMessageType;
use clap::Parser;
use log::LevelFilter;
use log::{error, info};
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use std::error::Error;
use std::fs;
use std::io::prelude::*;
use std::net::TcpStream;
use std::path::Path;
use tokio::time::{sleep, Duration};

const MAX_TCP_BUFFER_SIZE: usize = 8000;

#[derive(Parser, Debug, Clone, Default)]
#[command(name = "acars_vdlm2_parser Library Tester", author, version, about, long_about = None)]
pub struct Input {
    /// Server(s) to connect to
    #[clap(long, required = true)]
    list_of_servers: Vec<String>,
    /// Output file to log to
    #[clap(long, value_parser, default_value = "./output.log")]
    log_file_path: String,
    /// If the log file exists, remove it.
    #[clap(long, value_parser)]
    remove_existing_log_file: bool,
    /// The run duration of the program, in seconds
    #[clap(long, default_value = "10")]
    run_duration: f64,
}

fn process_json_packet(
    line: String,
    successful_decodes: &mut u32,
    json_tries: &mut Vec<String>,
    server: &str,
) {
    // this is a valid json line
    if let Ok(message) = DecodeMessage::decode_message(&line, ExpectedMessageType::Json) {
        info!(target: server, "Message: {:?}", message);
        *successful_decodes += 1;
    } else {
        error!(target: server, "Failed to decode JSON");
    }

    // if we have decoded any valid json there must never be a partially
    // decoded json waiting around. Ensure json_tries is empty
    if !json_tries.is_empty() {
        *json_tries = vec![];
    }
}

fn process_json_packet_from_parts(
    line: String,
    successful_decodes: &mut u32,
    json_tries: &mut Vec<String>,
    server: &str,
) {
    let mut json_line: String = json_tries.pop().unwrap();
    json_line.push_str(&line);

    if let Ok(message) = DecodeMessage::decode_message(&json_line, ExpectedMessageType::Json) {
        info!(
            target: server,
            "Message from reconstituted lines: {:?}", message
        );
        *successful_decodes += 1;
    } else {
        error!(
            target: server,
            "Failed to decode JSON from reconstituted lines"
        );
    }
}

fn process_raw_packet(
    line: String,
    successful_decodes: &mut u32,
    adsb_raw_tries: &mut Vec<String>,
    server: &str,
) {
    let formatted_line = format_adsb_raw_frame_from_str(&line);
    match formatted_line {
        Ok(good_line) => {
            if let Ok(hex_line) = hex::decode(good_line) {
                if let Ok(message) =
                    DecodeMessage::decode_message(&hex_line, ExpectedMessageType::Raw)
                {
                    info!(target: server, "Message: {:?}", message);
                    *successful_decodes += 1;
                }
            }

            if !adsb_raw_tries.is_empty() {
                *adsb_raw_tries = vec![];
            }
        }
        Err(_) => adsb_raw_tries.push(line),
    }
}

fn process_raw_packet_from_parts(
    line: String,
    successful_decodes: &mut u32,
    adsb_raw_tries: &mut Vec<String>,
    server: &str,
) {
    // this may be the second part of a cut off ADSB Raw frame. Reconstruct, and attempted decode
    let mut attempted_line = adsb_raw_tries.pop().unwrap();

    attempted_line.push_str(&line);

    if let Ok(message) = format_adsb_raw_frame_from_str(&attempted_line) {
        if let Ok(hex_line) = hex::decode(&message) {
            if let Ok(decoded_message) =
                DecodeMessage::decode_message(&hex_line, ExpectedMessageType::Raw)
            {
                info!(
                    target: server,
                    "Message from reconstituted raw frame: {:?}", decoded_message
                );
                *successful_decodes += 1;
            }
        } else {
            error!(
                target: server,
                "Failed to create hex from reconstituted raw frame {message}"
            );
        }
    } else {
        error!(
            target: server,
            "Failed to decode reconstituted ADSB Raw frame: {}", attempted_line
        );
    }
}

fn connect_to_and_monitor_server(server: &str) {
    info!(target: server, "Connecting to {:?}", server);

    if let Ok(mut stream) = TcpStream::connect(server) {
        info!(target: server, "Connected to {server}");
        let mut buffer: [u8; MAX_TCP_BUFFER_SIZE] = [0; MAX_TCP_BUFFER_SIZE];
        let mut json_tries: Vec<String> = Vec::new();
        let mut adsb_raw_tries: Vec<String> = Vec::new();
        loop {
            match stream.read(&mut buffer) {
                Ok(bytes_read) => {
                    info!(target: server, "Read {} bytes", bytes_read);
                    let mut successful_decodes: u32 = 0;
                    let mut attempted_decodes: u32 = 0;
                    // create a string from the buffer and split it on newline
                    let buffer_string = String::from_utf8_lossy(&buffer[..bytes_read]);
                    let buffer_lines = buffer_string.split('\n');
                    for line in buffer_lines {
                        if !line.is_empty() {
                            attempted_decodes += 1;
                            // TODO: Perhaps deal with a suuuuuuuuper long json message
                            // that spans more than two packets? Probably such an outlier it isn't
                            // worth doing?

                            if line.starts_with('{') && line.ends_with('}') {
                                process_json_packet(
                                    line.to_string(),
                                    &mut successful_decodes,
                                    &mut json_tries,
                                    server,
                                )
                            } else if line.starts_with('{') {
                                // this is the start of a json message, but it's been cut off. Likely
                                // it will be in the next packet
                                json_tries.push(line.to_string());
                            } else if line.ends_with('}') {
                                // this is the end of a json packet that likely started with the previous packet
                                if !json_tries.is_empty() {
                                    process_json_packet_from_parts(
                                        line.to_string(),
                                        &mut successful_decodes,
                                        &mut json_tries,
                                        server,
                                    );
                                } else {
                                    error!(target: server,"Failed to decode JSON. Received end of JSON line but had no start");
                                }
                            } else if line.starts_with('*') {
                                process_raw_packet(
                                    line.to_string(),
                                    &mut successful_decodes,
                                    &mut adsb_raw_tries,
                                    server,
                                );
                            } else if !adsb_raw_tries.is_empty()
                                && (line.ends_with(';') || line.ends_with(";\n"))
                            {
                                process_raw_packet_from_parts(
                                    line.to_string(),
                                    &mut successful_decodes,
                                    &mut adsb_raw_tries,
                                    server,
                                );
                            } else {
                                error!(
                                    target: server,
                                    "Unknown message type!\nEntry: {attempted_decodes} Attempted: {line}"
                                );
                            }
                        }
                    }
                    info!(target: server, "Successful decodes: {}", successful_decodes);
                    info!(
                        target: server,
                        "Total messages attempted: {}", attempted_decodes
                    );
                }
                Err(e) => error!(target: server, "Error: {}", e),
            }
        }
    } else {
        error!(target: server, "Could not connect to {}", &server);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Input = Input::parse();
    let run_duration = (args.run_duration * 1000.0) as u64;

    if args.remove_existing_log_file && Path::new(&args.log_file_path).exists() {
        fs::remove_file(&args.log_file_path)?;
    }

    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S)}:{l}:{t}:{m}\n",
        )))
        .build(args.log_file_path)?;

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Info))?;

    log4rs::init_config(config)?;

    for server in args.list_of_servers {
        tokio::spawn(async move {
            connect_to_and_monitor_server(&server);
        });
    }
    println!("Started up, running for {run_duration} milliseconds.");
    info!(target: "Main", "Started up, running for {run_duration} milliseconds.");
    sleep(Duration::from_millis(run_duration)).await;
    std::process::exit(0);
}
