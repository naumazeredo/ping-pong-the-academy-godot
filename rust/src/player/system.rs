use super::*;

use godot::classes::*;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(init, base=Node3D)]
pub struct PlayerSystem {
    #[export]
    player_model: Option<Gd<PackedScene>>,

    #[export]
    spawn_position: Option<Gd<Node3D>>,

    player_names_rng: Gd<RandomNumberGenerator>,
    player_names: PlayerNames,

    players: Vec<PlayerData>,

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

        self.load_player_names();
    }
}

// Player instances
impl PlayerSystem {
    pub fn spawn_player(&mut self) -> Gd<PlayerInstance> {
        let position = self.spawn_position.as_ref().unwrap().get_position();
        self.spawn_player_at(position, Direction::Up)
    }

    pub fn spawn_player_at(
        &mut self,
        position: Vector3,
        direction: Direction,
    ) -> Gd<PlayerInstance> {
        let mut player = self
            .player_model
            .as_ref()
            .unwrap()
            .instantiate_as::<PlayerInstance>();

        // Create player data
        let player_id = self.create_player_data();
        player.bind_mut().set_player_id(player_id);

        // Position
        player.set_position(position);
        player.set_rotation_degrees(direction.to_degrees_vector());

        // Set parent
        self.to_gd().add_child(&player);
        self.player_instances.push(player.clone());

        self.signals().player_instance_spawned().emit(&player);
        godot_print!("Player spawned");

        player
    }
}

// Player data
impl PlayerSystem {
    pub fn create_player_data(&mut self) -> PlayerId {
        let new_player_id = PlayerId::new(self.players.len() as _);

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
            attributes: PlayerAttributes::BASE,
        });
        new_player_id
    }

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
        godot_print!(
            "Loaded player names: {male_first_names_len} males, {female_first_names_len} females, {last_names_len} last names"
        );
    }
}
