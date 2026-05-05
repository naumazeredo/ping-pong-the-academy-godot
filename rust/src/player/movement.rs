use super::*;

use godot::classes::*;
use godot::classes::object::*;
use godot::global::*;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(init, base=CharacterBody3D)]
pub struct CharacterMovement {
    #[export]
    navigation_agent: Option<Gd<NavigationAgent3D>>,

    #[export]
    animation_tree: Option<Gd<AnimationTree>>,

    #[init(val = CharacterMovementState::Idle)]
    state: CharacterMovementState,

    base: Base<CharacterBody3D>,
}

enum CharacterMovementState {
    Idle,
    Moving,
}

#[godot_api]
impl ICharacterBody3D for CharacterMovement {
    fn ready(&mut self) {
        let self_gd = self.to_gd();
        self
            .navigation_agent
            .as_mut()
            .unwrap()
            .signals()
            .target_reached()
            .builder()
            .flags(ConnectFlags::DEFERRED)
            .connect_other_mut(&self_gd, Self::on_target_reached);
    }

    fn physics_process(&mut self, delta: f64) {
        match self.state {
            CharacterMovementState::Idle => self.on_idle(delta),
            CharacterMovementState::Moving => self.on_moving(delta),
        }

        let velocity_length = self.base().get_velocity().length();
        self.animation_tree
            .as_mut()
            .unwrap()
            .set("parameters/idle-walk/blend_position", &velocity_length.to_variant());

        self.base_mut().move_and_slide();
    }
}

// States
impl CharacterMovement {
    fn change_to_idle(&mut self) {
        self.state = CharacterMovementState::Idle;
    }

    fn on_idle(&mut self, _delta: f64) {
        self.base_mut().set_velocity(Vector3::ZERO);
    }

    fn on_moving(&mut self, delta: f64) {
        let current_position = self.base().get_global_position();
        let next_position = self.navigation_agent.as_mut().unwrap().get_next_path_position();
        let direction = next_position - current_position;
        let direction = Vector3::new(direction.x, 0.0, direction.z).normalized_or_zero();

        let current_rotation = self.base().get_rotation().y;
        let rotation = lerp_angle(
            current_rotation as f64,
            -direction.signed_angle_to(Vector3::BACK, Vector3::UP) as f64,
            delta * 8.0,
        );

        self.base_mut().set_rotation(
            Vector3::new(
                0.0,
                rotation as f32,
                0.0,
            )
        );

        self.base_mut().set_velocity(direction * 2.0);
    }
}

// Set movement
#[godot_api]
impl CharacterMovement {
    pub fn set_target_position(&mut self, target_cell: Vector2i) {
        godot_print!("character move: {target_cell}");
        self.navigation_agent.as_mut().unwrap().set_target_position(grid_cell_to_global(target_cell) + Vector3::new(0.5, 0.0, 0.5));

        self.state = CharacterMovementState::Moving;
    }

    #[func]
    pub fn on_target_reached(&mut self) {
        self.change_to_idle();
    }
}
