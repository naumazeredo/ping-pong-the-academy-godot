use super::*;

#[derive(GodotClass)]
#[class(init, base=Control)]
pub struct BuildingUIControl {
    #[export]
    building_list_toggle_button: Option<Gd<Button>>,

    #[export]
    building_list_panel: Option<Gd<Control>>,

    #[export]
    table_button: Option<Gd<Button>>,

    #[export]
    wall_button: Option<Gd<Button>>,

    #[export]
    floor_button: Option<Gd<Button>>,

    #[init(val = false)]
    toggled: bool,

    base: Base<Control>,
}

#[godot_api]
impl IControl for BuildingUIControl {
    fn ready(&mut self) {
        self.building_list_panel.as_mut().unwrap().hide();

        // Connect signals
        let self_gd = self.to_gd();

        let toggle_button = self.building_list_toggle_button.as_mut().unwrap();
        toggle_button
            .signals()
            .toggled()
            .connect_other(&self_gd, Self::toggle_building_list);
    }
}

// Setup
impl BuildingUIControl {
    pub fn connect_signals(&mut self, systems: &SystemsForSignals) {
        // Building buttons
        self.table_button
            .as_mut()
            .unwrap()
            .signals()
            .pressed()
            .connect_other(&systems.game_coordinator, |game_coordinator| {
                game_coordinator.change_to_building_state(PlacingLayer::Objects)
            });

        self.wall_button
            .as_mut()
            .unwrap()
            .signals()
            .pressed()
            .connect_other(&systems.game_coordinator, |game_coordinator| {
                game_coordinator.change_to_building_walls_state();
            });

        self.floor_button
            .as_mut()
            .unwrap()
            .signals()
            .pressed()
            .connect_other(&systems.game_coordinator, |game_coordinator| {
                game_coordinator.change_to_building_state(PlacingLayer::Ground)
            });

        // Overlay interaction
        let self_gd = self.to_gd();
        systems
            .ui_coordinator
            .signals()
            .opened_overlay()
            .connect_other(&self_gd, |this| {
                this.building_list_toggle_button
                    .as_mut()
                    .unwrap()
                    .run_deferred_gd(|mut this| this.set_pressed(false));

                //this.toggle_building_list(false);
            });

        // Game coordinator interaction
        self.building_list_toggle_button
            .as_mut()
            .unwrap()
            .signals()
            .toggled()
            .connect_other(&systems.game_coordinator, |game_coordinator, toggled_on| {
                if toggled_on {
                    game_coordinator.change_to_building_selecting_state();
                } else {
                    game_coordinator.change_to_managing_state();
                }
            });
    }

    pub fn process_input(&mut self, event: &Gd<InputEvent>) -> bool {
        if event.is_action_released("toggle_build") {
            let toggled = self.toggled;
            self.building_list_toggle_button
                .as_mut()
                .unwrap()
                .run_deferred_gd(move |mut this| this.set_pressed(!toggled));
            return true;
        }

        if event.is_action_released("cancel") {
            self.building_list_toggle_button
                .as_mut()
                .unwrap()
                .run_deferred_gd(|mut this| this.set_pressed(false));
            return true;
        }

        if self.toggled {
            if event.is_action_released("start_placing_objects") {
                self.table_button
                    .as_mut()
                    .unwrap()
                    .run_deferred_gd(|this| this.signals().pressed().emit());
                return true;
            }

            if event.is_action_released("start_placing_walls") {
                self.wall_button
                    .as_mut()
                    .unwrap()
                    .run_deferred_gd(|this| this.signals().pressed().emit());
                return true;
            }

            if event.is_action_released("start_placing_floor") {
                self.floor_button
                    .as_mut()
                    .unwrap()
                    .run_deferred_gd(|this| this.signals().pressed().emit());
                return true;
            }
        }

        false
    }
}

impl BuildingUIControl {
    fn toggle_building_list(&mut self, toggled_on: bool) {
        if toggled_on {
            self.open_building_list();
        } else {
            self.close_building_list();
        }
    }

    fn open_building_list(&mut self) {
        self.toggled = true;
        self.building_list_panel.as_mut().unwrap().show();
    }

    fn close_building_list(&mut self) {
        self.toggled = false;
        self.building_list_panel.as_mut().unwrap().hide();
    }
}
