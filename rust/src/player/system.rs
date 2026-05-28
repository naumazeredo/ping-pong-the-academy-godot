use super::*;

use godot::classes::*;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(init, base=Node3D)]
pub struct PlayerSystem {
    #[export]
    player_model: Option<Gd<PackedScene>>,

    #[export]
    players: Array<Gd<Player>>,

    base: Base<Node3D>,
}

impl PlayerSystem {
    pub fn spawn_player(&mut self, position: Vector3, direction: Direction) -> Gd<Player> {
        let mut player = self
            .player_model
            .as_ref()
            .unwrap()
            .instantiate_as::<Player>();

        player.set_position(position);
        player.set_rotation_degrees(direction.to_degrees_vector());

        self.to_gd().add_child(&player);

        godot_print!("player created");

        self.players.push(&player);
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
