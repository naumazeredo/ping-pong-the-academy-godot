mod system;

pub use system::*;

use super::*;

use godot::builtin::math::ApproxEq;
use godot::classes::object::*;
use godot::classes::*;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(init, base=CharacterBody3D)]
pub struct Player {
    #[export]
    navigation_agent: Option<Gd<NavigationAgent3D>>,

    #[export]
    animation_tree: Option<Gd<AnimationTree>>,

    #[init(val = PlayerMovementState::Idle)]
    state: PlayerMovementState,

    target_facing_direction: Option<Direction>,

    // Player system back-ref
    system: Option<Gd<PlayerSystem>>,

    base: Base<CharacterBody3D>,
}

enum PlayerMovementState {
    Idle,
    Moving,
    ChangingFacing,
}

#[godot_api]
impl ICharacterBody3D for Player {
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
            PlayerMovementState::Idle => self.on_idle(delta),
            PlayerMovementState::Moving => self.on_moving(delta),
            PlayerMovementState::ChangingFacing => self.on_changing_facing(delta),
        }

        let velocity_length = self.base().get_velocity().length();
        self.animation_tree.as_mut().unwrap().set(
            "parameters/idle-walk/blend_position",
            &velocity_length.to_variant(),
        );

        self.base_mut().move_and_slide();
    }
}

// States
impl Player {
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
            self.state = PlayerMovementState::Idle;
        }
    }
}

// Set movement
#[godot_api]
impl Player {
    pub fn move_to(&mut self, target_cell: Vector2i, facing_direction: Option<Direction>) {
        self.navigation_agent
            .as_mut()
            .unwrap()
            .set_target_position(grid_cell_to_global(target_cell) + Vector3::new(0.5, 0.0, 0.5));

        self.target_facing_direction = facing_direction;

        self.state = PlayerMovementState::Moving;
    }

    #[func]
    pub fn on_target_reached(&mut self) {
        self.base_mut().set_velocity(Vector3::ZERO);

        self.state = if self.target_facing_direction.is_some() {
            PlayerMovementState::ChangingFacing
        } else {
            PlayerMovementState::Idle
        };
    }
}
