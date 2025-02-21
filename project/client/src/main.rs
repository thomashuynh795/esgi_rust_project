#[macro_use]
extern crate shared;

use grid::maze::{choose_next_move, send_and_receive, MazeState};
use shared::{
    types::action::RelativeDirection,
    utils::{connect_to_server, register_player, register_team},
};
use std::env;
use std::io;

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

    // Connects to the server and registers the team.
    let mut stream: std::net::TcpStream = connect_to_server(server_address)?;
    let registration_token: String = register_team(&mut stream)?;

    // Connection to register the player.
    let mut stream: std::net::TcpStream = connect_to_server(server_address)?;
    register_player(&mut stream, &registration_token)?;

    // Explore the maze using the Tremaux algorithm.
    let mut maze: MazeState = MazeState::new(20, 20, RelativeDirection::Front);
    let max_moves: i32 = 100;

    for _ in 0..max_moves {
        if let Some(next_move) = choose_next_move(&mut maze) {
            log_info!("Next move (Tremaux): {:?}", next_move);
            send_and_receive(&mut stream, next_move, &mut maze)?;
        } else {
            log_warning!("No valid move found, stopping");
            break;
        }
    }

    log_info!("Exploration finished");

    return Ok(());
}
