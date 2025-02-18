use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum RegistrationError {
    AlreadyRegistered,
    InvalidName,
    InvalidRegistrationToken,
    TooManyPlayers,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RelativeDirection {
    Front,
    Right,
    Back,
    Left,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterTeam {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RegisterTeamResult {
    Ok {
        expected_players: u8,
        registration_token: String,
    },
    Err(RegistrationError),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubscribePlayer {
    pub name: String,
    pub registration_token: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SubscribePlayerResult {
    Ok,
    Err(RegistrationError),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Action {
    MoveTo(RelativeDirection),
    SolveChallenge { answer: String },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ActionError {
    CannotPassThroughWall,
    CannotPassThroughOpponent,
    NoRunningChallenge,
    SolveChallengeFirst,
    InvalidChallengeSolution,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Hint {
    RelativeCompass { angle: f32 },
    GridSize { columns: u32, rows: u32 },
    Secret(u64),
    SOSHelper,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Challenge {
    SecretSumModulo(u64),
    SOS,
}

#[derive(Serialize, Deserialize, Debug)]
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
