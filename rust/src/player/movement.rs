use godot::classes::*;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(init, base=CharacterBody3D)]
pub struct CharacterMovement {
    #[export]
    navigation_agent: Option<Gd<NavigationAgent3D>>,

    #[init(val = CharacterMovementState::Idle)]
    state: CharacterMovementState,

    base: Base<CharacterBody3D>,
}

enum CharacterMovementState {
    Idle,
    WaitingToMove { time_left: f64 },
    Moving,
}

impl ICharacterBody3D for CharacterMovement {
    fn physics_process(&mut self, delta: f64) {
        match self.state {
            CharacterMovementState::Idle => self.on_idle(delta),
            CharacterMovementState::WaitingToMove { .. } => self.on_waiting_to_move(delta),
            CharacterMovementState::Moving => self.on_moving(delta),
        }

        self.base_mut().move_and_slide();
    }
}

impl CharacterMovement {
    fn on_idle(&mut self, _delta: f64) {
        self.state = CharacterMovementState::WaitingToMove { time_left: 2.0 };
    }

    fn on_waiting_to_move(&mut self, delta: f64) {
        let CharacterMovementState::WaitingToMove { time_left } = &mut self.state else {
            unreachable!();
        };

        *time_left -= delta;
        if *time_left <= 0.0 {
            self.state = CharacterMovementState::Moving;
        }
    }

    fn on_moving(&mut self, _delta: f64) {}
}
