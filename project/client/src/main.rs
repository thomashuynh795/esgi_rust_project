use shared::messages::{GameMessage, RegisterTeam};
use std::net::TcpStream;

fn main() -> std::io::Result<()> {
    let mut stream: TcpStream = TcpStream::connect("localhost:8778")?;

    let register_team: RegisterTeam = RegisterTeam {
        name: String::from("team_1"),
    };

    let message: GameMessage = GameMessage::RegisterTeam(register_team);

    message.send(&mut stream)?;
    println!("Message sent to the server.");

    let response: GameMessage = GameMessage::receive(&mut stream)?;
    println!("Server response: {:?}", response);

    return Ok(());
}
