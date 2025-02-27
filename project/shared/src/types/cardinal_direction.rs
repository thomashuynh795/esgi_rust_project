use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum CardinalDirection {
    North,
    East,
    South,
    West,
}
