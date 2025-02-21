use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum RelativeDirection {
    Up,
    Right,
    Down,
    Left,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Action {
    MoveTo(RelativeDirection),
    SolveChallenge { answer: String },
}
