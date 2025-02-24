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
pub fn decode_base64(s: &str) -> Result<Vec<u8>, String> {
    const ALPHABET: &[u8; 64] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789+/";
    // Construction d'une table de correspondance pour retrouver la valeur associée à chaque caractère.
    let mut rev_table = [255u8; 128];
    for (i, &byte) in ALPHABET.iter().enumerate() {
        rev_table[byte as usize] = i as u8;
    }

    // Vérification : seule une taille de la forme 4n+1 est invalide.
    if s.len() % 4 == 1 {
        return Err("Taille invalide pour un encodage base64".to_string());
    }

    let mut output = Vec::new();
    let mut chars = s.chars().peekable();

    while chars.peek().is_some() {
        let mut group = Vec::new();
        for _ in 0..4 {
            if let Some(c) = chars.peek() {
                let c_val = *c as usize;
                if c_val >= 128 || rev_table[c_val] == 255 {
                    return Err(format!("Caractère non autorisé: {}", c));
                }
                group.push(rev_table[c_val]);
                chars.next();
            } else {
                break;
            }
        }

        match group.len() {
            4 => {
                let byte1 = (group[0] << 2) | (group[1] >> 4);
                let byte2 = ((group[1] & 0x0F) << 4) | (group[2] >> 2);
                let byte3 = ((group[2] & 0x03) << 6) | group[3];
                output.push(byte1);
                output.push(byte2);
                output.push(byte3);
            }
            3 => {
                let byte1 = (group[0] << 2) | (group[1] >> 4);
                let byte2 = ((group[1] & 0x0F) << 4) | (group[2] >> 2);
                output.push(byte1);
                output.push(byte2);
            }
            2 => {
                let byte1 = (group[0] << 2) | (group[1] >> 4);
                output.push(byte1);
            }
            _ => return Err("Groupe de caractères invalide".to_string()),
        }
    }
    Ok(output)
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
