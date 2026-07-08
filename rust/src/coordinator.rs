use super::*;

enum GameState {
    Building,
    Managing,
    OnTournament,
}

#[derive(GodotClass)]
#[class(init, base=Node)]
pub struct GameCoordinator {
    #[export]
    building_system: Option<Gd<BuildingSystem>>,

    #[export]
    player_system: Option<Gd<PlayerSystem>>,

    #[export]
    gym_system: Option<Gd<GymSystem>>,

    #[export]
    ui_coordinator: Option<Gd<UICoordinator>>,

    #[init(val = GameState::Managing)]
    state: GameState,

    base: Base<Node>,
}

pub struct SystemsForSignals {
    pub game_coordinator: Gd<GameCoordinator>,
    pub building_system: Gd<BuildingSystem>,
    pub player_system: Gd<PlayerSystem>,
    pub gym_system: Gd<GymSystem>,
    pub ui_coordinator: Gd<UICoordinator>,
}

#[godot_api]
impl INode for GameCoordinator {
    fn ready(&mut self) {
        self.run_deferred(Self::setup_systems);
    }

    fn unhandled_input(&mut self, event: Gd<InputEvent>) {
        match self.state {
            GameState::Managing => {}

            GameState::Building => {
                let handled = self
                    .building_system
                    .as_mut()
                    .unwrap()
                    .bind_mut()
                    .process_event(&event);
                if handled {
                    return;
                }
            }

            GameState::OnTournament => {}
        }

        let handled = self
            .ui_coordinator
            .as_mut()
            .unwrap()
            .bind_mut()
            .process_event(&event);
        if handled {
            return;
        }

        if event.is_action_released("debug_offer_new_member") {
            let new_offered_member_id = self
                .player_system
                .as_mut()
                .unwrap()
                .bind_mut()
                .create_player_data();

            self.gym_system
                .as_mut()
                .unwrap()
                .bind_mut()
                .offer_new_member(new_offered_member_id);
        }
    }
}

// Setup systems
impl GameCoordinator {
    fn get_systems_for_signals(&self) -> SystemsForSignals {
        SystemsForSignals {
            game_coordinator: self.to_gd(),
            building_system: self.building_system.as_ref().unwrap().clone(),
            player_system: self.player_system.as_ref().unwrap().clone(),
            gym_system: self.gym_system.as_ref().unwrap().clone(),
            ui_coordinator: self.ui_coordinator.as_ref().unwrap().clone(),
        }
    }

    fn setup_systems(&mut self) {
        self.setup_gym_member_offer();

        // Connect signals
        let systems = self.get_systems_for_signals();
        self.player_system
            .as_mut()
            .unwrap()
            .bind_mut()
            .connect_signals(&systems);

        self.ui_coordinator
            .as_mut()
            .unwrap()
            .bind_mut()
            .connect_signals(&systems);
    }

    fn setup_gym_member_offer(&mut self) {
        let mut gym_system = self.gym_system.as_mut().unwrap().bind_mut();
        let mut player_system = self.player_system.as_mut().unwrap().bind_mut();
        gym_system.offer_new_member(player_system.create_player_data());
    }
}

// State management
impl GameCoordinator {
    pub fn change_to_managing_state(&mut self) {
        self.state = GameState::Managing;

        self.building_system
            .as_mut()
            .unwrap()
            .bind_mut()
            .change_to_none_state();
    }

    pub fn change_to_building_selecting_state(&mut self) {
        self.state = GameState::Building;

        self.building_system
            .as_mut()
            .unwrap()
            .bind_mut()
            .change_to_selecting_state();
    }

    pub fn change_to_building_state(&mut self, layer: PlacingLayer) {
        self.state = GameState::Building;

        self.building_system
            .as_mut()
            .unwrap()
            .bind_mut()
            .change_to_placing_state(layer);
    }

    pub fn change_to_building_walls_state(&mut self) {
        self.state = GameState::Building;

        self.building_system
            .as_mut()
            .unwrap()
            .bind_mut()
            .change_to_placing_walls_state();
    }
}
