use super::*;

use godot::classes::*;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(init, base=Node3D)]
pub struct PlayerSystem {
    #[export]
    player_model: Option<Gd<PackedScene>>,

    pub player_instances: Vec<Gd<PlayerInstance>>,

    base: Base<Node3D>,
}

#[godot_api]
impl PlayerSystem {
    #[signal]
    pub fn player_spawned(player: Gd<PlayerInstance>);

    #[signal]
    pub fn player_destroyed(player: Gd<PlayerInstance>);
}

impl PlayerSystem {
    pub fn spawn_player(&mut self, position: Vector3, direction: Direction) -> Gd<PlayerInstance> {
        let mut player = self
            .player_model
            .as_ref()
            .unwrap()
            .instantiate_as::<PlayerInstance>();

        player.set_position(position);
        player.set_rotation_degrees(direction.to_degrees_vector());

        self.to_gd().add_child(&player);
        self.player_instances.push(player.clone());

        self.signals().player_spawned().emit(&player);
        godot_print!("Player spawned");

        player
    }
}

/*
    self.debug_player
        .as_mut()
        .unwrap()
        .bind_mut()
        .move_to(grid_cell, None);
*/
