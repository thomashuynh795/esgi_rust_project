use shared::messages::{Action, GameMessage, RegisterTeam, RegisterTeamResult, RelativeDirection};
use std::net::TcpStream;

fn main() -> std::io::Result<()> {
    let mut stream: TcpStream = TcpStream::connect("localhost:8778")?;

    let register_team: RegisterTeam = RegisterTeam {
        name: String::from("team_1"),
    };

    let message: GameMessage = GameMessage::RegisterTeam(register_team);
    message.send(&mut stream)?;
    println!("Message sent to the server.");

    if let GameMessage::RegisterTeamResult(result) = GameMessage::receive(&mut stream)? {
        match result {
            RegisterTeamResult::Ok {
                expected_players,
                registration_token,
            } => {
                println!(
                    "Team registered successfully. Expected players: {}, Registration token: {}",
                    expected_players, registration_token
                );
            }
            RegisterTeamResult::Err(e) => panic!("Failed to register: {:?}", e),
        }
    }

    let moves: Vec<RelativeDirection> = vec![
        RelativeDirection::Right,
        RelativeDirection::Forward,
        RelativeDirection::Left,
        RelativeDirection::Backward,
    ];

    for direction in moves {
        let action: GameMessage = GameMessage::Action(Action::MoveTo(direction));
        action.send(&mut stream)?;
        println!("Move sent: {:?}", direction);
    }

    return Ok(());
}
