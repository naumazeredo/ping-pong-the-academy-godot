use std::collections::HashMap;

use godot::classes::{Camera3D, Input, Node3D};
use godot::prelude::*;

// Required to setup the Godot Extension
struct PingPongTheAcademyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for PingPongTheAcademyExtension {}

// Building system
#[derive(GodotClass)]
#[class(base=Node3D)]
struct BuildingSystem {
    #[export]
    camera: Option<Gd<Camera3D>>,
    #[export]
    selector: Option<Gd<Node3D>>,
    #[export]
    grid_graphics: Option<Gd<Node3D>>,

    // Ground plane used to raycast from camera to position structures in the layers
    ground_plane: Plane,

    layers: Array<Gd<BuildingLayer>>,

    // TODO: make this more safe
    current_building_layer: u32,
    current_building_structure_index: u32,

    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for BuildingSystem {
    fn init(base: Base<Node3D>) -> Self {
        let ground_plane = Plane::new(Vector3::UP, 0.0);
        let layers = Array::new();

        Self {
            ground_plane,
            layers,
            camera: None,
            selector: None,
            grid_graphics: None,
            current_building_layer: 1,
            current_building_structure_index: 0,
            base,
        }
    }

    fn ready(&mut self) {
        self.layers.clear();
        self.base().get_children().iter_shared().for_each(|child| {
            if let Ok(layer) = child.try_cast::<BuildingLayer>() {
                self.layers.push(&layer);
                godot_print!("layer: {}", layer.get_name());
            }
        })
    }

    fn process(&mut self, delta: f64) {
        if let Some(mouse_projection) = self.get_mouse_projection() {
            let grid_cell = self.get_grid_cell(mouse_projection);
            let grid_cell_3d = Vector3::new(grid_cell.x as f32, 0.0, grid_cell.y as f32);

            // Position selector
            let selector = self.selector.as_mut().unwrap();
            let selector_position = selector.get_position();
            selector.set_position(selector_position.lerp(grid_cell_3d, delta as f32 * 40.0));

            // Position grid graphics
            let grid_graphics = self.grid_graphics.as_mut().unwrap();
            grid_graphics.set_position(mouse_projection);

            // Check if is building
            if Input::singleton().is_action_just_pressed("build") {
                self.layers
                    .at(self.current_building_layer as usize)
                    .bind_mut()
                    .place(self.current_building_structure_index, grid_cell);
            }
        }
    }
}

impl BuildingSystem {
    fn get_mouse_projection(&self) -> Option<Vector3> {
        let mouse_position = self.base().get_viewport().unwrap().get_mouse_position();
        let view_camera = self.camera.as_ref().unwrap();

        self.ground_plane.intersect_ray(
            view_camera.project_ray_origin(mouse_position),
            view_camera.project_ray_normal(mouse_position),
        )
    }

    fn get_grid_cell(&self, mouse_projection: Vector3) -> Vector2i {
        Vector2i::new(
            mouse_projection.x.as_f32().floor() as i32,
            mouse_projection.z.as_f32().floor() as i32,
        )
    }
}

// Building layer
#[derive(GodotClass)]
#[class(init, base=Node3D)]
struct BuildingLayer {
    #[export]
    structures: Array<Gd<Structure>>,

    placed_structures: HashMap<Vector2i, Gd<Node3D>>,

    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for BuildingLayer {
    fn ready(&mut self) {
        for structure in self.structures.iter_shared() {
            godot_print!("structure: {}", structure.get_path());
        }
    }
}

impl BuildingLayer {
    fn place(&mut self, structure_index: u32, cell: Vector2i) {
        // TODO: verify if the structure can be placed

        let Some(model) = self
            .structures
            .get(structure_index as usize)
            .and_then(|structure| structure.bind().model.clone())
        else {
            godot_warn!(
                "Tried to place an invalid structure in layer: {} (structure index {})",
                self.base().get_name(),
                structure_index
            );
            return;
        };

        let mut instantiated_model = model.instantiate_as::<Node3D>();
        instantiated_model.set_position(Vector3::new(cell.x as f32, 0.0, cell.y as f32));

        self.placed_structures
            .insert(cell, instantiated_model.clone());

        self.base_mut()
            .add_child(&instantiated_model.upcast::<Node>());
    }
}

// Building structure
#[derive(GodotClass)]
#[class(tool, init, base=Resource)]
struct Structure {
    #[export]
    model: Option<Gd<PackedScene>>,
    #[export]
    #[init(val = Vector2i::new(1, 1))]
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
