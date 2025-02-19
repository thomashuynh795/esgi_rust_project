use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum RegistrationError {
    AlreadyRegistered,
    InvalidName,
    InvalidRegistrationToken,
    TooManyPlayers,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ActionError {
    CannotPassThroughWall,
    CannotPassThroughOpponent,
    NoRunningChallenge,
    SolveChallengeFirst,
    InvalidChallengeSolution,
}
