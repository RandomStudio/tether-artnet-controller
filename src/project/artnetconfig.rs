use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub enum ArtNetConfigMode {
    Broadcast,
    Unicast(String, String),
}
