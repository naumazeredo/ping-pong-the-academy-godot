mod building;
mod utils;

use utils::*;

use godot::classes::*;
use godot::prelude::*;

// Required to setup the Godot Extension
struct PingPongTheAcademyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for PingPongTheAcademyExtension {}

// GymCamera
#[derive(GodotClass)]
#[class(init, base=Node3D)]
struct GymCamera {
    target_position: Vector3,
    target_rotation: Vector3,

    target_zoom: f32,

    #[init(node = "Camera3D")]
    camera: OnReady<Gd<Camera3D>>,

    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for GymCamera {
    fn ready(&mut self) {
        // Save the initial data
        self.target_position = self.base().get_position();
        self.target_rotation = self.base().get_rotation_degrees();
        self.target_zoom = self.camera.get_position().z;
    }

    fn process(&mut self, delta: f64) {
        // Handle input
        self.handle_input(delta);

        // Smoothly update position, rotation and zoom
        let position = self.base().get_position();
        let target_position = self.target_position;
        self.base_mut()
            .set_position(position.lerp(target_position, delta as f32 * 8.0));

        let rotation = self.base().get_rotation_degrees();
        let target_rotation = self.target_rotation;
        self.base_mut()
            .set_rotation_degrees(rotation.lerp(target_rotation, delta as f32 * 8.0));

        let camera_position = self.camera.get_position();
        let target_zoom = self.target_zoom;
        self.camera.set_position(
            camera_position.lerp(Vector3::new(0.0, 0.0, target_zoom), delta as f32 * 8.0),
        );
    }
}

impl GymCamera {
    fn handle_input(&mut self, delta: f64) {
        // Position
        let mut input = Vector3::ZERO;
        input.x = Input::singleton().get_axis("camera_left", "camera_right");
        input.z = Input::singleton().get_axis("camera_forward", "camera_back");

        input = input
            .rotated(Vector3::UP, self.base().get_rotation().y)
            .try_normalized()
            .unwrap_or(Vector3::ZERO);
        self.target_position += input * 15.0 * delta as f32;

        // Rotation
        let delta_rotation = Input::singleton().get_axis("camera_rotate_cw", "camera_rotate_ccw");
        self.target_rotation.y += delta_rotation * 120.0 * delta as f32;

        // Zoom
        if Input::singleton().is_action_just_released("zoom_in") {
            self.target_zoom = (self.target_zoom - 300.0 * delta as f32).max(15.0);
        }

        if Input::singleton().is_action_just_released("zoom_out") {
            self.target_zoom = (self.target_zoom + 300.0 * delta as f32).min(80.0);
        }

        // Back to center
        if Input::singleton().is_action_pressed("camera_center") {
            self.target_position = Vector3::ZERO;
        }
    }
}

#[derive(GodotClass)]
#[class(base=MeshInstance3D)]
struct SelectorMesh {
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
