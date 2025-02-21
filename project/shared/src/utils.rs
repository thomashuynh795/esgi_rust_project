pub fn log_message(message: &str) {
    println!("LOG: {}", message);
}

/// Decodes a Base64-encoded string into an array of bytes.
///
/// # Arguments
///
/// * `encoded_input` - A Base64-encoded string.
///
/// # Returns
///
/// A `Result` with the decoded bytes if the input is valid, or an error message.
///
/// # Errors
///
/// Returns an error if the encoded input contains invalid characters.
pub fn decode_base64(encoded_input: &str) -> Result<Vec<u8>, &'static str> {
    // Base64 alphabet.
    let base64_table: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut decoded_bytes: Vec<u8> = Vec::new();
    let mut six_bits_packets: Vec<u8> = Vec::new();

    for c in encoded_input.chars() {
        // Stops if the character is a Base64 padding.
        if c == '=' {
            break;
        }

        // Stores the 6-bits value of the character. It's only 6 bits because the Base64.
        let six_bits_value: Option<usize> = base64_table.find(c);
        if six_bits_value.is_none() {
            return Err("Invalid character in Base64 input.");
        }

        // Converts the 6-bits value to a u8 and stores it.
        // This unwrap is safe because we already checked if the value is not None.
        six_bits_packets.push(six_bits_value.unwrap() as u8);
    }

    // Process the 6-bit values into 8-bit bytes.
    let mut buffer: u64 = 0; // 32 bits buffer.
    let mut bits_collected: u8 = 0; // Tracks how many bits are stored.

    // Can extract a byte only if there is an entire one in the buffer.
    for &six_bits in &six_bits_packets {
        // Shifts the buffer to the left by 6 0 and adds the new 6-bits value.
        // The << operator does not return something.
        buffer = (buffer << 6) | (six_bits as u64);
        bits_collected += 6;

        while 8 <= bits_collected {
            bits_collected -= 8;
            // Extracts the 8-bit byte from the buffer and stores it.
            // The >> operator returns the bits shifted to the right and does not modify the original value.
            let byte: u8 = (buffer >> bits_collected) as u8;
            // Stores the byte in the result.
            decoded_bytes.push(byte);

            // Cleans the buffer by removing the extracted byte.
            let extracted_bits: u8 = (buffer >> bits_collected) as u8;
            let shifted_back: u8 = extracted_bits << bits_collected; // Create the bits to remove.
            buffer -= shifted_back as u64; // Subtracts the bits to remove.
        }
    }

    return Ok(decoded_bytes);
}

#[cfg(test)]
mod tests {
    use super::decode_base64;

    #[test]
    fn test_base64_decode_valid() {
        let encoded: &str = "SGVsbG8gd29ybGQh"; // Hello world!
        let decoded: Vec<u8> = decode_base64(encoded).expect("Failed to decode Base64.");
        assert_eq!(decoded, b"Hello world!");
    }

    #[test]
    fn test_base64_invalid_chars() {
        let encoded1: &str = "abc!";
        let encoded2: &str = "abc*";
        assert!(decode_base64(encoded1).is_err());
        assert!(decode_base64(encoded2).is_err());
    }
}
