use crate::types::error::RegistrationError;
use crate::types::{action::Action, challenge::Challenge, error::ActionError, hint::Hint};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterTeam {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RegisterTeamResult {
    Ok {
        expected_players: u8,
        registration_token: String,
    },
    Err(RegistrationError),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubscribePlayer {
    pub name: String,
    pub registration_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SubscribePlayerResult {
    Ok,
    Err(RegistrationError),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum GameMessage {
    RegisterTeam(RegisterTeam),
    RegisterTeamResult(RegisterTeamResult),
    SubscribePlayer(SubscribePlayer),
    SubscribePlayerResult(SubscribePlayerResult),
    RadarView(String),
    Hint(Hint),
    Action(Action),
    ActionError(ActionError),
    Challenge(Challenge),
}
