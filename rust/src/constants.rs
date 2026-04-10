use godot::classes::*;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(init, singleton)]
pub struct Constants {
    #[export_group(name = "Selector", prefix = "selector_")]
    #[export]
    #[init(val = 12.0)]
    pub selector_lerp_speed: f32,

    base: Base<Object>,
}
