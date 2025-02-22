use crate::{
    log_error, log_info, log_warning,
    types::message::{
        GameMessage, RegisterTeam, RegisterTeamResult, SubscribePlayer, SubscribePlayerResult,
    },
};
use std::io::{self};
use std::net::TcpStream;

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

/// Connects to the server and returns the TCP stream.
///
/// # Arguments
///
/// * `server_address` - The address of the server.
///
/// # Returns
///
/// An `io::Result` with the TCP stream if the connection was successful, or an error message.
///
/// # Errors
///
/// Returns an error if the connection to the server failed.
pub fn connect_to_server(server_address: &str) -> io::Result<TcpStream> {
    log_info!("Connecting to {}...", server_address);
    match TcpStream::connect(server_address) {
        Ok(stream) => {
            log_info!("Connected to the server");
            return Ok(stream);
        }
        Err(error) => {
            log_error!("Connection error: {}", error);
            return Err(error);
        }
    }
}

/// Registers a team and returns the registration token.
///
/// # Arguments
///
/// * `stream` - A mutable reference to the TCP stream.
///
/// # Returns
///
/// The registration token if the registration was successful, or an error message.
///
/// # Errors
///
/// Returns an error if the server response is unexpected.
pub fn register_team(stream: &mut TcpStream) -> io::Result<String> {
    let register_team = RegisterTeam {
        name: String::from("team_1"),
    };
    let message = GameMessage::RegisterTeam(register_team);
    message.send(stream)?;
    log_info!("Registration message sent to the server");

    match GameMessage::receive(stream)? {
        GameMessage::RegisterTeamResult(RegisterTeamResult::Ok {
            expected_players,
            registration_token,
        }) => {
            log_info!(
                "Team registered. Waiting for players: {}. Registering token: {}",
                expected_players,
                registration_token
            );
            Ok(registration_token)
        }
        GameMessage::RegisterTeamResult(RegisterTeamResult::Err(e)) => {
            log_error!("Registration failed: {:?}", e);
            Err(io::Error::new(io::ErrorKind::Other, "Registration failed"))
        }
        _ => {
            log_warning!("Unexpected server response");
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Unexpected server response",
            ))
        }
    }
}

/// Registers a player by using the registration token.
///
/// # Arguments
///
/// * `stream` - A mutable reference to the TCP stream.
/// * `registration_token` - The registration token of the team.
///
/// # Returns
///
/// An empty `std::io::Result`.
///
/// # Errors
///
/// Returns an error if the registration has failed or the server response is unexpected.
pub fn register_player(
    stream: &mut TcpStream,
    registration_token: &str,
    player_name: &str,
) -> io::Result<String> {
    let subscribe_player: SubscribePlayer = SubscribePlayer {
        name: String::from(player_name),
        registration_token: registration_token.to_string(),
    };

    let message: GameMessage = GameMessage::SubscribePlayer(subscribe_player);
    message.send(stream)?;
    log_info!("SubscribePlayer message sent");

    match GameMessage::receive(stream)? {
        GameMessage::SubscribePlayerResult(SubscribePlayerResult::Ok) => {
            log_info!("Player successfully registered. Waiting for first RadarView...");

            match GameMessage::receive(stream) {
                Ok(GameMessage::RadarView(encoded_radar)) => {
                    log_info!("First RadarView received: {}", encoded_radar);

                    return Ok(encoded_radar);
                }
                Ok(other_message) => {
                    log_warning!(
                        "Unexpected message instead of RadarView: {:?}",
                        other_message
                    );

                    let error: std::io::Error =
                        io::Error::new(io::ErrorKind::InvalidData, "Unexpected server response");

                    return Err(error);
                }
                Err(err) => {
                    log_error!("Failed to receive first RadarView: {}", err);

                    let error: std::io::Error =
                        io::Error::new(io::ErrorKind::Other, "Failed to receive RadarView");

                    return Err(error);
                }
            }
        }
        GameMessage::SubscribePlayerResult(SubscribePlayerResult::Err(e)) => {
            log_error!("Player registration failed: {:?}", e);

            let error: std::io::Error =
                io::Error::new(io::ErrorKind::Other, "Player registration failed");

            return Err(error);
        }
        _ => {
            log_warning!("Unexpected server response");

            let error: std::io::Error =
                io::Error::new(io::ErrorKind::InvalidData, "Unexpected server response");

            return Err(error);
        }
    }
}
