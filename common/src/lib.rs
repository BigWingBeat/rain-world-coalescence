use serde::{Deserialize, Serialize};

pub const DEFAULT_PORT: u16 = 7110;

#[derive(Serialize, Deserialize)]
pub struct Handshake;
