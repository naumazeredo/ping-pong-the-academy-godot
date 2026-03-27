mod utils;
use utils::*;

use std::collections::HashMap;

use godot::classes::{
    BaseMaterial3D, Camera3D, IMeshInstance3D, Input, MeshInstance3D, Node3D, PlaneMesh,
    ShaderMaterial,
};
use godot::prelude::*;

// Required to setup the Godot Extension
struct PingPongTheAcademyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for PingPongTheAcademyExtension {}

// Building system
#[derive(Copy, Clone)]
enum SelectedLayer {
    Ground,
    Objects,
}

#[derive(Clone)]
enum BuildingSystemState {
    Selecting,
    //Selected
    Placing {
        layer: SelectedLayer,
        structure_index: u32,
    },
}

#[derive(GodotClass)]
#[class(init, base=Node3D)]
struct BuildingSystem {
    #[export]
    camera: Option<Gd<Camera3D>>,

    #[export]
    grid: Option<Gd<Node3D>>,

    #[export_group(name = "Selector", prefix = "selector_")]
    #[export]
    selector_element: Option<Gd<Node3D>>,
    #[export]
    selector_preview: Option<Gd<Node3D>>,
    #[export]
    selector_mesh: Option<Gd<SelectorMesh>>,

    // Ground plane used to raycast from camera to position structures in the layers
    #[init(val = Plane::new(Vector3::UP, 0.0))]
    ground_plane: Plane,

    #[export_group(name = "Layers", prefix = "layer_")]
    #[export]
    layer_ground: Option<Gd<BuildingLayer>>,
    #[export]
    layer_objects: Option<Gd<BuildingLayer>>,

    #[init(val = BuildingSystemState::Selecting)]
    state: BuildingSystemState,

    /*
    #[init(val = SelectedLayer::Objects)]
    current_selected_layer: SelectedLayer,
    #[init(val = 0)]
    current_building_structure_index: u32,
    */
    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for BuildingSystem {
    fn process(&mut self, delta: f64) {
        // Draw grid and get `grid_cell` and
        let mouse_projection = self.get_mouse_projection();
        let maybe_grid_cell =
            mouse_projection.and_then(|mouse_proj| Some(self.get_grid_cell(mouse_proj)));

        if let Some(grid_cell) = maybe_grid_cell {
            let grid_cell_3d = Vector3::new(grid_cell.x as f32, 0.0, grid_cell.y as f32);

            // Position selector
            let selector = self.selector_element.as_mut().unwrap();
            let selector_position = selector.get_position();
            selector.set_position(selector_position.lerp(grid_cell_3d, delta as f32 * 40.0));

            // Position grid graphics
            let grid = self.grid.as_mut().unwrap();
            // SAFETY: `maybe_grid_cell` is only `Some` if `mouse_projection` is `Some`;
            grid.set_position(mouse_projection.unwrap());
        }

        match self.state {
            BuildingSystemState::Selecting => {
                // Handle start placing
                if Input::singleton().is_action_just_pressed("start_placing") {
                    self.start_placing();
                }
            }

            BuildingSystemState::Placing { .. } => {
                // Handle stop placing
                if Input::singleton().is_action_just_pressed("stop_placing") {
                    self.stop_placing();
                }

                // Handle selector mesh and placing logic
                if let Some(grid_cell) = maybe_grid_cell {
                    // Update preview
                    // TODO: only do this when moving to a new grid cell since this is a bit too costly right now
                    self.update_selection_preview_material(self.can_place(grid_cell));

                    // Check if is building
                    if Input::singleton().is_action_just_pressed("build") {
                        self.try_place(grid_cell);
                    }
                }
            }
        }
    }
}

impl BuildingSystem {
    fn start_placing(&mut self) {
        if let BuildingSystemState::Placing { .. } = self.state {
            unreachable!();
        }

        // Show selector preview
        self.selector_preview.as_mut().unwrap().show();

        // Get placing parameters
        let layer = SelectedLayer::Objects;
        let structure_index = 0;
        let structure = self
            .get_building_layer(layer)
            .bind()
            .get_structure(structure_index)
            .expect("Building layer is empty");

        // Resize selector mesh
        let selector_mesh = self.selector_mesh.as_mut().unwrap();
        selector_mesh
            .bind_mut()
            .set_target_size(structure.bind().size.cast_float());

        // Update state
        self.state = BuildingSystemState::Placing {
            layer,
            structure_index,
        };

        // Recreate preview
        self.recreate_selection_preview();
    }

    fn stop_placing(&mut self) {
        if let BuildingSystemState::Placing { .. } = self.state {
            // Hide selector preview
            self.selector_preview.as_mut().unwrap().hide();

            // Resize selector mesh
            let selector_mesh = self.selector_mesh.as_mut().unwrap();
            selector_mesh
                .bind_mut()
                .set_target_size(Vector2::splat(1.0));

            // Update state
            self.state = BuildingSystemState::Selecting;
        } else {
            unreachable!();
        }
    }

    fn recreate_selection_preview(&mut self) {
        let BuildingSystemState::Placing {
            layer,
            structure_index,
            ..
        } = self.state
        else {
            return;
        };

        let mut selector_preview = self.selector_preview.as_mut().unwrap().clone();
        for mut child in selector_preview.get_children().iter_shared() {
            child.queue_free();
        }

        let Some(model) = self
            .get_building_layer(layer)
            .bind()
            .instantiate_model(structure_index)
        else {
            return;
        };

        selector_preview.add_child(&model);
    }

    fn update_selection_preview_material(&mut self, can_place: bool) {
        // XXX: this should be a temporary way to update the alpha of the preview
        //      We should use animations and avoid touching the node tree

        let selector_preview = self.selector_preview.as_mut().unwrap().clone();
        for child in NodeIter::new(selector_preview.upcast::<Node>()) {
            let Ok(mesh) = child.try_cast::<MeshInstance3D>() else {
                continue;
            };

            let Some(mut material) = mesh
                .get_active_material(0)
                .and_then(|material| material.try_cast::<BaseMaterial3D>().ok())
            else {
                continue;
            };

            let color = if can_place { Color::WHITE } else { Color::RED };
            material.set_albedo(color);
        }
    }

    fn can_place(&self, grid_cell: Vector2i) -> bool {
        let BuildingSystemState::Placing {
            layer,
            structure_index,
            ..
        } = self.state
        else {
            return false;
        };

        self.get_building_layer(layer)
            .bind()
            .can_place(structure_index, grid_cell)
            .is_some()
    }

    fn try_place(&mut self, grid_cell: Vector2i) {
        let BuildingSystemState::Placing {
            layer,
            structure_index,
            ..
        } = self.state
        else {
            return;
        };

        if !self
            .get_building_layer(layer)
            .bind_mut()
            .try_place(structure_index, grid_cell)
        {
            godot_print!("Could not place");
        }
    }

    fn get_building_layer(&self, layer: SelectedLayer) -> Gd<BuildingLayer> {
        match layer {
            SelectedLayer::Ground => self.layer_ground.as_ref().unwrap().clone(),
            SelectedLayer::Objects => self.layer_objects.as_ref().unwrap().clone(),
        }
    }

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

    // TODO: create a PlacedStructure here instead of a Node3D
    placed_structures: HashMap<Vector2i, Gd<Node3D>>,

    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for BuildingLayer {
    fn ready(&mut self) {
        godot_print!("BuildingLayer: {}", self.base().get_name());
        for structure in self.structures.iter_shared() {
            godot_print!("-> structure: {}", structure.get_path());
        }
    }
}

impl BuildingLayer {
    fn get_structure(&self, structure_index: u32) -> Option<Gd<Structure>> {
        self.structures.get(structure_index as usize)
    }

    fn instantiate_model_from_structure(structure: Gd<Structure>) -> Option<Gd<Node3D>> {
        if let Some(model) = structure.bind().model.clone() {
            model.try_instantiate_as::<Node3D>()
        } else {
            None
        }
    }

    fn instantiate_model(&self, structure_index: u32) -> Option<Gd<Node3D>> {
        if let Some(model) = self
            .get_structure(structure_index)
            .and_then(|structure| Self::instantiate_model_from_structure(structure))
        {
            Some(model)
        } else {
            godot_warn!(
                "Tried to place an invalid structure in layer: {} (structure index {})",
                self.base().get_name(),
                structure_index
            );

            None
        }
    }

    fn can_place_from_structure(&self, structure: Gd<Structure>, cell: Vector2i) -> Option<()> {
        for structure_cell in structure.bind().iter_cells(cell) {
            if self.placed_structures.contains_key(&structure_cell) {
                return None;
            }
        }

        Some(())
    }

    fn can_place(&self, structure_index: u32, cell: Vector2i) -> Option<()> {
        let Some(structure) = self.get_structure(structure_index) else {
            return None;
        };

        self.can_place_from_structure(structure, cell)
    }

    fn try_place(&mut self, structure_index: u32, cell: Vector2i) -> bool {
        // TODO: verify if the structure can be placed

        let Some(structure) = self.get_structure(structure_index) else {
            return false;
        };

        if !self
            .can_place_from_structure(structure.clone(), cell)
            .is_some()
        {
            return false;
        }

        let Some(mut instantiated_model) =
            Self::instantiate_model_from_structure(structure.clone())
        else {
            return false;
        };
        instantiated_model.set_position(Vector3::new(cell.x as f32, 0.0, cell.y as f32));

        for structure_cell in structure.bind().iter_cells(cell) {
            self.placed_structures
                .insert(structure_cell, instantiated_model.clone());
        }

        self.base_mut()
            .add_child(&instantiated_model.upcast::<Node>());

        true
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

impl Structure {
    fn iter_cells(&self, origin: Vector2i) -> StructureCellsIter {
        StructureCellsIter::new(origin, self.size)
    }
}

struct StructureCellsIter {
    origin: Vector2i,
    size: Vector2i,

    // Not offset by the origin
    current: Vector2i,
}

impl StructureCellsIter {
    fn new(origin: Vector2i, size: Vector2i) -> Self {
        Self {
            origin,
            size,
            current: Vector2i::ZERO,
        }
    }
}

impl Iterator for StructureCellsIter {
    type Item = Vector2i;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.y >= self.size.y {
            return None;
        }

        let ret = self.current + self.origin;

        self.current.x += 1;
        if self.current.x >= self.size.x {
            self.current.x = 0;
            self.current.y += 1;
        }

        Some(ret)
    }
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
