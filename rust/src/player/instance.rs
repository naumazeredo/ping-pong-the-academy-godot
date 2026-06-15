use super::*;

use godot::builtin::math::ApproxEq;
use godot::classes::object::*;
use godot::classes::*;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(init, base=CharacterBody3D)]
pub struct PlayerInstance {
    player_id: Option<PlayerId>,

    #[export]
    navigation_agent: Option<Gd<NavigationAgent3D>>,

    #[export]
    animation_tree: Option<Gd<AnimationTree>>,

    #[init(val = PlayerState::Idle)]
    state: PlayerState,

    target_facing_direction: Option<Direction>,

    // Player system back-ref
    system: Option<Gd<PlayerSystem>>,

    base: Base<CharacterBody3D>,
}

enum PlayerState {
    Idle,
    Moving,
    ChangingFacing,
    Playing,
}

#[godot_api]
impl PlayerInstance {
    // Signals
    #[signal]
    pub fn reached_destination(player: Gd<PlayerInstance>);
}

#[godot_api]
impl ICharacterBody3D for PlayerInstance {
    fn ready(&mut self) {
        let self_gd = self.to_gd();
        self.navigation_agent
            .as_mut()
            .unwrap()
            .signals()
            .target_reached()
            .builder()
            .flags(ConnectFlags::DEFERRED)
            .connect_other_mut(&self_gd, Self::on_target_reached);

        // Set system
        self.system = self.base().get_parent().map(|parent| parent.cast());
    }

    fn physics_process(&mut self, delta: f64) {
        match self.state {
            PlayerState::Idle => self.on_idle(delta),
            PlayerState::Moving => self.on_moving(delta),
            PlayerState::ChangingFacing => self.on_changing_facing(delta),
            PlayerState::Playing => self.on_playing(delta),
        }

        let velocity_length = self.base().get_velocity().length();
        self.animation_tree.as_mut().unwrap().set(
            "parameters/idle-walk/blend_position",
            &velocity_length.to_variant(),
        );

        self.base_mut().move_and_slide();
    }
}

// Initialization
impl PlayerInstance {
    pub fn set_player_id(&mut self, player_id: PlayerId) {
        self.player_id = Some(player_id);
    }
}

// States
impl PlayerInstance {
    pub fn stop_move(&mut self) {
        let position = self.base().get_position();
        let navigation_agent = self.navigation_agent.as_mut().unwrap();
        navigation_agent.set_target_position(position);

        self.target_facing_direction = None;
    }

    pub fn start_playing(&mut self) {
        self.state = PlayerState::Playing;
    }
}

impl PlayerInstance {
    fn on_idle(&mut self, _delta: f64) {}

    fn on_moving(&mut self, delta: f64) {
        let current_position = self.base().get_global_position();
        let next_position = self
            .navigation_agent
            .as_mut()
            .unwrap()
            .get_next_path_position();
        let direction = next_position - current_position;
        let direction = Vector3::new(direction.x, 0.0, direction.z).normalized_or_zero();

        let current_rotation = self.base().get_rotation_degrees().y;
        let target_rotation = -direction
            .signed_angle_to(Vector3::BACK, Vector3::UP)
            .to_degrees();
        let rotation = current_rotation.approach_angle(target_rotation, delta as f32 * 2.0 * 360.0);

        self.base_mut()
            .set_rotation_degrees(Vector3::new(0.0, rotation, 0.0));

        self.base_mut().set_velocity(direction * 2.0);
    }

    fn on_changing_facing(&mut self, delta: f64) {
        let Some(target_rotation) = self.target_facing_direction.map(|r| r.to_degrees()) else {
            return;
        };

        let current_rotation = self.base().get_rotation_degrees().y;
        let rotation = current_rotation.approach_angle(target_rotation, delta as f32 * 2.0 * 360.0);

        self.base_mut()
            .set_rotation_degrees(Vector3::new(0.0, rotation, 0.0));

        if rotation.approx_eq(&target_rotation) {
            self.target_facing_direction = None;
            self.state = PlayerState::Idle;

            let self_gd = self.to_gd();
            self.signals().reached_destination().emit(&self_gd);
        }
    }

    fn on_playing(&mut self, _delta: f64) {}
}

// Movement
impl PlayerInstance {
    pub fn move_to(&mut self, target: Vector2, facing_direction: Option<Direction>) {
        let navigation_agent = self.navigation_agent.as_mut().unwrap();

        let navigation_map_rid = navigation_agent.get_navigation_map();
        let closest_point = NavigationServer3D::singleton()
            .map_get_closest_point(navigation_map_rid, Vector3::new(target.x, 0.0, target.y));

        navigation_agent.set_target_position(closest_point);

        self.target_facing_direction = facing_direction;

        self.state = PlayerState::Moving;
    }

    pub fn on_target_reached(&mut self) {
        log!("player reached target");

        self.base_mut().set_velocity(Vector3::ZERO);

        self.state = if self.target_facing_direction.is_some() {
            PlayerState::ChangingFacing
        } else {
            let self_gd = self.to_gd();
            self.signals().reached_destination().emit(&self_gd);
            PlayerState::Idle
        };
    }
}
