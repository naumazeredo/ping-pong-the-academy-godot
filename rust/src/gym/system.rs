use super::*;

#[derive(GodotClass)]
#[class(init, base=Node)]
pub struct GymSystem {
    #[export]
    player_system: Option<Gd<PlayerSystem>>,

    #[export]
    #[init(val = 5000)]
    pub money: i64,

    pub offered_member_id: Option<PlayerId>,

    members: Vec<PlayerId>,

    base: Base<Node>,
}

#[godot_api]
impl GymSystem {
    #[signal]
    pub fn accepted_new_member(player_id_as_u32: u32);
}

impl GymSystem {
    pub fn offer_new_member(&mut self) {
        let mut player_system = self.player_system.as_mut().unwrap().bind_mut();

        if let Some(offered_member_id) = self.offered_member_id {
            player_system.discard_player_data(offered_member_id);
        }
        self.offered_member_id = Some(player_system.create_player_data());
    }

    pub fn reject_member(&mut self) {
        self.offered_member_id.take();
    }

    pub fn accept_member(&mut self) {
        let Some(offered_member_id) = self.offered_member_id.take() else {
            return;
        };

        self.members.push(offered_member_id);

        self.signals()
            .accepted_new_member()
            .emit(offered_member_id.as_u32());
    }
}
