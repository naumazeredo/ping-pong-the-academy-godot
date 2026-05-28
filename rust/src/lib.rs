mod building;
mod camera;
mod constants;
mod grid;
mod player;
mod utils;

use constants::*;
use grid::*;
use player::*;
use utils::*;

use godot::prelude::*;

// Required to setup the Godot Extension
struct PingPongTheAcademyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for PingPongTheAcademyExtension {}
