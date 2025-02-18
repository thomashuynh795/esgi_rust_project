use crate::messages::GameMessage;
use std::io::{Read, Write};
use std::net::TcpStream;

impl GameMessage {
    /// Serializes the `GameMessage` into JSON format,
    /// writes its size as a 4-byte little-endian integer to the TCP stream,
    /// writes the serialized message and sends it through a TCP stream.
    /// # Arguments
    /// * `stream` - A mutable reference to the `TcpStream` used for sending the message.
    /// # Returns
    /// Returns `Ok(())` if the entire message is successfully sent, otherwise returns an `std::io::Error`.
    /// # Errors
    /// Returns an error if:
    /// * The message serialization fails.
    /// * Writing to the TCP stream fails.
    pub fn send(&self, stream: &mut TcpStream) -> std::io::Result<()> {
        // Serializes the GameMessage to a JSON byte array.
        let message: Vec<u8> = serde_json::to_vec(self)?;

        // Gets the size of the serialized message with a safe way.
        // If the message is too large, it will return an error.
        // The method try_from throws an error that is not of type std::io::Error so we need to convert it.
        let message_size_buffer: u32 = u32::try_from(message.len())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        // Converts the message length to 4 little-endian bytes because the size of the message is 4 bytes.
        let size: [u8; 4] = message_size_buffer.to_le_bytes();

        stream.write_all(&size)?; // Writes the size to the TCP stream.
        stream.write_all(&message)?; // Writes the serialized message to the TCP stream.

        return Ok(()); // Returns success with void tuple.
    }

    /// Receives a message from a TCP stream, deserializes it and returns it.
    /// # Arguments
    /// * `stream` - A mutable reference to a `TcpStream` used to collect the message.
    /// # Returns
    /// A `std::io::Result` containing the received `GameMessage`.
    /// # Errors
    /// Returns an error if:
    /// * Reading the TCP stream fails.
    /// * The message deserialization fails.
    pub fn receive(stream: &mut TcpStream) -> std::io::Result<Self> {
        // The size is an u32, so it is 4 bytes long so creates an array of 4 bytes with value 0.
        let mut message_size_buffer: [u8; 4] = [0u8; 4];

        // From the stream, receives the size of the message as an array of 4 bytes and
        // updates the message_size_buffer with the received bytes.
        stream.read_exact(&mut message_size_buffer)?;

        // Converts safely the message size buffer to a u32 and then to a usize.
        let size: usize = usize::try_from(u32::from_le_bytes(message_size_buffer))
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        // Creates a vector of `size` bytes.
        let mut buf: Vec<u8> = vec![0u8; size];

        // Reads `size` bytes from the stream.
        stream.read_exact(&mut buf)?;

        // Deserializes the bytes into a `GameMessage`.
        let message: GameMessage = serde_json::from_slice(&buf)?;

        return Ok(message); // Returns the `GameMessage`.
    }
}
