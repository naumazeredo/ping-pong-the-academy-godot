use super::*;

#[derive(GodotClass)]
#[class(init, base=ColorRect)]
pub struct Backstop {
    tween: Option<Gd<Tween>>,

    #[export]
    #[init(val = tween::TransitionType::QUAD)]
    fade_transition: tween::TransitionType,

    #[export]
    #[init(val = 0.2)]
    fade_duration: f64,

    #[export]
    #[init(val = 0.85)]
    final_alpha: f64,

    base: Base<ColorRect>,
}

impl Backstop {
    pub fn connect_signals(&mut self, systems: &SystemsForSignals) {
        self.base_mut().signals().gui_input().connect_other(
            &systems.ui_coordinator,
            |ui_coordinator, event| {
                let Ok(mouse_button_event) = event.try_cast::<InputEventMouseButton>() else {
                    return;
                };

                if !mouse_button_event.is_pressed() {
                    return;
                }

                if mouse_button_event.get_button_index() == MouseButton::LEFT {
                    ui_coordinator.close_overlay()
                }
            },
        );
    }

    pub fn animate_in(&mut self) {
        self.base_mut().show();

        let color = self.base().get_color();
        self.base_mut().set_color(color.with_alpha(0.0));

        if let Some(mut tween) = self.tween.take() {
            tween.kill();
        }

        let mut tween = self.base().get_tree().create_tween();
        tween
            .tween_property(
                &self.to_gd(),
                "color:a",
                &self.final_alpha.to_variant(),
                self.fade_duration,
            )
            .set_ease(tween::EaseType::IN)
            .set_trans(self.fade_transition);

        self.tween = Some(tween);
    }

    pub fn animate_out(&mut self) {
        if let Some(mut tween) = self.tween.take() {
            tween.kill();
        }

        let mut tween = self.base().get_tree().create_tween();
        tween
            .tween_property(
                &self.to_gd(),
                "color:a",
                &0.0.to_variant(),
                self.fade_duration,
            )
            .set_ease(tween::EaseType::OUT)
            .set_trans(self.fade_transition);

        let mut self_gd = self.to_gd();
        let callable = Callable::from_fn("backstop::animate_out/callback", move |_| self_gd.hide());
        tween.tween_callback(&callable);

        self.tween = Some(tween);
    }
}
