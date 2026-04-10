use super::*;

use godot::classes::*;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(init, base=Node3D)]
pub(super) struct Selector {
    target_position: Vector2,

    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for Selector {
    fn process(&mut self, delta: f64) {
        let current_position = self.base().get_position();
        let position = current_position.lerp(
            Vector3::new(
                self.target_position.x,
                current_position.y,
                self.target_position.y,
            ),
            delta as f32 * Constants::singleton().bind().selector_lerp_speed,
        );
        self.base_mut().set_position(position);
    }
}

impl Selector {
    pub fn set_target_position(&mut self, position: Vector2) {
        self.target_position = position;
    }
}

#[derive(GodotClass)]
#[class(base=MeshInstance3D)]
pub(super) struct SelectorMesh {
    #[export]
    selector: Option<Gd<Selector>>,

    border_size: OnReady<f32>,
    mesh: OnReady<Gd<PlaneMesh>>,
    shader_material: OnReady<Gd<ShaderMaterial>>,

    target_position: Option<Vector2>,
    target_size: Vector2,

    base: Base<MeshInstance3D>,
}

#[godot_api]
impl IMeshInstance3D for SelectorMesh {
    fn init(base: Base<MeshInstance3D>) -> Self {
        let mesh = OnReady::from_base_fn(|base| {
            base.clone()
                .cast::<MeshInstance3D>()
                .get_mesh()
                .unwrap()
                .cast::<PlaneMesh>()
        });

        let shader_material = OnReady::from_base_fn(|base| {
            base.clone()
                .cast::<MeshInstance3D>()
                .get_active_material(0)
                .unwrap()
                .cast::<ShaderMaterial>()
        });

        let border_size = OnReady::from_base_fn(|base| {
            base.clone()
                .cast::<MeshInstance3D>()
                .get_active_material(0)
                .unwrap()
                .cast::<ShaderMaterial>()
                .get_shader_parameter("border_size")
                .to()
        });

        Self {
            selector: None,
            border_size,
            mesh,
            shader_material,
            target_position: None,
            target_size: Vector2::ONE,
            base,
        }
    }

    fn ready(&mut self) {
        self.target_size = self.mesh.get_size();
    }

    fn process(&mut self, delta: f64) {
        let lerp_speed = delta as f32 * Constants::singleton().bind().selector_lerp_speed;

        let border_size = *self.border_size;

        let current_size = self.mesh.get_size();
        let size = current_size.lerp(self.target_size, lerp_speed);

        if size != current_size {
            self.mesh.set_size(size);
            self.shader_material
                .set_shader_parameter("size", &Variant::from(size));
        }

        if let Some(target_position) = self.target_position {
            // If there's a target global position, we lerp the position to it
            let current_position = self.base().get_position();
            let target_position = Vector3::new(
                0.5 * self.target_size.x + target_position.x - border_size,
                current_position.y,
                0.5 * self.target_size.y + target_position.y - border_size,
            );
            let position = current_position.lerp(target_position, lerp_speed);

            self.base_mut().set_position(position);
        } else {
            // If there's no target position, just follow the selector node
            let selector_position = self.selector.as_ref().unwrap().bind().target_position;
            let current_position = self.base().get_position();
            let target_position = Vector3::new(
                0.5 * self.target_size.x - border_size + selector_position.x,
                current_position.y,
                0.5 * self.target_size.y - border_size + selector_position.y,
            );
            let position = current_position.lerp(target_position, lerp_speed);

            self.base_mut().set_position(position);
        }
    }
}

impl SelectorMesh {
    pub fn set_target_position(&mut self, position: Option<Vector2>) {
        self.target_position = position;
    }

    pub fn set_target_size(&mut self, size: Vector2) {
        let border_size = *self.border_size;
        self.target_size = size + Vector2::splat(2.0 * border_size);
    }
}
