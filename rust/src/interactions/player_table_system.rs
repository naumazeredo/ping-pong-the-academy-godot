use super::*;

use godot::classes::*;
use godot::prelude::*;
use godot::signal::*;

use std::collections::HashMap;
use std::collections::HashSet;

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

    free_players: HashSet<Gd<PlayerInstance>>,
    free_tables: HashSet<Gd<StructureInstance>>,

    base: Base<Node>,
}

type PlayerAssignment = ([Gd<PlayerInstance>; 2], Gd<StructureInstance>);
type PlayerAssignments = Vec<PlayerAssignment>;

#[godot_api]
impl INode for PlayerTableSystem {
    fn ready(&mut self) {
        let self_gd = self.to_gd().clone();

        self.player_system
            .as_ref()
            .unwrap()
            .signals()
            .player_instance_spawned()
            .connect_other(&self_gd, Self::on_player_spawned);

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
        /*
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
        */
    }
}

impl PlayerTableSystem {
    fn on_player_spawned(&mut self, player: Gd<PlayerInstance>) {
        self.free_players.insert(player);

        // Try to assign free players (deferred since we might need to wait for the navmesh to update)
        self.run_deferred(Self::assign_free_players);
    }

    /// On player destroyed, we should treat the cases:
    /// 1. is free
    /// 2. is moving to the table
    /// 3. has a pairing
    ///  3.1. pairing is moving to table
    ///  3.2. cleanup table assignment
    fn on_player_destroyed(&mut self, player: Gd<PlayerInstance>) {
        log!("at player_table_system: Player destroyed");

        // 1. is free
        if self.free_players.remove(&player) {
            // Early return since it can't be moving to tables or be playing already
            return;
        }

        // 2. moving to the table:
        if let Some(connect_handle) = self.players_going_to_table.remove(&player) {
            connect_handle.disconnect();
        }

        // 3. has a pairing
        if let Some((mut other_player, table)) = self.player_pairing.remove(&player) {
            // Add table back to the free tables
            self.free_tables.insert(table.clone());

            // Stop other player movement and add it back to the free players
            other_player.bind_mut().stop_move();
            self.free_players.insert(other_player.clone());

            // remove the pairing
            let other_player_pairing = self.player_pairing.remove(&other_player);
            assert_eq!(other_player_pairing, Some((player.clone(), table.clone())));

            // 3.1. pairing is moving to table
            if let Some(connect_handle) = self.players_going_to_table.remove(&other_player) {
                connect_handle.disconnect();
            }

            // 3.2. cleanup table assignment
            let table_assignment = self.table_player_mapping.remove(&table);
            assert!(table_assignment.is_some());
        }

        // Try to assign free players (deferred since we might need to wait for the navmesh to update)
        self.run_deferred(Self::assign_free_players);
    }

    fn on_table_placed(&mut self, table: Gd<StructureInstance>) {
        self.free_tables.insert(table);

        // Try to assign free players (deferred since we might need to wait for the navmesh to update)
        self.run_deferred(Self::assign_free_players);
    }

    // On table removed, we should treat the cases:
    // 1. is free
    // 2. players are playing at the table
    fn on_table_removed(&mut self, table: Gd<StructureInstance>) {
        log!("at player_table_system: Table removed");

        // 1. is free
        if self.free_tables.remove(&table) {
            return;
        }

        let Some(mut players) = self.table_player_mapping.remove(&table) else {
            return;
        };

        // 2. players are playing at the table
        let mut handle_player = |player: &mut Gd<PlayerInstance>| {
            if let Some(connect_handle) = self.players_going_to_table.remove(player) {
                connect_handle.disconnect();
            }

            self.player_pairing.remove(player);
            player.bind_mut().stop_move();

            // Add back to free players
            self.free_players.insert(player.clone());
        };

        handle_player(&mut players[0]);
        handle_player(&mut players[1]);

        // Try to assign free players (deferred since we might need to wait for the navmesh to update)
        self.run_deferred(Self::assign_free_players);
    }
}

impl PlayerTableSystem {
    fn assign_free_players(&mut self) {
        log!(
            "free_players: {} free_tables: {}",
            self.free_players.len(),
            self.free_tables.len()
        );
        if self.free_players.len() < 2 || self.free_tables.is_empty() {
            return;
        }

        let mut assignments = Vec::new();

        let total_assignments = (self.free_players.len() / 2).min(self.free_tables.len());

        let free_players: Vec<_> = self.free_players.drain().collect();
        let free_tables: Vec<_> = self.free_tables.drain().collect();

        for i in 0..total_assignments {
            assignments.push((
                [free_players[2 * i].clone(), free_players[2 * i + 1].clone()],
                free_tables[i].clone(),
            ));
        }

        for player in &free_players[(total_assignments * 2)..] {
            self.free_players.insert(player.clone());
        }

        for table in &free_tables[total_assignments..] {
            self.free_tables.insert(table.clone());
        }

        self.move_players_to_tables(assignments);
    }

    pub fn move_players_to_tables(&mut self, assignments: PlayerAssignments) {
        log!("assigning players to tables");

        let self_gd = self.to_gd();

        for (mut players, table) in assignments.into_iter() {
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
        log!("player {} reached table", player.get_name());

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
