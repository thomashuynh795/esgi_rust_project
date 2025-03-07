use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum RelativeDirection {
    Front,
    Right,
    Back,
    Left,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Action {
    MoveTo(RelativeDirection),
    SolveChallenge { answer: String },
}
