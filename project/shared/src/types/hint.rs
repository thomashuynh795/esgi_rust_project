use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Hint {
    RelativeCompass { angle: f32 },
    GridSize { columns: u32, rows: u32 },
    Secret(u64),
    SOSHelper,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SOSHelper {
    pub message: String,
}
