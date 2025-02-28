use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CardinalDirection {
    North,
    East,
    South,
    West,
}
