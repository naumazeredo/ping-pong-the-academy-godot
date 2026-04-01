use godot::classes::*;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=MeshInstance3D)]
pub(super) struct SelectorMesh {
    border_size: OnReady<f32>,
    mesh: OnReady<Gd<PlaneMesh>>,
    shader_material: OnReady<Gd<ShaderMaterial>>,

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
            border_size,
            mesh,
            shader_material,
            target_size: Vector2::ONE,
            base,
        }
    }

    fn ready(&mut self) {
        self.target_size = self.mesh.get_size();
    }

    fn process(&mut self, delta: f64) {
        let current_size = self.mesh.get_size();
        let size = current_size.lerp(self.target_size, delta as f32 * 8.0);
        self.resize_internal(size);
    }
}

impl SelectorMesh {
    // TODO: do this on process with lerping
    pub fn set_target_size(&mut self, size: Vector2) {
        let border_size = *self.border_size;
        self.target_size = size + Vector2::splat(2.0 * border_size);
    }

    fn resize_internal(&mut self, size: Vector2) {
        let border_size = *self.border_size;
        let position = self.base().get_position();
        self.base_mut().set_position(Vector3::new(
            0.5 * size.x - border_size,
            position.y,
            0.5 * size.y - border_size,
        ));

        self.mesh.set_size(size);
        self.shader_material
            .set_shader_parameter("size", &Variant::from(size));
    }
}
