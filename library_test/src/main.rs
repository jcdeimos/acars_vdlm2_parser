// take in two command line argument of server:port to connect to
// and then connect to the server and port and print out the response

extern crate acars_vdlm2_parser;
extern crate hex;

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
        loop {
            match stream.read(&mut buffer) {
                Ok(bytes_read) => {
                    println!("Read {} bytes", bytes_read);
                    // create a string from the buffer and split it on newline
                    let buffer_string = String::from_utf8_lossy(&buffer[..bytes_read]);
                    let buffer_lines = buffer_string.split("\n");
                    for line in buffer_lines {
                        if line.len() > 0 {
                            // remove * from the start of the line and ; \n from the end
                            let line = line.trim_start_matches('*');
                            let line = line.trim_end_matches(";");
                            let line = line.trim_end_matches("\n"); // should be unnecessary
                                                                    // decode the message
                            if let Ok(hex_line) = hex::decode(line) {
                                if let Ok(message) = DecodeMessage::decode_message(
                                    &hex_line,
                                    ExpectedMessageType::Raw,
                                ) {
                                    println!("Message: {:?}", message);
                                }
                            } else if let Ok(message) =
                                DecodeMessage::decode_message(line, ExpectedMessageType::Json)
                            {
                                println!("Message: {:?}", message);
                            } else {
                                println!(
                                    "Could not decode message line: {} length: {}",
                                    line,
                                    line.len()
                                );
                            }
                        }
                    }
                    buffer = [0; 3000];
                }
                Err(e) => println!("Error: {}", e),
            }
        }
    } else {
        println!("Could not connect to {}", &args[1]);
    }
}
