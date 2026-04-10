mod building;
mod camera;
mod constants;
mod utils;

use constants::*;
use utils::*;

use godot::prelude::*;

// Required to setup the Godot Extension
struct PingPongTheAcademyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for PingPongTheAcademyExtension {}
