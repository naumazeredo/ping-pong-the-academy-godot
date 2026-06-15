mod building;
mod camera;
mod constants;
mod coordinator;
mod grid;
mod gym;
mod interactions;
mod player;
mod ui;
mod utils;

use building::*;
use constants::*;
//use coordinator::*;
use grid::*;
use gym::*;
//use interactions::*;
use player::*;
use ui::*;
use utils::*;

use godot::classes::*;
use godot::global::*;
use godot::prelude::*;
use godot::signal::*;

// Required to setup the Godot Extension
struct PingPongTheAcademyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for PingPongTheAcademyExtension {}
