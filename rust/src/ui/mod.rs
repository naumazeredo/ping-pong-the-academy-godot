mod backstop;
mod building_ui;
mod new_member_ui;

pub use backstop::*;
pub use building_ui::*;
pub use new_member_ui::*;

use super::*;

#[derive(GodotClass)]
#[class(init, base=Node)]
pub struct UICoordinator {
    active_overlay: Option<Overlay>,

    #[export]
    building_ui: Option<Gd<BuildingUIControl>>,

    #[export]
    backstop: Option<Gd<Backstop>>,

    #[export_group(name = "Overlays", prefix = "overlay_")]
    #[export]
    overlay_new_member_ui: Option<Gd<NewMemberUIControl>>,

    base: Base<Node>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Overlay {
    NewMemberUI,
}

#[godot_api]
impl UICoordinator {
    #[signal]
    pub fn opened_overlay();

    #[signal]
    pub fn closed_overlay();
}

#[godot_api]
impl INode for UICoordinator {
    fn ready(&mut self) {
        // Hide backstop and overlays
        self.backstop.as_mut().unwrap().hide();
        self.overlay_new_member_ui.as_mut().unwrap().hide();
    }
}

impl UICoordinator {
    pub fn connect_signals(&mut self, systems: &SystemsForSignals) {
        // UIs
        self.building_ui
            .as_mut()
            .unwrap()
            .bind_mut()
            .connect_signals(systems);

        // Backstop
        self.backstop
            .as_mut()
            .unwrap()
            .bind_mut()
            .connect_signals(systems);

        // Overlays
        self.overlay_new_member_ui
            .as_mut()
            .unwrap()
            .bind_mut()
            .connect_signals(systems);

        // Game coordinator
        self.signals().opened_overlay().connect_other(
            &systems.game_coordinator,
            GameCoordinator::change_to_building_selecting_state,
        );
    }
}

impl UICoordinator {
    pub fn process_event(&mut self, event: &Gd<InputEvent>) -> bool {
        if event.is_action_released("toggle_new_member_ui") {
            // Deferred toggle since signals go back to GameCoordinator
            self.to_gd().run_deferred_gd(|mut ui_coordinator| {
                ui_coordinator
                    .bind_mut()
                    .toggle_overlay(Overlay::NewMemberUI)
            });

            return true;
        }

        if let Some(active_overlay) = self.active_overlay {
            let handled = match active_overlay {
                Overlay::NewMemberUI => self
                    .overlay_new_member_ui
                    .as_mut()
                    .unwrap()
                    .bind_mut()
                    .process_input(event),
            };

            if handled {
                return true;
            }
        } else {
            let handled = self
                .building_ui
                .as_mut()
                .unwrap()
                .bind_mut()
                .process_input(event);

            if handled {
                return true;
            }
        }

        false
    }
}

impl UICoordinator {
    fn get_overlay(&self, overlay: Overlay) -> Gd<Control> {
        match overlay {
            Overlay::NewMemberUI => self
                .overlay_new_member_ui
                .as_ref()
                .unwrap()
                .clone()
                .upcast::<Control>(),
        }
    }

    pub fn open_overlay(&mut self, overlay: Overlay) {
        if let Some(active_overlay) = self.active_overlay {
            // Ignore if trying to open the same overlay
            if overlay == active_overlay {
                return;
            }

            self.close_overlay();
        }

        self.active_overlay = Some(overlay);

        match overlay {
            Overlay::NewMemberUI => {
                self.overlay_new_member_ui
                    .as_mut()
                    .unwrap()
                    .bind_mut()
                    .animate_in();
            }
        }

        self.backstop.as_mut().unwrap().bind_mut().animate_in();

        self.signals().opened_overlay().emit();
    }

    pub fn close_overlay(&mut self) {
        let Some(active_overlay) = self.active_overlay.take() else {
            return;
        };

        let mut overlay = self.get_overlay(active_overlay);
        overlay.hide();

        self.backstop.as_mut().unwrap().bind_mut().animate_out();

        self.signals().closed_overlay().emit();
    }

    pub fn toggle_overlay(&mut self, overlay: Overlay) {
        if let Some(active_overlay) = self.active_overlay {
            self.close_overlay();

            if overlay == active_overlay {
                return;
            }
        }

        self.open_overlay(overlay);
    }
}
