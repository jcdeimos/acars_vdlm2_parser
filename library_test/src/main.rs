// take in two command line argument of server:port to connect to
// and then connect to the server and port and print out the response

extern crate acars_vdlm2_parser;
extern crate hex;

use acars_vdlm2_parser::helpers::encode_adsb_raw_input::format_adsb_raw_frame_from_str;
use acars_vdlm2_parser::DecodeMessage;
use acars_vdlm2_parser::ExpectedMessageType;
use std::env;
use std::io::prelude::*;
use std::net::TcpStream;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("Connecting to {}", &args[1]);

    if let Ok(mut stream) = TcpStream::connect(&args[1]) {
        println!("Connected to {}", &args[1]);
        let mut buffer: [u8; 3000] = [0; 3000];
        let mut json_tries: Vec<String> = Vec::new();
        loop {
            match stream.read(&mut buffer) {
                Ok(bytes_read) => {
                    println!("Read {} bytes", bytes_read);
                    let mut successful_decodes = 0;
                    let mut attempted_decodes = 0;
                    // create a string from the buffer and split it on newline
                    let buffer_string = String::from_utf8_lossy(&buffer[..bytes_read]);
                    let buffer_lines = buffer_string.split("\n");
                    for line in buffer_lines {
                        if line.len() > 0 {
                            attempted_decodes += 1;
                            // TODO: Perhaps deal with a suuuuuuuuper long json message
                            // that spans more than two packets? Probably such an outlier it isn't
                            // worth doing?

                            if line.starts_with('{') && line.ends_with('}') {
                                // this is a valid json line
                                if let Ok(message) =
                                    DecodeMessage::decode_message(line, ExpectedMessageType::Json)
                                {
                                    println!("Message: {:?}", message);
                                    successful_decodes += 1;
                                } else {
                                    println!("Failed to decode JSON");
                                }

                                // if we have decoded any valid json there must never be a partially
                                // decoded json waiting around. Ensure json_tries is empty
                                if json_tries.len() > 0 {
                                    json_tries = vec![];
                                }
                            } else if line.starts_with('{') {
                                // this is the start of a json message, but it's been cut off. Likely
                                // it will be in the next packet
                                json_tries.push(line.to_string());
                            } else if line.ends_with('}') {
                                // this is the end of a json packet that likely started with the previous packet
                                if json_tries.len() > 0 {
                                    let mut json_line: String = json_tries.pop().unwrap();
                                    json_line.push_str(line);

                                    if let Ok(message) = DecodeMessage::decode_message(
                                        &json_line,
                                        ExpectedMessageType::Json,
                                    ) {
                                        println!("Message from reconstituted lines: {:?}", message);
                                        successful_decodes += 1;
                                    } else {
                                        println!("Failed to decode JSON from reconstituted lines");
                                    }
                                } else {
                                    println!("Failed to decode JSON. Received end of JSON line but had no start");
                                }
                            } else if line.starts_with('*') {
                                let formatted_line = format_adsb_raw_frame_from_str(&line);
                                if let Ok(hex_line) = hex::decode(formatted_line) {
                                    if let Ok(message) = DecodeMessage::decode_message(
                                        &hex_line,
                                        ExpectedMessageType::Raw,
                                    ) {
                                        println!("Message: {:?}", message);
                                        successful_decodes += 1;
                                    }
                                }
                            } else {
                                println!("Unknown message type!\nAttempted: {}", line);
                            }
                        }
                    }
                    println!("Successful decodes: {}", successful_decodes);
                    println!("Total messages attempted: {}", attempted_decodes);
                    buffer = [0; 3000];
                }
                Err(e) => println!("Error: {}", e),
            }
        }
    } else {
        println!("Could not connect to {}", &args[1]);
    }
}
