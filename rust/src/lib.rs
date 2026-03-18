use godot::classes::{Camera3D, GridMap, IGridMap, Input, Node3D};
use godot::prelude::*;

// Required to setup the Godot Extension
struct PingPongTheAcademyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for PingPongTheAcademyExtension {}

// Building system
#[derive(GodotClass)]
#[class(base=Node3D)]
struct BuildingSystem {
    // Ground plane used to raycast from camera to position structures in the layers
    //ground_plane: Plane,
    #[export]
    layers: Array<Gd<BuildingLayer>>,
    #[export]
    selector: Option<Gd<Node3D>>,
    #[export]
    view_camera: Option<Gd<Camera3D>>,

    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for BuildingSystem {
    fn init(base: Base<Node3D>) -> Self {
        //let ground_plane = Plane::new(Vector3::UP, 0.0);
        let layers = Array::new();

        Self {
            //ground_plane,
            layers,
            selector: None,
            view_camera: None,
            base,
        }
    }

    fn process(&mut self, _delta: f64) {}
}

/*
impl BuildingSystem {
    fn get_gridmap_cell(&self) -> Option<Vector3> {
        let mouse_position = self.base().get_viewport().unwrap().get_mouse_position();
        let view_camera = self.view_camera.as_ref().unwrap();

        self.ground_plane
            .intersect_ray(
                view_camera.project_ray_origin(mouse_position),
                view_camera.project_ray_normal(mouse_position),
            )
            .map(|world_position| {
                Vector3::new(
                    world_position.x.as_f32().round(),
                    0.0,
                    world_position.z.as_f32().round(),
                )
            })
    }
}
*/

// Building layer
#[derive(GodotClass)]
#[class(init, base=GridMap)]
struct BuildingLayer {
    #[export]
    structures: Array<Gd<Structure>>,

    base: Base<GridMap>,
}

#[godot_api]
impl IGridMap for BuildingLayer {
    fn ready(&mut self) {
        //let mesh_library = MeshLibrary::new_gd();
        for structure in self.structures.iter_shared() {
            godot_print!("structure: {}", structure.get_name());
        }
    }
}

// Building structure
#[derive(GodotClass)]
#[class(init, base=Resource)]
struct Structure {
    #[export]
    model: Option<Gd<PackedScene>>,
    #[export]
    size: Vector2i,
}

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
