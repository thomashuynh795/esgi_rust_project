#[macro_use]
extern crate shared;

use grid::map::Map;
use shared::utils::print_string_matrix;
use shared::utils::{connect_to_server, register_player, register_team};
use std::collections::HashMap;
use std::env;
use std::io;
use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, Mutex};

const PLAYERS_NUMBER: usize = 3;

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

    for i in 0..PLAYERS_NUMBER {
        let stream = match connect_to_server(server_address) {
            Ok(s) => Arc::new(Mutex::new(s)),
            Err(e) => {
                log_error!("Failed to connect player {}: {}", i + 1, e);
                return Err(e);
            }
        };

        let addr = match stream.lock().unwrap().peer_addr() {
            Ok(a) => a,
            Err(e) => {
                log_error!("Failed to get peer address for player {}: {}", i + 1, e);
                return Err(e);
            }
        };

        let player_name = format!("Player {}", i + 1);
        let encoded_radar = match register_player(
            &mut stream.lock().unwrap(),
            &registration_token,
            &player_name,
        ) {
            Ok(r) => r,
            Err(e) => {
                log_error!("Failed to register {}: {}", player_name, e);
                return Err(e);
            }
        };

        let player_map: Map = match Map::new(&encoded_radar);

        player_maps.insert(addr, (stream.clone(), player_map));

        if let Some((_, map)) = player_maps.get(&addr) {
            let matrix_name: String = format!("{}'s map", player_name);
            print_string_matrix(&matrix_name, &map.matrix);
        }
    }

    log_info!("All players registered");

    return Ok(());
}
