use std::collections::HashMap;
use std::io;
use std::net::TcpStream;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

use grid::map::Map;
use grid::radar::RadarView;
use shared::types::action::Action;
use shared::types::cardinal_direction::CardinalDirection;
use shared::types::challenge::Challenge;
use shared::types::error::ActionError;
use shared::types::hint::Hint;
use shared::types::message::GameMessage;

pub struct TurnState {
    pub current: usize,
    pub game_over: bool,
}

pub struct Player {
    pub name: String,
    pub stream: TcpStream,
    pub map: Map,
    pub cardinal_direction: CardinalDirection,
    pub move_count: usize,
    pub pending_challenge: Option<Challenge>,
    pub secrets: Arc<Mutex<HashMap<String, u64>>>,
}

impl Player {
    pub fn new(
        name: String,
        stream: TcpStream,
        encoded_radar: String,
        secrets: Arc<Mutex<HashMap<String, u64>>>,
    ) -> Self {
        let initial_radar: RadarView = RadarView::new(encoded_radar, CardinalDirection::North);
        let map: Map = Map::new(&initial_radar.grid, initial_radar.cardinal_direction);

        Self {
            name,
            stream,
            map,
            cardinal_direction: initial_radar.cardinal_direction,
            move_count: 0,
            pending_challenge: None,
            secrets,
        }
    }

    pub fn try_solve_challenge(&mut self) -> io::Result<()> {
        if let Some(ch) = &self.pending_challenge {
            match ch {
                Challenge::SecretSumModulo(modulo) => {
                    let shared_secrets: std::sync::MutexGuard<'_, HashMap<String, u64>> =
                        self.secrets.lock().unwrap();

                    let mut sum128: u128 = 0;
                    for &secret_val in shared_secrets.values() {
                        sum128 = (sum128 + (secret_val as u128)) % (*modulo as u128);
                    }

                    let answer: u64 = sum128 as u64;

                    log_info!(
                        "{}: SolveChallenge => SecretSumModulo({}), sum={}, answer={}",
                        self.name,
                        modulo,
                        sum128,
                        answer
                    );

                    let solve: GameMessage = GameMessage::Action(Action::SolveChallenge {
                        answer: answer.to_string(),
                    });

                    solve.send(&mut self.stream)?;
                }
                _ => {
                    log_warning!(
                        "{}: Challenge inconnu, pas de solution implémentée.",
                        self.name
                    );
                }
            }
        } else {
            log_warning!(
                "{} has called try_solve_challenge() but there is no pending challenge",
                self.name
            );
        }
        return Ok(());
    }

    pub fn play(
        mut self,
        player_id: usize,
        turn_state: Arc<(Mutex<TurnState>, Condvar)>,
        total_players: usize,
    ) -> io::Result<()> {
        loop {
            let (lock, conditional_variable) = &*turn_state;
            let mut state: std::sync::MutexGuard<'_, TurnState> = lock.lock().unwrap();
            while state.current != player_id && !state.game_over {
                state = conditional_variable.wait(state).unwrap();
            }
            if state.game_over {
                break;
            }
            drop(state);

            match self.map.next_move_tremaux() {
                Some((relative_direction, chosen_cardinal_direction)) => {
                    self.move_count += 1;

                    let action: GameMessage =
                        GameMessage::Action(Action::MoveTo(relative_direction));
                    action.send(&mut self.stream)?;
                    log_info!("{} has sent a move: {}", self.name, self.move_count);

                    let response: GameMessage = GameMessage::receive(&mut self.stream)?;
                    log_info!("{} has received a response: {:?}", self.name, response);

                    match response {
                        GameMessage::RadarView(new_radar_data) => {
                            let new_radar: RadarView =
                                RadarView::new(new_radar_data, chosen_cardinal_direction);
                            self.cardinal_direction = chosen_cardinal_direction;
                            self.map
                                .merge_radar_view(&new_radar.grid, chosen_cardinal_direction);
                        }
                        GameMessage::ActionError(err) => match err {
                            ActionError::SolveChallengeFirst => {
                                log_warning!(
                                    "{}: The server requires to solve a challenge first",
                                    self.name
                                );
                                self.try_solve_challenge()?;
                            }
                            _ => {
                                log_warning!("{} has performed a bad action: {:?}", self.name, err);
                            }
                        },
                        GameMessage::Hint(Hint::Secret(value)) => {
                            log_info!("{} has received a secret from a hint: {}", self.name, value);
                            let mut shared_secrets: std::sync::MutexGuard<
                                '_,
                                HashMap<String, u64>,
                            > = self.secrets.lock().unwrap();
                            shared_secrets.insert(self.name.clone(), value);
                        }
                        GameMessage::Challenge(Challenge::SecretSumModulo(modulo)) => {
                            log_info!(
                                "{} has received a SecretSumModulo challenge: {}",
                                self.name,
                                modulo
                            );

                            self.pending_challenge = Some(Challenge::SecretSumModulo(modulo));

                            self.try_solve_challenge()?;
                        }
                        _ => {
                            log_warning!("{} has received an unexpected message", self.name);
                        }
                    }

                    let (lock, cvar) = &*turn_state;
                    let mut state: std::sync::MutexGuard<'_, TurnState> = lock.lock().unwrap();
                    state.current = (state.current + 1) % total_players;
                    cvar.notify_all();
                }
                None => {
                    log_info!("{} has no more moves available, game over", self.name);
                    let (lock, cvar) = &*turn_state;
                    let mut state: std::sync::MutexGuard<'_, TurnState> = lock.lock().unwrap();
                    state.game_over = true;
                    cvar.notify_all();
                    break;
                }
            }

            thread::sleep(Duration::from_millis(200));
        }

        return Ok(());
    }
}
