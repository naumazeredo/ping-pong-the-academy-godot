use super::*;

#[derive(GodotClass)]
#[class(init, base=Control)]
pub struct NewMemberUIControl {
    #[export]
    new_member_info: Option<Gd<Control>>,

    #[export]
    no_member_available: Option<Gd<Control>>,

    #[export]
    name_label: Option<Gd<Label>>,

    #[export]
    rating_number_label: Option<Gd<Label>>,

    #[export_group(name = "Technique")]
    #[export_subgroup(name = "Serve", prefix = "tech_serve_")]
    #[export]
    tech_serve_spin_attribute_row: Option<Gd<NewMemberAttributeRow>>,
    #[export]
    tech_serve_accuracy_attribute_row: Option<Gd<NewMemberAttributeRow>>,
    #[export]
    tech_serve_deception_attribute_row: Option<Gd<NewMemberAttributeRow>>,

    #[export_subgroup(name = "Core", prefix = "tech_core_")]
    #[export]
    tech_core_short_game_attribute_row: Option<Gd<NewMemberAttributeRow>>,
    #[export]
    tech_core_loop_attribute_row: Option<Gd<NewMemberAttributeRow>>,
    #[export]
    tech_core_block_attribute_row: Option<Gd<NewMemberAttributeRow>>,
    #[export]
    tech_core_smash_attribute_row: Option<Gd<NewMemberAttributeRow>>,

    #[export_group(name = "Physical", prefix = "physical_")]
    #[export]
    physical_movement_attribute_row: Option<Gd<NewMemberAttributeRow>>,
    #[export]
    physical_stamina_attribute_row: Option<Gd<NewMemberAttributeRow>>,
    #[export]
    physical_reflexes_attribute_row: Option<Gd<NewMemberAttributeRow>>,

    #[export_group(name = "Mental", prefix = "mental_")]
    #[export]
    mental_motivation_attribute_row: Option<Gd<NewMemberAttributeRow>>,
    #[export]
    mental_discipline_attribute_row: Option<Gd<NewMemberAttributeRow>>,
    #[export]
    mental_confidence_attribute_row: Option<Gd<NewMemberAttributeRow>>,
    #[export]
    mental_composure_attribute_row: Option<Gd<NewMemberAttributeRow>>,
    #[export]
    mental_game_sense_attribute_row: Option<Gd<NewMemberAttributeRow>>,

    #[export_group(name = "Buttons", prefix = "button_")]
    #[export]
    back_button: Option<Gd<Button>>,
    #[export]
    accept_button: Option<Gd<Button>>,
    #[export]
    reject_button: Option<Gd<Button>>,

    tween: Option<Gd<Tween>>,

    #[export_group(name = "Animation", prefix = "anim_")]
    #[export]
    anim_elements: Array<Gd<Control>>,

    #[export]
    #[init(val = tween::EaseType::IN)]
    anim_ease: tween::EaseType,

    #[export]
    #[init(val = tween::TransitionType::SINE)]
    anim_transition: tween::TransitionType,

    #[export]
    #[init(val = 0.3)]
    anim_duration: f64,

    #[export]
    #[init(val = 0.06)]
    anim_delay: f64,

    #[export]
    #[init(val = Vector2::new(0.0, 5.0))]
    anim_delta_position: Vector2,

    #[export_group(name = "")]
    base: Base<Control>,
}

// Setup
impl NewMemberUIControl {
    pub fn connect_signals(&mut self, systems: &SystemsForSignals) {
        let self_gd = self.to_gd();

        // Buttons
        self.accept_button
            .as_mut()
            .unwrap()
            .signals()
            .button_down()
            .connect_other(&systems.gym_system, GymSystem::accept_member);

        self.reject_button
            .as_mut()
            .unwrap()
            .signals()
            .button_down()
            .connect_other(&systems.gym_system, GymSystem::reject_member);

        self.back_button
            .as_mut()
            .unwrap()
            .signals()
            .button_up()
            .connect_other(&systems.ui_coordinator, UICoordinator::close_overlay);

        // Gym system member accept, reject and offer
        systems
            .gym_system
            .signals()
            .accepted_member()
            .connect_other(&self_gd, |this, _| {
                this.new_member_info.as_mut().unwrap().hide();
                this.no_member_available.as_mut().unwrap().show();
            });

        systems
            .gym_system
            .signals()
            .rejected_member()
            .connect_other(&self_gd, |this, _| {
                this.new_member_info.as_mut().unwrap().hide();
                this.no_member_available.as_mut().unwrap().show();
            });

        let player_system_clone = systems.player_system.clone();
        systems
            .gym_system
            .signals()
            .offer_new_member()
            .connect_other(&self_gd, move |this, player_id_as_u32| {
                let binding = player_system_clone.bind();
                let player_data = binding.get_player_data(PlayerId::new(player_id_as_u32));
                this.populate(player_data);
            });
    }
}

// Input handling
impl NewMemberUIControl {
    pub fn process_input(&mut self, event: &Gd<InputEvent>) -> bool {
        if event.is_action_released("cancel") {
            self.back_button
                .as_mut()
                .unwrap()
                .signals()
                .pressed()
                .emit();

            return true;
        }

        false
    }
}

impl NewMemberUIControl {
    pub fn populate(&mut self, player_data: &PlayerData) {
        // Show/hide respective UIs
        self.new_member_info.as_mut().unwrap().show();
        self.no_member_available.as_mut().unwrap().hide();

        // Name
        self.name_label.as_mut().unwrap().set_text(&format!(
            "{} {}",
            player_data.first_name, player_data.last_name
        ));

        // Attributes
        macro_rules! set_attribute_row {
            ($row:ident, $($attrib:tt)*) => {
                self.$row
                    .as_mut()
                    .unwrap()
                    .bind_mut()
                    .set_value(player_data.attributes.$($attrib)*);
            };
        }

        set_attribute_row!(tech_serve_spin_attribute_row, technique.serve.spin);
        set_attribute_row!(tech_serve_accuracy_attribute_row, technique.serve.accuracy);
        set_attribute_row!(
            tech_serve_deception_attribute_row,
            technique.serve.deception
        );

        set_attribute_row!(
            tech_core_short_game_attribute_row,
            technique.core.short_game
        );
        set_attribute_row!(tech_core_loop_attribute_row, technique.core.r#loop);
        set_attribute_row!(tech_core_block_attribute_row, technique.core.block);
        set_attribute_row!(tech_core_smash_attribute_row, technique.core.smash);

        set_attribute_row!(physical_movement_attribute_row, physical.movement);
        set_attribute_row!(physical_stamina_attribute_row, physical.stamina);
        set_attribute_row!(physical_reflexes_attribute_row, physical.reflexes);

        set_attribute_row!(mental_motivation_attribute_row, mental.motivation);
        set_attribute_row!(mental_discipline_attribute_row, mental.discipline);
        set_attribute_row!(mental_confidence_attribute_row, mental.confidence);
        set_attribute_row!(mental_composure_attribute_row, mental.composure);
        set_attribute_row!(mental_game_sense_attribute_row, mental.game_sense);
    }

    pub fn animate_in(&mut self) {
        for mut element in self.anim_elements.iter_shared() {
            element.set_offset_transform_enabled(true);
            element.set_offset_transform_position(self.anim_delta_position);
            element.set_modulate(Color::TRANSPARENT_WHITE);
        }

        self.base_mut().show();

        if let Some(mut tween) = self.tween.take() {
            tween.kill();
        }

        let mut tween = self.base().get_tree().create_tween();
        tween.set_parallel();

        for (index, element) in self.anim_elements.iter_shared().enumerate() {
            tween
                .tween_property(
                    &element.clone().upcast::<Node>(),
                    "offset_transform_position",
                    &Vector2::ZERO.to_variant(),
                    self.anim_duration,
                )
                .set_ease(self.anim_ease)
                .set_trans(self.anim_transition)
                .set_delay(index as f64 * self.anim_delay);

            tween
                .tween_property(
                    &element.clone().upcast::<Node>(),
                    "modulate",
                    &Color::WHITE.to_variant(),
                    self.anim_duration,
                )
                .set_ease(self.anim_ease)
                .set_trans(self.anim_transition)
                .set_delay(index as f64 * self.anim_delay);
        }

        self.tween = Some(tween);
    }
}

#[derive(GodotClass)]
#[class(init, base=HBoxContainer)]
pub struct NewMemberAttributeRow {
    #[export]
    progress_bar: Option<Gd<ProgressBar>>,

    #[export]
    value_label: Option<Gd<Label>>,

    base: Base<HBoxContainer>,
}

impl NewMemberAttributeRow {
    pub fn set_value(&mut self, attribute: Attribute) {
        self.progress_bar
            .as_mut()
            .unwrap()
            .set_value(attribute.value as f64);

        self.value_label
            .as_mut()
            .unwrap()
            .set_text(&attribute.value.to_string());
    }
}
