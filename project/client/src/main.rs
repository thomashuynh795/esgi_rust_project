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

    // Creates a shared turn state protected by a mutex and a conditional variable.
    let turn_state: Arc<(Mutex<TurnState>, Condvar)> = Arc::new((
        Mutex::new(TurnState {
            current: 0,
            game_over: false,
        }),
        Condvar::new(),
    ));

    // Creates an empty vector to store the players threads.
    let mut threads: Vec<thread::JoinHandle<Result<(), io::Error>>> =
        Vec::with_capacity(team.players.len());
    // Fills the vector with the players threads.
    for (player_id, player) in team.players.into_iter().enumerate() {
        // Clones the shared turn state to give a reference to each player. The reference is moved to the player thread to allow each player to access the shared state.
        // Clone here is not a traditional clone, it is a reference count incrementation to allow multiple ownership of the same data instead of copying it.
        let turn_state: Arc<(Mutex<TurnState>, Condvar)> = Arc::clone(&turn_state);

        // Creates a thread for each player.
        let thread: thread::JoinHandle<Result<(), io::Error>> =
        // The `move` keyword is used to move ownership of the variables to the thread. This is necessary because the thread may outlive the current scope.
            thread::spawn(move || -> io::Result<()> {
                player.play(player_id, turn_state, PLAYERS_NUMBER)?;
                return Ok(());
            });
        // Adds the thread to the vector.
        threads.push(thread);
    }

    for thread in threads {
        if let Err(e) = thread.join() {
            log_error!("A thread has panicked: {:?}", e);

            /*===========================================
                TEST MINI SERVER PART
            ===========================================*/

            // let encoded_radar: String = register_player(
            //     &mut stream.lock().unwrap(),
            //     &registration_token,
            //     &player_name,
            // )?;
            // print!("Received radar view: {}", encoded_radar);
            // let radar_view_1: RadarView = RadarView::new(encoded_radar, CardinalDirection::North);

            // log_debug!("Cardinal direction: {:?}", radar_view_1.cardinal_direction);
            // print_string_matrix("Radar view 1", &radar_view_1.grid);
            // let mut map: Map = Map::new(&radar_view_1.grid, radar_view_1.cardinal_direction);

            // log_debug!(
            //     "Cardinal direction of the map: {:?}",
            //     map.current_cardinal_direction
            // );
            // print_string_matrix("Updated map", &map.grid.as_ref());

            // let mut i: i32 = 0;
            // for _ in 0..8 {
            //     match map.next_move_tremaux() {
            //         Some((relative_direction, chosen_cardinal_direction)) => {
            //             i += 1;
            //             log_info!("ITERATION: {}", i);
            //             log_info!("Next move to send: {:?}", relative_direction);

            //             let action: GameMessage = GameMessage::Action(Action::MoveTo(relative_direction));
            //             let mut action_sent = false;
            //             while !action_sent {
            //                 {
            //                     let mut stream_lock: std::sync::MutexGuard<'_, TcpStream> =
            //                         stream.lock().unwrap();
            //                     match action.send(&mut stream_lock) {
            //                         Ok(_) => {
            //                             log_info!("Action sent.");
            //                             action_sent = true;
            //                             break;
            //                         }
            //                         Err(e) if e.kind() == io::ErrorKind::BrokenPipe => {
            //                             log_warning!("Broken pipe error, attempting to reconnect...");
            //                             *stream_lock = connect_to_server(server_address)?;
            //                         }
            //                         Err(e) => return Err(e),
            //                     }
            //                 }
            //                 thread::sleep(Duration::from_millis(10));
            //             }
            //             if !action_sent {
            //                 return Err(io::Error::new(
            //                     io::ErrorKind::BrokenPipe,
            //                     "Failed to send action after multiple attempts",
            //                 ));
            //             }

            //             let response: GameMessage;
            //             {
            //                 let mut stream_lock: std::sync::MutexGuard<'_, TcpStream> =
            //                     stream.lock().unwrap();
            //                 response = GameMessage::receive(&mut stream_lock)?;
            //                 log_info!("Response received.");
            //             }
            //             match response {
            //                 GameMessage::RadarView(new_radar_data) => {
            //                     let new_radar_view: RadarView =
            //                         RadarView::new(new_radar_data, chosen_cardinal_direction);
            //                     let radar_view_log = format!("Radar view {}", i + 1);
            //                     print_string_matrix(&radar_view_log, &new_radar_view.grid);
            //                     map.merge_radar_view(&new_radar_view.grid, chosen_cardinal_direction);
            //                     print_string_matrix("Updated map", &map.grid);
            //                 }
            //                 GameMessage::ActionError(err) => {
            //                     log_warning!("Action error received: {:?}", err);
            //                 }
            //                 _ => {
            //                     log_warning!("Unexpected message received.");
            //                 }
            //             }
            //             thread::sleep(Duration::from_millis(10));
            //         }
            //         None => {
            //             log_info!("No more possible moves. Stopping exploration.\n");
            //             break;
            //         }
        }
    }

    return Ok(());
}
