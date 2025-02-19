use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Challenge {
    SecretSumModulo(u64),
    SOS,
}
