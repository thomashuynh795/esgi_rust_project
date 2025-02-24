use crate::{
    log_debug, log_error, log_info, log_warning,
    types::message::{
        GameMessage, RegisterTeam, RegisterTeamResult, SubscribePlayer, SubscribePlayerResult,
    },
};
use std::io::{self};
use std::net::TcpStream;

/// Decodes a Base64-encoded string into a vector of bytes.
///
/// # Arguments
///
/// * `input` - The Base64-encoded string.
///
/// # Returns
///
/// A `Result` containing the decoded bytes if successful, or an error message if invalid.
///
/// # Errors
///
/// Returns an error if the input contains invalid characters or has an incorrect length.
pub fn decode_base64(input: &str) -> Result<Vec<u8>, String> {
    // Define the Base64 character set.
    const BASE64_TABLE: &[u8; 64] =
        b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789+/";

    // Creates an array of 128 elements with a default value of 255.
    // Default value of 255 means "invalid character".
    let mut lookup_table: [u8; 128] = [255u8; 128];
    for i in 0..lookup_table.len() {
        log_debug!("byte: {}", lookup_table[i]);
    }
    for (i, &symbol) in BASE64_TABLE.iter().enumerate() {
        lookup_table[symbol as usize] = i as u8; // Store the Base64 index in the table.
    }

    // Validate that the input length is not of the form `4n+1` (which is invalid for Base64).
    if input.len() % 4 == 1 {
        return Err("Invalid Base64 length".to_string());
    }

    let mut decoded_bytes: Vec<u8> = Vec::new();
    let mut char_iter: std::iter::Peekable<std::str::Chars<'_>> = input.chars().peekable(); // Allows peeking ahead in the iterator.

    while char_iter.peek().is_some() {
        let mut chunk: Vec<u8> = Vec::new();

        // Read up to 4 characters from the input string.
        for _ in 0..4 {
            if let Some(&current_char) = char_iter.peek() {
                let char_index = current_char as usize;

                // Validate that the character exists in our lookup table.
                if 128 <= char_index || lookup_table[char_index] == 255 {
                    return Err(format!("Invalid character found: {}", current_char));
                }

                // Convert the Base64 character to its 6-bit value.
                chunk.push(lookup_table[char_index]);
                char_iter.next();
            } else {
                break;
            }
        }

        // Convert the 6-bit chunks into 8-bit bytes.
        match chunk.len() {
            4 => {
                let first_byte: u8 = (chunk[0] << 2) | (chunk[1] >> 4);
                let second_byte: u8 = ((chunk[1] & 0x0F) << 4) | (chunk[2] >> 2);
                let third_byte: u8 = ((chunk[2] & 0x03) << 6) | chunk[3];

                decoded_bytes.push(first_byte);
                decoded_bytes.push(second_byte);
                decoded_bytes.push(third_byte);
            }
            3 => {
                let first_byte: u8 = (chunk[0] << 2) | (chunk[1] >> 4);
                let second_byte: u8 = ((chunk[1] & 0x0F) << 4) | (chunk[2] >> 2);

                decoded_bytes.push(first_byte);
                decoded_bytes.push(second_byte);
            }
            2 => {
                let first_byte: u8 = (chunk[0] << 2) | (chunk[1] >> 4);
                decoded_bytes.push(first_byte);
            }
            _ => return Err("Invalid Base64 group".to_string()),
        }
    }

    return Ok(decoded_bytes);
}

#[cfg(test)]
mod tests {
    use super::decode_base64;

    #[test]
    fn test_base64_decode_valid() {
        let encoded: &str = "ieysGjGO8papd/a";
        let decoded: Vec<u8> = decode_base64(encoded).expect("Failed to decode Base64.");
        let expected = vec![
            0b00100000, 0b01000110, 0b00010010, 0b10000000, 0b10011000, 0b00101000, 0b11110000,
            0b11110000, 0b00001111, 0b00001111, 0b11110000,
        ];
        assert_eq!(decoded, expected);
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
