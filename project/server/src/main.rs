use shared::types::message::{GameMessage, RegisterTeamResult, SubscribePlayerResult};
use shared::{log_debug, log_error, log_info};
use std::io::Error as IoError;
use std::net::{TcpListener, TcpStream};
use uuid::Uuid;

fn main() {
    log_debug!("Server is running on localhost:8778");
    let listener = TcpListener::bind("127.0.0.1:8778").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => match handle_connection(&mut stream) {
                Ok(_) => {
                    log_debug!("Connection closed: {:?}", stream.peer_addr());
                }
                Err(e) => {
                    log_error!("ERROR Failed to handle connection: {:?}", e);
                }
            },
            Err(e) => {
                log_error!("ERROR Failed to establish connection: {:?}", e);
            }
        }
    }
}

fn handle_connection(stream: &mut TcpStream) -> Result<(), IoError> {
    log_info!("New connection: {:?}", stream.peer_addr());
    match GameMessage::receive(stream) {
        Ok(GameMessage::RegisterTeam(register_team)) => {
            log_info!("Registering team: {:?}", register_team);
            let register_team_result = RegisterTeamResult::Ok {
                expected_players: 1,
                registration_token: Uuid::new_v4().to_string(),
            };
            let response = GameMessage::RegisterTeamResult(register_team_result);
            response.send(stream)?;
        }
        Ok(GameMessage::SubscribePlayer(subscribe_player)) => {
            log_info!("Subscribing player: {:?}", subscribe_player);
            let subscribe_player_result = SubscribePlayerResult::Ok;
            let response = GameMessage::SubscribePlayerResult(subscribe_player_result);
            response.send(stream)?;
            let radar_view = GameMessage::RadarView("jiucAjGa//cpapa".to_string());
            log_info!("Sending radar view: {:?}", radar_view);
            radar_view.send(stream)?;
        }
        Ok(GameMessage::Action(action)) => {
            log_info!("Action received: {:?}", action);
            let radar_view = GameMessage::RadarView("jiucAjGa//cpapa".to_string());
            log_info!("Sending radar view: {:?}", radar_view);
            radar_view.send(stream)?;
        }
        _ => {
            log_error!("Invalid request: {:?}", stream.peer_addr());
        }
    }
    return Ok(());
}

#[test]
fn test_register_team() {
    // do a server in a separate thread then call it
    std::thread::spawn(|| {
        main();
    });
    std::thread::sleep(std::time::Duration::from_millis(100));
    let mut stream = TcpStream::connect("127.0.0.1:8778").unwrap();
    let register_team = GameMessage::RegisterTeam(shared::types::message::RegisterTeam {
        name: "team_1".to_string(),
    });
    register_team.send(&mut stream).unwrap();
    let response = GameMessage::receive(&mut stream).unwrap();
    match response {
        GameMessage::RegisterTeamResult(register_team_result) => match register_team_result {
            RegisterTeamResult::Ok {
                expected_players,
                registration_token,
            } => {
                assert_eq!(expected_players, 1);
                assert_eq!(registration_token.len(), 36);
            }
            _ => panic!("Invalid response"),
        },
        _ => panic!("Invalid response"),
    }
}
