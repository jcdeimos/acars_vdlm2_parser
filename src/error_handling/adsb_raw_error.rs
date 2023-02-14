use custom_error::custom_error;

custom_error! {pub ADSBRawError
    ByteSequenceWrong{size: u8}             = "Not enough bytes in the sequence to parse the message. ADSB Raw messages should be 14 or 28 bytes long. Found {size} bytes.",
    StringError{message: String}            = "Error converting the byte sequence to a string: {message}",
}
