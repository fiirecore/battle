use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct PlayerSettings {
    pub gains_exp: bool,
}

impl Default for PlayerSettings {
    fn default() -> Self {
        Self {
            gains_exp: true,
        }
    }
}