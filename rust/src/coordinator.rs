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

        // Connect signals
        let gym_system = self.gym_system.as_ref().unwrap();
        let player_system = self.player_system.as_mut().unwrap();
        let gym_new_member_ui = self.gym_new_member_ui.as_mut().unwrap();

        player_system.bind_mut().connect_signals(gym_system);
        gym_new_member_ui
            .bind_mut()
            .connect_signals(gym_system, player_system);
    }

    fn setup_gym_member_offer(&mut self) {
        let mut gym_system = self.gym_system.as_mut().unwrap().bind_mut();
        let mut player_system = self.player_system.as_mut().unwrap().bind_mut();
        gym_system.offer_new_member(player_system.create_player_data());
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
            });
    }
}

// New members
impl GameCoordinator {
    pub fn toggle_new_member_ui(&mut self) {
        let gym_new_member_ui = self.gym_new_member_ui.as_mut().unwrap();
        gym_new_member_ui.bind_mut().toggle();
    }
}
