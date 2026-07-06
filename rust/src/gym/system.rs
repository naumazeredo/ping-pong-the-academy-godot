use super::*;

#[derive(GodotClass)]
#[class(init, base=Node)]
pub struct GymSystem {
    #[export]
    #[init(val = 5000)]
    pub money: i64,

    offered_member_id: Option<PlayerId>,

    members: Vec<PlayerId>,

    base: Base<Node>,
}

#[godot_api]
impl GymSystem {
    #[signal]
    pub fn offer_new_member(player_id_as_u32: u32);

    #[signal]
    pub fn accepted_member(player_id_as_u32: u32);

    #[signal]
    pub fn rejected_member(player_id_as_u32: u32);
}

impl GymSystem {
    pub fn offer_new_member(&mut self, player_id: PlayerId) {
        self.reject_member();
        self.offered_member_id = Some(player_id);
        self.signals().offer_new_member().emit(player_id.as_u32());
    }

    pub fn reject_member(&mut self) {
        if let Some(player_id) = self.offered_member_id.take() {
            self.signals().rejected_member().emit(player_id.as_u32());
        }
    }

    pub fn accept_member(&mut self) {
        let Some(offered_member_id) = self.offered_member_id.take() else {
            return;
        };

        self.members.push(offered_member_id);

        self.signals()
            .accepted_member()
            .emit(offered_member_id.as_u32());
    }
}
