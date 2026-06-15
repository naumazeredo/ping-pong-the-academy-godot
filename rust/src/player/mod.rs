mod attributes;
mod instance;
mod names;
mod system;

pub use attributes::*;
pub use instance::*;
pub use names::*;
pub use system::*;

use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayerId(u32);

impl PlayerId {
    pub fn new(v: u32) -> Self {
        Self(v)
    }
}

pub struct PlayerData {
    pub id: PlayerId,

    pub first_name: String,
    pub last_name: String,

    pub attributes: PlayerAttributes,
}
