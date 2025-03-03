use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Challenge {
    SecretSumModulo(u64),
    SOS,
}
