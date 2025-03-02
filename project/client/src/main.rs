#[macro_use]
extern crate shared;

use grid::map::Map;
use grid::radar::RadarView;
use shared::types::action::{Action, RelativeDirection};
use shared::types::cardinal_direction::CardinalDirection;
use shared::types::log;
use shared::types::message::GameMessage;
use shared::utils::{connect_to_server, print_string_matrix, register_player, register_team};
use std::collections::HashMap;
use std::env;
use std::io;
use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const PLAYERS_NUMBER: usize = 1;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        log_error!("Prompt: worker <server_address>");
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Server address required",
        ));
    }
    let server_address: &String = &args[1];

    let mut registering_team_stream: TcpStream = connect_to_server(server_address)?;
    let registration_token: String = register_team(&mut registering_team_stream)?;

    let mut player_maps: HashMap<SocketAddr, (Arc<Mutex<TcpStream>>, Map)> = HashMap::new();

    let stream: Arc<Mutex<TcpStream>> = Arc::new(Mutex::new(connect_to_server(server_address)?));

    let addr: SocketAddr = stream.lock().unwrap().peer_addr()?;
    let player_name: String = "Player 1".to_string();

    let encoded_radar: String = register_player(
        &mut stream.lock().unwrap(),
        &registration_token,
        &player_name,
    )?;
    print!("Received radar view: {}", encoded_radar);
    let radar_view_1: RadarView = RadarView::new(encoded_radar, CardinalDirection::North);

    log_debug!("Cardinal direction: {:?}", radar_view_1.cardinal_direction);
    print_string_matrix("Radar view 1", &radar_view_1.grid);
    let mut map: Map = Map::new(&radar_view_1.grid, radar_view_1.cardinal_direction);

    log_debug!(
        "Cardinal direction of the map: {:?}",
        map.current_cardinal_direction
    );
    print_string_matrix("Updated map", &map.grid.as_ref());

    let mut i: i32 = 0;
    for _ in 0..8 {
        match map.next_move_tremaux() {
            Some((relative_direction, chosen_cardinal_direction)) => {
                i += 1;
                log_info!("ITERATION: {}", i);
                log_info!("Next move to send: {:?}", relative_direction);

                let action: GameMessage = GameMessage::Action(Action::MoveTo(relative_direction));
                let mut action_sent = false;
                while !action_sent {
                    {
                        let mut stream_lock: std::sync::MutexGuard<'_, TcpStream> =
                            stream.lock().unwrap();
                        match action.send(&mut stream_lock) {
                            Ok(_) => {
                                log_info!("Action sent.");
                                action_sent = true;
                                break;
                            }
                            Err(e) if e.kind() == io::ErrorKind::BrokenPipe => {
                                log_warning!("Broken pipe error, attempting to reconnect...");
                                *stream_lock = connect_to_server(server_address)?;
                            }
                            Err(e) => return Err(e),
                        }
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                if !action_sent {
                    return Err(io::Error::new(
                        io::ErrorKind::BrokenPipe,
                        "Failed to send action after multiple attempts",
                    ));
                }

                let response: GameMessage;
                {
                    let mut stream_lock: std::sync::MutexGuard<'_, TcpStream> =
                        stream.lock().unwrap();
                    response = GameMessage::receive(&mut stream_lock)?;
                    log_info!("Response received.");
                }
                match response {
                    GameMessage::RadarView(new_radar_data) => {
                        let new_radar_view: RadarView =
                            RadarView::new(new_radar_data, chosen_cardinal_direction);
                        let radar_view_log = format!("Radar view {}", i + 1);
                        print_string_matrix(&radar_view_log, &new_radar_view.grid);
                        map.merge_radar_view(&new_radar_view.grid, chosen_cardinal_direction);
                        print_string_matrix("Updated map", &map.grid);
                    }
                    GameMessage::ActionError(err) => {
                        log_warning!("Action error received: {:?}", err);
                    }
                    _ => {
                        log_warning!("Unexpected message received.");
                    }
                }
                thread::sleep(Duration::from_millis(10));
            }
            None => {
                log_info!("No more possible moves. Stopping exploration.\n");
                break;
            }
        }
    }

    return Ok(());
}
