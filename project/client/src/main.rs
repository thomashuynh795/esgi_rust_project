#[macro_use]
extern crate shared;
pub mod player;
pub mod team;
use std::env;
use std::io;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

use player::TurnState;
use team::Team;

const PLAYERS_NUMBER: usize = 3;

fn main() -> io::Result<()> {
    // Enables backtrace in case of panic.
    env::set_var("RUST_BACKTRACE", "full");

    // Parses command line arguments.
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        log_error!("Usage: worker <server_address>");
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Server address required",
        ));
    }

    // Stores the server address.
    let server_address: &String = &args[1];

    // Registers the team.
    let mut team: Team = Team::register(server_address, &String::from("Team 1"))?;

    for i in 0..PLAYERS_NUMBER {
        let player_name: String = format!("Player {}", i + 1);
        team.add_player(&player_name, server_address)?;
    }

    let turn_state: Arc<(Mutex<TurnState>, Condvar)> = Arc::new((
        Mutex::new(TurnState {
            current: 0,
            game_over: false,
        }),
        Condvar::new(),
    ));

    let mut handles: Vec<thread::JoinHandle<Result<(), io::Error>>> =
        Vec::with_capacity(team.players.len());
    for (player_id, player) in team.players.into_iter().enumerate() {
        let turn_state: Arc<(Mutex<TurnState>, Condvar)> = Arc::clone(&turn_state);

        let handle: thread::JoinHandle<Result<(), io::Error>> =
            thread::spawn(move || -> io::Result<()> {
                player.play(player_id, turn_state, PLAYERS_NUMBER)?;
                Ok(())
            });
        handles.push(handle);
    }

    for handle in handles {
        if let Err(e) = handle.join() {
            log_error!("A thread has panicked: {:?}", e);
        }
    }

    return Ok(());
}
