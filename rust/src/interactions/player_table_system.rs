use super::*;

use godot::classes::*;
use godot::prelude::*;

use std::collections::HashMap;

#[derive(GodotClass)]
#[class(init, base=Node)]
pub struct PlayerTableSystem {
    #[export]
    player_system: Option<Gd<PlayerSystem>>,

    #[export]
    building_system: Option<Gd<BuildingSystem>>,

    players_at_table: HashMap<Gd<StructureInstance>, [Gd<PlayerInstance>; 2]>,

    base: Base<Node>,
}

type PlayerAssignment = [Gd<PlayerInstance>; 2];
type PlayerAssignments = Vec<PlayerAssignment>;

#[godot_api]
impl INode for PlayerTableSystem {
    fn ready(&mut self) {
        let self_gd = self.to_gd().clone();

        self.player_system
            .as_ref()
            .unwrap()
            .signals()
            .player_spawned()
            .connect_other(&self_gd, Self::on_player_spawned);

        self.player_system
            .as_ref()
            .unwrap()
            .signals()
            .player_destroyed()
            .connect_other(&self_gd, Self::on_player_destroyed);

        self.building_system
            .as_ref()
            .unwrap()
            .signals()
            .table_placed()
            .connect_other(&self_gd, Self::on_table_placed);

        self.building_system
            .as_ref()
            .unwrap()
            .signals()
            .table_removed()
            .connect_other(&self_gd, Self::on_table_removed);
    }

    fn process(&mut self, _delta: f64) {
        if Input::singleton().is_action_just_pressed("debug_auto_assign_players_to_tables") {
            let player_system_bind = self.player_system.as_ref().unwrap().bind();

            let mut assignments = PlayerAssignments::new();
            for players in player_system_bind.player_instances.chunks_exact(2) {
                assignments.push([players[0].clone(), players[1].clone()]);
            }

            // XXX: rust managing the lifetime correctly here, so force the drop here to avoid an error since this bind
            // holds the self reference
            std::mem::drop(player_system_bind);

            self.move_players_to_tables(assignments);
        }
    }
}

impl PlayerTableSystem {
    fn on_player_spawned(&mut self, _player_instance: Gd<PlayerInstance>) {
        godot_print!("at player_table_system: Player spawned");
    }

    fn on_player_destroyed(&mut self, _player_instance: Gd<PlayerInstance>) {
        godot_print!("at player_table_system: Player destroyed");
    }

    fn on_table_placed(&mut self, _table_instance: Gd<StructureInstance>) {
        godot_print!("at player_table_system: Table placed");
    }

    fn on_table_removed(&mut self, _table_instance: Gd<StructureInstance>) {
        godot_print!("at player_table_system: Table removed");
    }
}

impl PlayerTableSystem {
    pub fn move_players_to_tables(&mut self, assignments: PlayerAssignments) {
        let building_system_bind = self.building_system.as_ref().unwrap().bind();

        self.players_at_table.clear();
        for (mut players, table) in assignments
            .into_iter()
            .zip(building_system_bind.placed_tables.iter())
        {
            self.players_at_table.insert(table.clone(), players.clone());

            for (player, (position, direction)) in players.iter_mut().zip(
                table
                    .bind()
                    .player_positions_and_directions_in_table()
                    .iter(),
            ) {
                player.bind_mut().move_to(*position, Some(*direction));

                // TODO: register for reached event
            }
        }
    }
}
