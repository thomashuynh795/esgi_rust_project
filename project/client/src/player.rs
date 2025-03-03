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
    pub players_finished: usize,
}

pub struct Player {
    pub name: String,
    pub stream: TcpStream,
    pub map: Map,
    pub cardinal_direction: CardinalDirection,
    pub move_count: usize,
    pub pending_challenge: Option<Challenge>,
    pub secrets: Arc<Mutex<HashMap<String, u64>>>,
    pub global_challenge: Arc<Mutex<Option<Challenge>>>,
}

impl Player {
    pub fn new(
        name: String,
        stream: TcpStream,
        encoded_radar: String,
        secrets: Arc<Mutex<HashMap<String, u64>>>,
        global_challenge: Arc<Mutex<Option<Challenge>>>,
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
            global_challenge,
        }
    }

    pub fn calculate_secret_sum(&self, modulo: u64) -> u64 {
        let shared_secrets: std::sync::MutexGuard<'_, HashMap<String, u64>> =
            self.secrets.lock().unwrap();
        let sum128: u128 = shared_secrets
            .values()
            .fold(0u128, |accumulator: u128, &secret| {
                accumulator + (secret as u128)
            });
        (sum128 % (modulo as u128)) as u64
    }

    pub fn solve_global_challenge(&mut self, total_players: usize) -> io::Result<()> {
        let maybe_challenge = self.global_challenge.lock().unwrap().clone();
        match maybe_challenge {
            Some(Challenge::SecretSumModulo(modulo)) => {
                let answer = self.calculate_secret_sum(modulo);
                log_info!(
                    "{}: SolveChallenge => SecretSumModulo({}), computed answer = {}",
                    self.name,
                    modulo,
                    answer
                );

                let solve_msg = GameMessage::Action(Action::SolveChallenge {
                    answer: answer.to_string(),
                });
                solve_msg.send(&mut self.stream)?;

                let response = GameMessage::receive(&mut self.stream)?;
                log_info!(
                    "{}: Received response after solving challenge: {:?}",
                    self.name,
                    response
                );

                if let GameMessage::ActionError(ActionError::SolveChallengeFirst) = response {
                    log_warning!(
                        "{}: Challenge response rejected => re-calculate with updated secrets",
                        self.name
                    );
                    self.solve_global_challenge(total_players)?;
                } else {
                    log_info!(
                        "{}: Challenge solved successfully => clearing global challenge",
                        self.name
                    );
                    let mut global_challenge: std::sync::MutexGuard<'_, Option<Challenge>> =
                        self.global_challenge.lock().unwrap();
                    *global_challenge = None;
                }
            }
            Some(_) => {
                log_warning!(
                    "{}: A global challenge is set but is not SecretSumModulo",
                    self.name
                );
            }
            None => {
                log_warning!(
                    "{}: solve_global_challenge() called but no global challenge is set",
                    self.name
                );
            }
        }
        Ok(())
    }

    pub fn play(
        mut self,
        player_id: usize,
        turn_state: Arc<(Mutex<TurnState>, Condvar)>,
        total_players: usize,
    ) -> io::Result<()> {
        let has_finished: bool = false;
        loop {
            let (lock, cvar) = &*turn_state;

            // Safely unwraps the MutexGuard.
            let mut state: std::sync::MutexGuard<'_, TurnState> = match lock.lock() {
                Ok(state) => state,
                Err(poisoned) => {
                    log_warning!(
                        "{}: Mutex is poisoned. The data you are using may be corrupted",
                        self.name
                    );
                    poisoned.into_inner()
                }
            };

            while state.current != player_id && !state.game_over {
                state = cvar.wait(state).unwrap();
            }
            if state.game_over {
                break;
            }
            drop(state);

            if has_finished {
                log_info!(
                    "{} has already finished and is waiting for others.",
                    self.name
                );
            } else {
                let gc_opt: Option<Challenge> = self.global_challenge.lock().unwrap().clone();
                if gc_opt.is_some() {
                    log_warning!("{} sees a global challenge => try to solve it", self.name);
                    self.solve_global_challenge(total_players)?;
                } else {
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
                                GameMessage::Challenge(Challenge::SecretSumModulo(m)) => {
                                    log_info!(
                                        "{}: Received a global SecretSumModulo: {}",
                                        self.name,
                                        m
                                    );
                                    let mut global_challenge: std::sync::MutexGuard<
                                        '_,
                                        Option<Challenge>,
                                    > = self.global_challenge.lock().unwrap();
                                    *global_challenge = Some(Challenge::SecretSumModulo(m));
                                    drop(global_challenge);

                                    self.solve_global_challenge(total_players)?;
                                }
                                GameMessage::RadarView(new_radar_data) => {
                                    let new_radar: RadarView =
                                        RadarView::new(new_radar_data, chosen_cardinal_direction);
                                    self.cardinal_direction = chosen_cardinal_direction;
                                    self.map.merge_radar_view(
                                        &new_radar.grid,
                                        chosen_cardinal_direction,
                                    );
                                }
                                GameMessage::ActionError(err) => match err {
                                    ActionError::SolveChallengeFirst => {
                                        log_warning!(
                                            "{}: The server requires to solve a challenge first",
                                            self.name
                                        );
                                        let mut global_challenge: std::sync::MutexGuard<
                                            '_,
                                            Option<Challenge>,
                                        > = self.global_challenge.lock().unwrap();
                                        if global_challenge.is_none() {
                                            log_warning!("{} has called try_solve_challenge() but no global challenge is set", self.name);
                                        } else {
                                            drop(global_challenge);
                                            self.solve_global_challenge(total_players)?;
                                        }
                                    }
                                    ActionError::InvalidChallengeSolution => {}
                                    _ => {
                                        log_warning!(
                                            "{} has performed a bad action: {:?}",
                                            self.name,
                                            err
                                        );
                                    }
                                },
                                GameMessage::Hint(Hint::Secret(value)) => {
                                    log_info!(
                                        "{} has received a secret from a hint: {}",
                                        self.name,
                                        value
                                    );
                                    let mut shared_secrets: std::sync::MutexGuard<
                                        '_,
                                        HashMap<String, u64>,
                                    > = self.secrets.lock().unwrap();
                                    shared_secrets.insert(self.name.clone(), value);
                                }
                                _ => {
                                    log_warning!(
                                        "{} has received an unexpected message",
                                        self.name
                                    );
                                }
                            }

                            let (lock, cvar) = &*turn_state;
                            let mut state: std::sync::MutexGuard<'_, TurnState> =
                                lock.lock().unwrap();
                            state.current = (state.current + 1) % total_players;
                            cvar.notify_all();
                        }
                        None => {
                            log_info!("{} has no more moves available, game over", self.name);
                            let (lock, cvar) = &*turn_state;
                            let mut state: std::sync::MutexGuard<'_, TurnState> =
                                lock.lock().unwrap();
                            state.game_over = true;
                            cvar.notify_all();
                            break;
                        }
                    }
                }
            }
            thread::sleep(Duration::from_millis(200));
        }

        return Ok(());
    }
}
