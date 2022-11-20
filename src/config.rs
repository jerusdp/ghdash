use serde_derive::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct GhConfig {
    user: String,
    token: String,
}
