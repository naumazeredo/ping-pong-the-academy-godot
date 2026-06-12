use super::*;

use godot::classes::*;
use godot::prelude::*;
use godot::signal::*;

use std::collections::HashMap;

#[derive(GodotClass)]
#[class(init, base=Node)]
pub struct PlayerTableSystem {
    #[export]
    player_system: Option<Gd<PlayerSystem>>,

    #[export]
    building_system: Option<Gd<BuildingSystem>>,

    table_player_mapping: HashMap<Gd<StructureInstance>, [Gd<PlayerInstance>; 2]>,
    player_pairing: HashMap<Gd<PlayerInstance>, (Gd<PlayerInstance>, Gd<StructureInstance>)>,
    players_going_to_table: HashMap<Gd<PlayerInstance>, ConnectHandle>,

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
            .player_instance_destroyed()
            .connect_other(&self_gd, Self::on_player_destroyed);

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
    /// On player destroyed, we should treat the cases:
    /// 1. is moving to the table
    /// 2. has a pairing
    /// 2.1. pairing is moving to table
    /// 2.2. cleanup table assignment
    fn on_player_destroyed(&mut self, player: Gd<PlayerInstance>) {
        godot_print!("at player_table_system: Player destroyed");

        // 1. moving to the table:
        if let Some(connect_handle) = self.players_going_to_table.remove(&player) {
            connect_handle.disconnect();
        }

        // 2. has a pairing
        if let Some((mut other_player, table)) = self.player_pairing.remove(&player) {
            other_player.bind_mut().stop_move();

            // remove the pairing
            let other_player_pairing = self.player_pairing.remove(&other_player);
            assert_eq!(other_player_pairing, Some((player.clone(), table.clone())));

            // 2.2. pairing is moving to table
            if let Some(connect_handle) = self.players_going_to_table.remove(&other_player) {
                connect_handle.disconnect();
            }

            // 2.3. cleanup table assignment
            let table_assignment = self.table_player_mapping.remove(&table);
            assert!(table_assignment.is_some());
        }
    }

    // On table removed, we should treat the cases:
    // 1. players are playing at the table
    fn on_table_removed(&mut self, table: Gd<StructureInstance>) {
        godot_print!("at player_table_system: Table removed");

        let Some(mut players) = self.table_player_mapping.remove(&table) else {
            return;
        };

        let mut handle_player = |player: &mut Gd<PlayerInstance>| {
            if let Some(connect_handle) = self.players_going_to_table.remove(player) {
                connect_handle.disconnect();
            }

            self.player_pairing.remove(player);

            player.bind_mut().stop_move();
        };

        handle_player(&mut players[0]);
        handle_player(&mut players[1]);
    }
}

impl PlayerTableSystem {
    pub fn move_players_to_tables(&mut self, assignments: PlayerAssignments) {
        let building_system_bind = self.building_system.as_ref().unwrap().bind();

        let self_gd = self.to_gd();

        self.table_player_mapping.clear();
        for (mut players, table) in assignments
            .into_iter()
            .zip(building_system_bind.placed_tables.iter())
        {
            self.table_player_mapping
                .insert(table.clone(), players.clone());
            self.player_pairing
                .insert(players[0].clone(), (players[1].clone(), table.clone()));
            self.player_pairing
                .insert(players[1].clone(), (players[0].clone(), table.clone()));

            for (player, (position, direction)) in players.iter_mut().zip(
                table
                    .bind()
                    .player_positions_and_directions_in_table()
                    .iter(),
            ) {
                player.bind_mut().move_to(*position, Some(*direction));

                let connect_handle = player
                    .signals()
                    .reached_destination()
                    .connect_other(&self_gd, Self::change_player_to_wait);

                if let Some(old_connect_handle) = self
                    .players_going_to_table
                    .insert(player.clone(), connect_handle)
                {
                    old_connect_handle.disconnect();
                }
            }
        }
    }

    pub fn change_player_to_wait(&mut self, mut player: Gd<PlayerInstance>) {
        godot_print!("player {} reached table", player.get_name());

        let Some(connect_handle) = self.players_going_to_table.remove(&player) else {
            unreachable!();
        };

        connect_handle.disconnect();

        let Some((other_player, _)) = self.player_pairing.get_mut(&player) else {
            unreachable!();
        };

        if !self.players_going_to_table.contains_key(&other_player) {
            // If both players reached the table, start playing
            player.bind_mut().start_playing();
            other_player.bind_mut().start_playing();
        }
    }
}
