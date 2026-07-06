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
    pub accept_button: Option<Gd<Button>>,
    #[export]
    pub reject_button: Option<Gd<Button>>,

    #[export_group(name = "")]
    base: Base<Control>,
}

#[godot_api]
impl IControl for NewMemberUIControl {
    fn ready(&mut self) {
        let self_gd = self.to_gd();
        self.back_button
            .as_mut()
            .unwrap()
            .signals()
            .button_up()
            .connect_other(&self_gd, |this| this.base_mut().hide());
    }
}

// Setup
impl NewMemberUIControl {
    pub fn connect_signals(
        &mut self,
        gym_system: &Gd<GymSystem>,
        player_system: &Gd<PlayerSystem>,
    ) {
        let self_gd = self.to_gd();

        gym_system
            .signals()
            .accepted_member()
            .connect_other(&self_gd, |this, _| {
                this.new_member_info.as_mut().unwrap().hide();
                this.no_member_available.as_mut().unwrap().show();
            });

        gym_system
            .signals()
            .rejected_member()
            .connect_other(&self_gd, |this, _| {
                this.new_member_info.as_mut().unwrap().hide();
                this.no_member_available.as_mut().unwrap().show();
            });

        let player_system_clone = player_system.clone();
        gym_system.signals().offer_new_member().connect_other(
            &self_gd,
            move |this, player_id_as_u32| {
                let binding = player_system_clone.bind();
                let player_data = binding.get_player_data(PlayerId::new(player_id_as_u32));
                this.populate(player_data);
            },
        );
    }
}

impl NewMemberUIControl {
    pub fn toggle(&mut self) {
        if self.base().is_visible() {
            self.base_mut().hide();
        } else {
            self.base_mut().show();
        }
    }

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
