mod building;
mod camera;
mod constants;
mod grid;
mod interactions;
mod player;
mod utils;

use building::*;
use constants::*;
use grid::*;
use interactions::*;
use player::*;
use utils::*;

use godot::classes::*;
use godot::prelude::*;

// Required to setup the Godot Extension
struct PingPongTheAcademyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for PingPongTheAcademyExtension {}
