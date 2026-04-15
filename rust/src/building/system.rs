use super::*;

use godot::classes::*;
use godot::prelude::*;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum PlacingLayer {
    Ground,
    Objects,
}

#[derive(Clone)]
enum BuildingSystemState {
    Selecting,
    Placing {
        structure: Gd<Structure>,
        layer: PlacingLayer,
        structure_index: u32,
        rotation: StructureRotation,
    },
    PlacingWalls {
        structure_index: u32,
        place_start_corner: Option<Vector2i>,
        end_corner_cache: Option<Vector2i>,
    },
}

impl BuildingSystemState {
    fn new_selecting() -> Self {
        Self::Selecting
    }
}

#[derive(GodotClass)]
#[class(init, base=Node3D)]
pub struct BuildingSystem {
    #[export]
    camera: Option<Gd<Camera3D>>,

    #[export]
    grid: Option<Gd<Node3D>>,

    #[export_group(name = "Selector", prefix = "selector_")]
    #[export]
    selector_preview: Option<Gd<Selector>>,
    #[export]
    selector_preview_walls: Option<Gd<Selector>>,

    // TODO: implement SelectorPreview class to hold the logic for placing multiple and to handle preview structure
    selector_preview_structures: Vec<Gd<Node3D>>,
    selector_preview_wall_structures: Vec<Gd<Node3D>>,
    selector_preview_wall_structures_is_pillar: bool,

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
    #[export]
    layer_walls: Option<Gd<BuildingWallsLayer>>,

    #[export_group(name = "Place Animation", prefix = "place_")]
    #[export]
    #[init(val = tween::EaseType::OUT)]
    place_easing: tween::EaseType,
    #[export]
    #[init(val = tween::TransitionType::BACK)]
    place_transition_type: tween::TransitionType,
    #[export]
    #[init(val = 0.3)]
    place_duration: f64,

    #[init(val = BuildingSystemState::new_selecting())]
    state: BuildingSystemState,

    // Used to give some more depth to the selection preview and to have a cool building animation
    selector_preview_height: f32,

    hovered_structure: Option<Gd<PlacedStructure>>,

    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for BuildingSystem {
    fn ready(&mut self) {
        // Use selector preview height defined in the editor inspector
        self.selector_preview_height = self.selector_preview.as_ref().unwrap().get_position().y;

        // XXX: temporarily autoload the map
        self.load_map();
    }

    fn process(&mut self, _delta: f64) {
        let mouse_projection = self.get_mouse_projection();
        let maybe_grid_cell = mouse_projection.map(|mouse_proj| self.get_grid_cell(mouse_proj));
        let maybe_wall_corner = mouse_projection.map(|mouse_proj| self.get_wall_corner(mouse_proj));

        if let Some(mouse_projection) = mouse_projection {
            // Position grid graphics
            let grid = self.grid.as_mut().unwrap();
            grid.set_position(mouse_projection);
        }

        // TODO: on state handling, we should have an update function for each state
        match self.state {
            BuildingSystemState::Selecting { .. } => {
                if let Some(grid_cell) = maybe_grid_cell {
                    // Position selector
                    let selector = self.selector_preview.as_mut().unwrap();
                    selector
                        .bind_mut()
                        .set_target_position(grid_cell.cast_float());

                    self.update_selecting_selector_mesh();

                    // Object selection
                    if Input::singleton().is_action_just_pressed("destroy_structure") {
                        self.try_destroy_hovered_object();
                    }
                }
            }

            BuildingSystemState::Placing { .. } => {
                // Handle selector mesh and placement logic
                if let Some(grid_cell) = maybe_grid_cell {
                    // Position selector
                    let selector = self.selector_preview.as_mut().unwrap();
                    selector
                        .bind_mut()
                        .set_target_position(grid_cell.cast_float());

                    // Update preview
                    // TODO: only do this when moving to a new grid cell since this is a bit too costly right now
                    self.update_placing_selection_preview_material(self.can_place(grid_cell));

                    // Structure rotation
                    if Input::singleton().is_action_just_pressed("rotate") {
                        self.placing_rotate_preview();
                    }

                    // Check if is placing structure
                    if Input::singleton().is_action_just_pressed("place_structure") {
                        self.try_place(grid_cell);
                    }
                }
            }

            BuildingSystemState::PlacingWalls { .. } => {
                if let Some(wall_corner) = maybe_wall_corner {
                    // Position selector
                    let selector = self.selector_preview.as_mut().unwrap();
                    selector
                        .bind_mut()
                        .set_target_position(wall_corner.cast_float());

                    self.update_placing_walls(wall_corner);
                }
            }
        }

        // Handle placing state inputs
        // XXX: right now this has to be called after the `update_selection_preview_material`, otherwise we can end up
        // updating the mesh material of the wrong mesh. We should have a simple, safe way to address this and avoid
        // having this limitation
        if Input::singleton().is_action_just_pressed("go_to_select_state") {
            self.change_to_selecting_state();
        }

        if Input::singleton().is_action_just_pressed("start_placing_ground") {
            self.change_to_placing_state(PlacingLayer::Ground);
        }

        if Input::singleton().is_action_just_pressed("start_placing_objects") {
            self.change_to_placing_state(PlacingLayer::Objects);
        }

        if Input::singleton().is_action_just_pressed("start_placing_walls") {
            self.change_to_placing_walls_state(maybe_wall_corner);
        }

        // Test input
        if Input::singleton().is_action_just_pressed("debug_save_map") {
            self.save_map();
        }

        if Input::singleton().is_action_just_pressed("debug_load_map") {
            self.load_map();
        }

        if Input::singleton().is_action_just_pressed("debug_clear_map") {
            self.clear_layers();
        }
    }
}

// General utils
impl BuildingSystem {
    fn clear_layers(&mut self) {
        self.layer_ground.as_mut().unwrap().bind_mut().clear();
        self.layer_objects.as_mut().unwrap().bind_mut().clear();
    }

    pub(super) fn on_mouse_enter_placed_structure(
        &mut self,
        placed_structure: Gd<PlacedStructure>,
    ) {
        // Update hovered structure
        self.hovered_structure = Some(placed_structure);
    }

    pub(super) fn on_mouse_exit_placed_structure(&mut self, placed_structure: Gd<PlacedStructure>) {
        if self.hovered_structure == Some(placed_structure) {
            self.hovered_structure = None;
        }
    }
}

// State management
impl BuildingSystem {
    fn change_to_selecting_state(&mut self) {
        if let BuildingSystemState::Selecting { .. } = self.state {
            return;
        }

        self.move_out_state();

        // Hide selector previews
        self.selector_preview.as_mut().unwrap().hide();
        self.selector_preview_walls.as_mut().unwrap().hide();

        // Resize selector mesh
        let selector_mesh = self.selector_mesh.as_mut().unwrap();
        selector_mesh
            .bind_mut()
            .set_target_size(Vector2::splat(1.0));

        self.state = BuildingSystemState::new_selecting();
    }

    fn change_to_placing_state(&mut self, layer: PlacingLayer) {
        if let BuildingSystemState::Placing {
            layer: old_layer, ..
        } = self.state
            && old_layer == layer
        {
            return;
        };

        self.move_out_state();

        // Get placing parameters
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

        // Show selector preview
        let selector_preview = self.selector_preview.as_mut().unwrap();
        selector_preview.show();

        // Update state
        self.state = BuildingSystemState::Placing {
            structure,
            layer,
            structure_index,
            rotation: StructureRotation::Up,
        };

        // Create new preview
        let mut selector_preview = selector_preview.clone();
        let Some(mut model) = self
            .get_building_layer(layer)
            .bind_mut()
            .get_or_instantiate_model(structure_index)
        else {
            unreachable!();
        };

        model.reparent(&selector_preview);
        model.set_position(Vector3::ZERO);
        model.set_rotation_degrees(Vector3::ZERO);
        self.selector_preview_structures = vec![model];

        // Reset position and rotation
        selector_preview
            .bind_mut()
            .set_offset_position(Vector2::ZERO);
        selector_preview.bind_mut().set_target_rotation(0.0);
    }

    fn change_to_placing_walls_state(&mut self, maybe_wall_corner: Option<Vector2i>) {
        self.move_out_state();

        // Update state
        self.state = BuildingSystemState::PlacingWalls {
            structure_index: 0,
            place_start_corner: None,
            end_corner_cache: None,
        };

        // Resize selector mesh
        let selector_mesh = self.selector_mesh.as_mut().unwrap();
        selector_mesh.bind_mut().set_centered(true);
        selector_mesh
            .bind_mut()
            .set_target_size(Vector2::splat(0.5));
        selector_mesh.bind_mut().set_corner_size(0.3);

        // Update preview
        self.selector_preview_walls.as_mut().unwrap().show();

        // Reposition walls selector preview
        if let Some(wall_corner) = maybe_wall_corner {
            self.selector_preview_walls
                .as_mut()
                .unwrap()
                .bind_mut()
                .set_position(wall_corner.cast_float());
        }
    }

    // Cleanup state when changing
    fn move_out_state(&mut self) {
        match self.state.clone() {
            BuildingSystemState::Selecting => {
                // Make sure the hovered structure is fully visible
                if let Some(structure) = self.hovered_structure.clone() {
                    Self::update_structure_material(structure.upcast::<Node>(), |mut material| {
                        material.set_transparency(base_material_3d::Transparency::DISABLED);
                        material.set_albedo(Color::WHITE);
                    });
                }

                // Reset selector mesh position
                self.selector_mesh
                    .as_mut()
                    .unwrap()
                    .bind_mut()
                    .set_target_position(None);
            }

            BuildingSystemState::Placing {
                layer,
                structure_index,
                ..
            } => {
                // Hide selector preview
                self.selector_preview.as_mut().unwrap().hide();

                self.reset_placing_selection_preview_material();

                let selector_preview_structures =
                    std::mem::take(&mut self.selector_preview_structures);

                for structure in selector_preview_structures.into_iter() {
                    self.get_building_layer(layer)
                        .bind_mut()
                        .return_to_pool(structure, structure_index);
                }
            }

            BuildingSystemState::PlacingWalls { .. } => {
                // Hide preview
                self.selector_preview_walls.as_mut().unwrap().hide();

                self.clear_placing_walls_preview();

                // Update selector mesh logic
                let selector_mesh = self.selector_mesh.as_mut().unwrap();
                selector_mesh.bind_mut().set_centered(false);
                selector_mesh
                    .bind_mut()
                    .set_target_size(Vector2::splat(1.0));
                selector_mesh.bind_mut().set_corner_size(0.5);
            }
        }
    }
}

// Placing state
impl BuildingSystem {
    fn update_placing_selection_preview_material(&mut self, can_place: bool) {
        // XXX: this should be a temporary way to update the alpha of the preview
        //      We should use animations and avoid touching the node tree

        let selector_preview = self.selector_preview.as_mut().unwrap().clone();
        Self::update_structure_material(selector_preview.upcast::<Node>(), |mut material| {
            if can_place {
                material.set_transparency(base_material_3d::Transparency::DISABLED);
                material.set_albedo(Color::WHITE);
            } else {
                material.set_transparency(base_material_3d::Transparency::ALPHA);
                material.set_albedo(Color::RED.with_alpha(0.5));
            };
        });
    }

    fn reset_placing_selection_preview_material(&mut self) {
        let selector_preview = self.selector_preview.as_mut().unwrap().clone();
        Self::update_structure_material(selector_preview.upcast::<Node>(), |mut material| {
            material.set_transparency(base_material_3d::Transparency::DISABLED);
            material.set_albedo(Color::WHITE);
        });
    }

    fn update_structure_material<F>(structure: Gd<Node>, material_func: F)
    where
        F: Fn(Gd<BaseMaterial3D>),
    {
        // XXX: this should be a temporary way to update the alpha of the preview
        //      We should use animations and avoid touching the node tree

        for child in NodeIter::new(structure) {
            let Ok(mesh) = child.try_cast::<MeshInstance3D>() else {
                continue;
            };

            let Some(material) = mesh
                .get_active_material(0)
                .and_then(|material| material.try_cast::<BaseMaterial3D>().ok())
            else {
                continue;
            };

            material_func(material);
        }
    }

    fn placing_rotate_preview(&mut self) {
        let BuildingSystemState::Placing {
            structure,
            rotation,
            ..
        } = &mut self.state
        else {
            return;
        };

        structure.bind().rotate(rotation);
        let selector_preview = self.selector_preview.as_mut().unwrap();

        selector_preview
            .bind_mut()
            .set_target_rotation(rotation.degrees().y);
        selector_preview
            .bind_mut()
            .set_offset_position(rotation.position_offset(structure.bind().size));

        // Resize selector mesh
        let selector_mesh = self.selector_mesh.as_mut().unwrap();
        selector_mesh
            .bind_mut()
            .set_target_size(structure.bind().rotated_size(*rotation).cast_float());
    }

    fn can_place(&self, grid_cell: Vector2i) -> bool {
        let BuildingSystemState::Placing {
            layer,
            structure_index,
            rotation,
            ..
        } = self.state
        else {
            return false;
        };

        self.get_building_layer(layer)
            .bind()
            .can_place(
                structure_index,
                grid_cell,
                rotation,
                self.layer_walls.as_ref().unwrap(),
            )
            .is_some()
    }

    fn try_place(&mut self, grid_cell: Vector2i) -> bool {
        let BuildingSystemState::Placing {
            layer,
            structure_index,
            rotation,
            ..
        } = self.state
        else {
            return false;
        };

        self.try_place_in_layer(
            layer,
            structure_index,
            rotation,
            grid_cell,
            true, /* with_placing_animation */
        )
    }

    fn try_place_in_layer(
        &mut self,
        layer: PlacingLayer,
        structure_index: u32,
        rotation: StructureRotation,
        grid_cell: Vector2i,
        with_placing_animation: bool,
    ) -> bool {
        let instantiated_model = self.get_building_layer(layer).bind_mut().try_place(
            structure_index,
            grid_cell,
            rotation,
            self.layer_walls.as_mut().unwrap(),
        );

        if let Some(mut model) = instantiated_model {
            // Update signals
            model.bind().connect_building_system(&mut self.to_gd());

            let target_position = model.get_position();

            if with_placing_animation {
                model.set_position(Vector3::new(
                    target_position.x,
                    self.selector_preview_height,
                    target_position.z,
                ));

                let mut tween = model.get_tree().create_tween();
                tween.set_ease(self.place_easing);
                tween.set_trans(self.place_transition_type);
                tween.tween_property(
                    &model.clone().upcast::<Node>(),
                    "position",
                    &target_position.to_variant(),
                    self.place_duration,
                );
            } else {
                model.set_position(Vector3::new(target_position.x, 0.0, target_position.z));
            }

            true
        } else {
            false
        }
    }

    fn get_building_layer(&self, layer: PlacingLayer) -> Gd<BuildingLayer> {
        match layer {
            PlacingLayer::Ground => self.layer_ground.as_ref().unwrap().clone(),
            PlacingLayer::Objects => self.layer_objects.as_ref().unwrap().clone(),
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

// Placing Walls state
impl BuildingSystem {
    fn clear_placing_walls_preview_internal(
        mut layer_walls: Gd<BuildingWallsLayer>,
        structure_index: u32,
        is_pillar: bool,
        selector_preview_wall_structures: &mut Vec<Gd<Node3D>>,
    ) {
        let structures = std::mem::take(selector_preview_wall_structures);
        for structure in structures.into_iter() {
            layer_walls
                .bind_mut()
                .return_to_pool(structure, structure_index, is_pillar);
        }
    }

    fn clear_placing_walls_preview(&mut self) {
        let BuildingSystemState::PlacingWalls {
            structure_index, ..
        } = self.state
        else {
            return;
        };

        let layer_walls = self.layer_walls.as_mut().unwrap();

        Self::clear_placing_walls_preview_internal(
            layer_walls.clone(),
            structure_index,
            self.selector_preview_wall_structures_is_pillar,
            &mut self.selector_preview_wall_structures,
        );
    }

    fn try_place_walls(
        &mut self,
        structure_index: u32,
        start_corner: Vector2i,
        end_corner: Vector2i,
        with_placing_animation: bool,
    ) -> bool {
        let models = &self.selector_preview_wall_structures;
        let Some(placed_structures) = self
            .layer_walls
            .as_mut()
            .unwrap()
            .bind_mut()
            .try_place_from_preview(structure_index, start_corner, end_corner, models)
        else {
            return false;
        };

        for mut model in placed_structures.into_iter() {
            let target_position = model.get_position();

            if with_placing_animation {
                model.set_position(Vector3::new(
                    target_position.x,
                    self.selector_preview_height,
                    target_position.z,
                ));

                let mut tween = model.get_tree().create_tween();
                tween.set_ease(self.place_easing);
                tween.set_trans(self.place_transition_type);
                tween.tween_property(
                    &model.clone().upcast::<Node>(),
                    "position",
                    &target_position.to_variant(),
                    self.place_duration,
                );
            }
        }

        true
    }

    fn update_placing_walls(&mut self, wall_corner: Vector2i) {
        let BuildingSystemState::PlacingWalls {
            structure_index,
            mut place_start_corner,
            mut end_corner_cache,
        } = self.state
        else {
            return;
        };

        let selector_preview_walls = self.selector_preview_walls.as_mut().unwrap();

        macro_rules! clear_preview {
            () => {
                let layer_walls = self.layer_walls.as_mut().unwrap();
                Self::clear_placing_walls_preview_internal(
                    layer_walls.clone(),
                    structure_index,
                    self.selector_preview_wall_structures_is_pillar,
                    &mut self.selector_preview_wall_structures,
                );
            };
        }

        macro_rules! create_pillar_preview {
            () => {
                let layer_walls = self.layer_walls.as_mut().unwrap();
                let Some(mut model) = layer_walls
                    .bind_mut()
                    .get_or_instantiate_model(structure_index, true)
                else {
                    unreachable!()
                };

                model.reparent(&*selector_preview_walls);
                model.set_position(Vector3::ZERO);
                model.set_rotation_degrees(Vector3::ZERO);
                self.selector_preview_wall_structures = vec![model];

                self.selector_preview_wall_structures_is_pillar = true;
            };
        }

        if let Some(start_corner) = place_start_corner {
            let end_corner = BuildingWallsLayer::real_end_corner(start_corner, wall_corner);
            if end_corner_cache != Some(end_corner) {
                end_corner_cache = Some(end_corner);

                // Rebuild preview
                clear_preview!();

                // Create new walls
                let corner_iter = CornerIter::new(start_corner, end_corner);

                // XXX: `windows` is not implemented for iterators for some reason
                let corners: Vec<_> = corner_iter.collect();

                self.selector_preview_wall_structures
                    .reserve_exact(corners.len().saturating_sub(1));

                for window in corners.windows(2) {
                    let [corner_0, corner_1] = *window else {
                        unreachable!()
                    };

                    let layer_walls = self.layer_walls.as_mut().unwrap();
                    let Some(mut model) = layer_walls
                        .bind_mut()
                        .get_or_instantiate_model(structure_index, false)
                    else {
                        unreachable!()
                    };

                    model.reparent(&*selector_preview_walls);
                    let corner = BuildingWallsLayer::wall_start_corner(corner_0, corner_1);
                    model.set_position(grid_cell_to_global(corner - start_corner));
                    model.set_rotation_degrees(BuildingWallsLayer::wall_rotation(
                        corner_0, corner_1,
                    ));

                    self.selector_preview_wall_structures.push(model);
                }

                // Check if it's a pillar
                if self.selector_preview_wall_structures.is_empty() {
                    create_pillar_preview!();
                } else {
                    self.selector_preview_wall_structures_is_pillar = false;
                }
            }

            if Input::singleton().is_action_just_released("place_structure") {
                godot_print!("placing walls: {} {}", start_corner, wall_corner);

                // Place
                let placed = self.try_place_walls(
                    structure_index,
                    start_corner,
                    wall_corner,
                    true, /* with_animation */
                );

                if placed {
                    // In case the placed succeeded, the structures were placed and we should not return them to the
                    // pool. So we just clear the references in `selector_preview_wall_structures`
                    self.selector_preview_wall_structures.clear();
                } else {
                    // In case the placing failed, the structures need to be returned to the pool
                    clear_preview!();
                }

                // Reset values
                let selector_preview_walls = self.selector_preview_walls.as_mut().unwrap();
                selector_preview_walls
                    .bind_mut()
                    .set_position(wall_corner.cast_float());
                place_start_corner = None;
                end_corner_cache = None;
            }

            if Input::singleton().is_action_just_pressed("place_cancel") {
                clear_preview!();

                // Reset values
                let selector_preview_walls = self.selector_preview_walls.as_mut().unwrap();
                selector_preview_walls
                    .bind_mut()
                    .set_position(wall_corner.cast_float());
                place_start_corner = None;
                end_corner_cache = None;
            }
        } else {
            // Reposition walls selector preview
            selector_preview_walls
                .bind_mut()
                .set_target_position(wall_corner.cast_float());

            if self.selector_preview_wall_structures.is_empty() {
                create_pillar_preview!();
            }

            if Input::singleton().is_action_just_pressed("place_structure") {
                place_start_corner = Some(wall_corner);
                godot_print!("started placing wall: {}", wall_corner);
            }
        }

        // Update state
        self.state = BuildingSystemState::PlacingWalls {
            structure_index,
            place_start_corner,
            end_corner_cache,
        };
    }

    fn get_wall_corner(&self, mouse_projection: Vector3) -> Vector2i {
        Vector2i::new(
            (mouse_projection.x.as_f32() + 0.5).floor() as i32,
            (mouse_projection.z.as_f32() + 0.5).floor() as i32,
        )
    }
}

// Mouse-layer interaction
impl BuildingSystem {
    fn update_selecting_selector_mesh(&mut self) {
        let BuildingSystemState::Selecting = self.state else {
            unreachable!();
        };

        let selector_mesh_global_position;
        let selector_mesh_size;

        if let Some(hovered_structure) = &self.hovered_structure {
            selector_mesh_global_position = Some(hovered_structure.bind().origin.cast_float());
            selector_mesh_size = hovered_structure.bind().rotated_size().cast_float();

            /*
            // TODO: `update_selecting_hovered_structure`
            Self::update_structure_material(structure.upcast::<Node>(), |mut material| {
                material.set_transparency(base_material_3d::Transparency::ALPHA);
                material.set_albedo(Color::WHITE.with_alpha(0.5));
            });
            */
        } else {
            selector_mesh_global_position = None;
            selector_mesh_size = Vector2::splat(1.0);
        }

        // Resize selector mesh
        let selector_mesh = self.selector_mesh.as_mut().unwrap();
        selector_mesh.bind_mut().set_target_size(selector_mesh_size);
        selector_mesh
            .bind_mut()
            .set_target_position(selector_mesh_global_position);
    }

    fn try_destroy_hovered_object(&mut self) {
        let BuildingSystemState::Selecting = self.state else {
            return;
        };

        if let Some(mut hovered_structure) = self.hovered_structure.take() {
            hovered_structure.bind_mut().destroy();
        }
    }
}

// Save and load map
impl BuildingSystem {
    pub fn save_map(&self) {
        let serialized = toml::to_string(&BuildingMapSerde::new(
            self.layer_ground.as_ref().unwrap(),
            self.layer_objects.as_ref().unwrap(),
        ))
        .unwrap();

        let mut file =
            FileAccess::open("user://savedmap.map", file_access::ModeFlags::WRITE).unwrap();
        file.store_string(&serialized);

        godot_print!(
            "Map saved: {}",
            ProjectSettings::singleton().globalize_path("user://savedmap.map")
        );
    }

    pub fn load_map(&mut self) {
        // XXX: should we use Rust's file i/o to avoid having to deal with GString and converting to String?
        let Some(file) = FileAccess::open("user://savedmap.map", file_access::ModeFlags::READ)
        else {
            godot_warn!("No map to load!");
            return;
        };

        let serialized = file.get_as_text().to_string();
        let map: BuildingMapSerde = match toml::from_str(&serialized) {
            Ok(m) => m,
            Err(err) => {
                godot_warn!("Could not load map: {err}");
                return;
            }
        };

        // Cleanup layers and create the structures
        macro_rules! populate_layer {
            ($layer_name:ident, $placing_layer:expr) => {
                let layer = self.$layer_name.as_mut().unwrap();
                layer.bind_mut().clear();

                for structure in map.$layer_name.structures.iter() {
                    let succeed = self.try_place_in_layer(
                        $placing_layer,
                        structure.index,
                        structure.rotation.into(),
                        Vector2i::from_tuple(structure.origin),
                        false, /* with_placing_animation */
                    );

                    assert!(succeed);
                }
            };
        }

        populate_layer!(layer_ground, PlacingLayer::Ground);
        populate_layer!(layer_objects, PlacingLayer::Objects);
    }
}
