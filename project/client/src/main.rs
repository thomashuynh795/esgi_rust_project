use std::env;
use std::io::{self};
use std::net::TcpStream;

use shared::types::{
    action::{Action, RelativeDirection},
    message::{GameMessage, RegisterTeam, RegisterTeamResult},
};

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Prompt: worker <server_address>");
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Server address required.",
        ));
    }
    let server_address = &args[1];

    println!("Connection to the address {}.", server_address);

    let mut stream: TcpStream = match TcpStream::connect(server_address) {
        Ok(s) => {
            println!("Connected to the server.");
            s
        }
        Err(e) => {
            eprintln!("Connection error: {}.", e);
            return Err(e);
        }
    };

    let register_team: RegisterTeam = RegisterTeam {
        name: String::from("team_1"),
    };
    let message: GameMessage = GameMessage::RegisterTeam(register_team);
    message.send(&mut stream)?;
    println!("Registration message sent to the server.");

    match GameMessage::receive(&mut stream)? {
        GameMessage::RegisterTeamResult(RegisterTeamResult::Ok {
            expected_players,
            registration_token,
        }) => {
            println!(
                "Team registered. Waiting for players: {}. Registering token: {}.",
                expected_players, registration_token
            );
        }
        GameMessage::RegisterTeamResult(RegisterTeamResult::Err(e)) => {
            eprintln!("Registration failed: {:?}.", e);
            return Err(io::Error::new(io::ErrorKind::Other, "Registration failed."));
        }
        _ => {
            eprintln!("Unexpected server response.");
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Unexpected server response.",
            ));
        }
    }

    let moves: Vec<RelativeDirection> = vec![
        RelativeDirection::Right,
        RelativeDirection::Up,
        RelativeDirection::Left,
        RelativeDirection::Down,
    ];

    for direction in moves {
        let action: GameMessage = GameMessage::Action(Action::MoveTo(direction));
        action.send(&mut stream)?;
        println!("Movement sent: {:?}.", direction);
    }

    println!("Movements sent.");

    return Ok(());
}
