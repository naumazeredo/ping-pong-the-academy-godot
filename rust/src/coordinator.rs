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
    gym_new_member_ui: Option<Gd<NewMemberUIControl>>,

    #[init(val = GameState::Managing)]
    state: GameState,

    base: Base<Node>,
}

#[godot_api]
impl INode for GameCoordinator {
    fn ready(&mut self) {
        self.gym_new_member_ui.as_mut().unwrap().hide();

        self.run_deferred(Self::setup_systems);
    }

    fn unhandled_input(&mut self, event: Gd<InputEvent>) {
        if event.is_action_released("debug_offer_new_member") {
            self.gym_system
                .as_mut()
                .unwrap()
                .bind_mut()
                .offer_new_member();
        }

        if event.is_action_released("toggle_new_member_ui") {
            self.toggle_new_member_ui();
        }
    }
}

// Setup systems
impl GameCoordinator {
    fn setup_systems(&mut self) {
        self.setup_gym_member_offer();
        self.setup_new_member_ui_buttons();

        // PlayerSystem signals
        let gym_system = self.gym_system.as_ref().unwrap();
        let player_system = self.player_system.as_mut().unwrap();
        player_system.bind_mut().setup_signals(gym_system);
    }

    fn setup_gym_member_offer(&mut self) {
        let mut gym_system = self.gym_system.as_mut().unwrap().bind_mut();
        let mut player_system = self.player_system.as_mut().unwrap().bind_mut();

        gym_system.offered_member_id = Some(player_system.create_player_data());
    }

    fn setup_new_member_ui_buttons(&mut self) {
        let self_gd = self.to_gd();
        let mut gym_new_member_ui = self.gym_new_member_ui.as_mut().unwrap().bind_mut();

        gym_new_member_ui
            .accept_button
            .as_mut()
            .unwrap()
            .signals()
            .button_down()
            .connect_other(&self_gd, |coordinator| {
                coordinator
                    .gym_system
                    .as_mut()
                    .unwrap()
                    .bind_mut()
                    .accept_member();
                coordinator.gym_new_member_ui.as_mut().unwrap().hide();
            });

        gym_new_member_ui
            .reject_button
            .as_mut()
            .unwrap()
            .signals()
            .button_down()
            .connect_other(&self_gd, |coordinator| {
                coordinator
                    .gym_system
                    .as_mut()
                    .unwrap()
                    .bind_mut()
                    .reject_member();
                coordinator.gym_new_member_ui.as_mut().unwrap().hide();
            });
    }
}

// New members
impl GameCoordinator {
    pub fn toggle_new_member_ui(&mut self) {
        let gym_system = self.gym_system.as_mut().unwrap().bind_mut();
        let player_system = self.player_system.as_ref().unwrap().bind();
        let gym_new_member_ui = self.gym_new_member_ui.as_mut().unwrap();

        gym_new_member_ui.bind_mut().toggle(
            gym_system
                .offered_member_id
                .map(|player_id| player_system.get_player_data(player_id)),
        );
    }
}
