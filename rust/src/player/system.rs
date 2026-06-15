use super::*;

#[derive(GodotClass)]
#[class(init, base=Node3D)]
pub struct PlayerSystem {
    #[export]
    player_model: Option<Gd<PackedScene>>,

    #[export]
    spawn_range_x: Vector2,
    #[export]
    spawn_range_z: Vector2,

    player_names_rng: Gd<RandomNumberGenerator>,
    player_names: PlayerNames,

    players: Vec<PlayerData>,
    unused_players: Vec<PlayerId>,
    player_attributes_rng: Gd<RandomNumberGenerator>,

    pub player_instances: Vec<Gd<PlayerInstance>>,

    // XXX: doesn't need to be a Node3D
    base: Base<Node3D>,
}

#[godot_api]
impl PlayerSystem {
    #[signal]
    pub fn player_instance_spawned(player: Gd<PlayerInstance>);

    #[signal]
    pub fn player_instance_destroyed(player: Gd<PlayerInstance>);
}

#[godot_api]
impl INode3D for PlayerSystem {
    fn ready(&mut self) {
        self.player_names_rng.randomize();
        self.player_attributes_rng.randomize();

        self.load_player_names();
    }
}

// Player system setup
impl PlayerSystem {
    pub fn setup_signals(&mut self, gym_system: &Gd<GymSystem>) {
        let self_gd = self.to_gd();
        gym_system.signals().accepted_new_member().connect_other(
            &self_gd,
            |this, player_id_as_u32| {
                this.spawn_player(PlayerId::new(player_id_as_u32));
            },
        );
    }
}

// Player instances
impl PlayerSystem {
    pub fn spawn_player(&mut self, player_id: PlayerId) -> Gd<PlayerInstance> {
        let spawn_position_x =
            randf_range(self.spawn_range_x.x as f64, self.spawn_range_x.y as f64);
        let spawn_position_z =
            randf_range(self.spawn_range_z.x as f64, self.spawn_range_z.y as f64);

        self.spawn_player_at(
            player_id,
            Vector3::new(spawn_position_x as f32, 0.0, spawn_position_z as f32),
            Direction::Up,
        )
    }

    pub fn spawn_player_at(
        &mut self,
        player_id: PlayerId,
        position: Vector3,
        direction: Direction,
    ) -> Gd<PlayerInstance> {
        let mut player = self
            .player_model
            .as_ref()
            .unwrap()
            .instantiate_as::<PlayerInstance>();

        // Create player data
        player.bind_mut().set_player_id(player_id);

        // Position
        player.set_position(position);
        player.set_rotation_degrees(direction.to_degrees_vector());

        // Set parent
        self.to_gd().add_child(&player);
        self.player_instances.push(player.clone());

        self.signals().player_instance_spawned().emit(&player);
        log!("Player spawned");

        player
    }
}

// Player data
impl PlayerSystem {
    pub fn create_player_data(&mut self) -> PlayerId {
        let new_player_id = self
            .unused_players
            .pop()
            .unwrap_or_else(|| PlayerId::new(self.players.len() as _));

        let is_male = self.player_names_rng.randi_range(0, 1) == 0;
        let first_name = if is_male {
            self.player_names
                .get_male_first_name(self.player_names_rng.randi())
                .to_owned()
        } else {
            self.player_names
                .get_female_first_name(self.player_names_rng.randi())
                .to_owned()
        };

        let last_name = self
            .player_names
            .get_last_name(self.player_names_rng.randi())
            .to_owned();

        self.players.push(PlayerData {
            id: new_player_id,
            first_name,
            last_name,
            attributes: PlayerAttributes::generate(&mut self.player_attributes_rng),
        });
        new_player_id
    }

    pub fn discard_player_data(&mut self, player_id: PlayerId) {
        assert!(self.players.len() > player_id.0 as usize);
        self.unused_players.push(player_id);
    }

    pub fn get_player_data(&self, player_id: PlayerId) -> &PlayerData {
        assert!((player_id.0 as usize) < self.players.len());
        assert!(!self.unused_players.contains(&player_id));

        &self.players[player_id.0 as usize]
    }
}

// Player names
impl PlayerSystem {
    pub fn load_player_names(&mut self) {
        let male_first_names_list = FileAccess::open(
            "res://player_data/male_first_names.txt",
            file_access::ModeFlags::READ,
        )
        .map_or_else(|| "Hugo".to_owned(), |file| file.get_as_text().to_string());

        let female_first_names_list = FileAccess::open(
            "res://player_data/female_first_names.txt",
            file_access::ModeFlags::READ,
        )
        .map_or_else(|| "Bruna".to_owned(), |file| file.get_as_text().to_string());

        let last_names_list = FileAccess::open(
            "res://player_data/last_names.txt",
            file_access::ModeFlags::READ,
        )
        .map_or_else(
            || "Calderano\nTakahashi".to_owned(),
            |file| file.get_as_text().to_string(),
        );

        self.player_names.load(
            male_first_names_list,
            female_first_names_list,
            last_names_list,
        );

        let (male_first_names_len, female_first_names_len, last_names_len) =
            self.player_names.len();
        log!(
            "Loaded player names: {male_first_names_len} males, {female_first_names_len} females, {last_names_len} last names"
        );
    }
}
