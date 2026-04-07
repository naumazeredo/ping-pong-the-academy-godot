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
    Selecting {
        hovered_structure: Option<Gd<PlacedStructure>>,
    },
    Placing {
        structure: Gd<Structure>,
        layer: PlacingLayer,
        structure_index: u32,
        rotation: StructureRotation,
    },
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

    #[init(val = BuildingSystemState::Selecting { hovered_structure: None })]
    state: BuildingSystemState,

    // Used to give some more depth to the selection preview and to have a cool building animation
    selector_preview_height: f32,

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

    fn process(&mut self, delta: f64) {
        let mouse_projection = self.get_mouse_projection();
        let maybe_grid_cell = mouse_projection.map(|mouse_proj| self.get_grid_cell(mouse_proj));

        if let Some(grid_cell) = maybe_grid_cell {
            let grid_cell_3d = Vector3::new(grid_cell.x as f32, 0.0, grid_cell.y as f32);

            // Position selector
            let selector = self.selector_element.as_mut().unwrap();
            let selector_position = selector.get_position();
            selector.set_position(selector_position.lerp(grid_cell_3d, delta as f32 * 16.0));

            // Position grid graphics
            let grid = self.grid.as_mut().unwrap();
            grid.set_position(mouse_projection.unwrap());
        }

        match self.state {
            BuildingSystemState::Selecting { .. } => {
                if let Some(grid_cell) = maybe_grid_cell {
                    let placed_structure = self.get_hovered_object(grid_cell);

                    self.update_selecting_hovered_structure(placed_structure.clone());

                    // Object selection
                    if Input::singleton().is_action_just_pressed("destroy_structure") {
                        self.try_destroy_hovered_object();
                    }
                }
            }

            BuildingSystemState::Placing { .. } => {
                // Handle selector mesh and placement logic
                if let Some(grid_cell) = maybe_grid_cell {
                    // Update preview
                    // TODO: only do this when moving to a new grid cell since this is a bit too costly right now
                    self.update_selection_preview_material(self.can_place(grid_cell));

                    // Structure rotation
                    if Input::singleton().is_action_just_pressed("rotate") {
                        self.rotate_preview();
                    }

                    // Check if is placing structure
                    if Input::singleton().is_action_just_pressed("place_structure") {
                        self.try_place(grid_cell);
                    }
                }
            }
        }

        // Handle placing state inputs
        // XXX: right now this has to be called after the `update_selection_preview_material`, otherwise we can end up
        // updating the mesh material of the wrong mesh. We should have a simple, safe way to address this and avoid
        // having this limitation
        if Input::singleton().is_action_just_pressed("go_to_select_state") {
            self.go_to_select_state();
        }

        if Input::singleton().is_action_just_pressed("start_placing_ground") {
            self.start_placing(PlacingLayer::Ground);
        }

        if Input::singleton().is_action_just_pressed("start_placing_objects") {
            self.start_placing(PlacingLayer::Objects);
        }

        // Test input
        if Input::singleton().is_action_just_pressed("debug_save_map") {
            self.save_map();
        }

        if Input::singleton().is_action_just_pressed("debug_load_map") {
            self.load_map();
        }
    }
}

impl BuildingSystem {
    fn start_placing(&mut self, layer: PlacingLayer) {
        if let BuildingSystemState::Placing {
            layer: old_layer, ..
        } = self.state
            && old_layer == layer
        {
            return;
        }

        // Show selector preview
        let selector_preview = self.selector_preview.as_mut().unwrap();
        selector_preview.show();

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

        // Update state
        self.state = BuildingSystemState::Placing {
            structure,
            layer,
            structure_index,
            rotation: StructureRotation::Up,
        };

        // Recreate preview
        self.recreate_selection_preview();
    }

    fn go_to_select_state(&mut self) {
        if let BuildingSystemState::Placing { .. } = self.state {
            // Hide selector preview
            self.selector_preview.as_mut().unwrap().hide();

            // Resize selector mesh
            let selector_mesh = self.selector_mesh.as_mut().unwrap();
            selector_mesh
                .bind_mut()
                .set_target_size(Vector2::splat(1.0));
        }

        // Update state
        self.state = BuildingSystemState::Selecting {
            hovered_structure: None,
        };
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

        // Reset position and rotation
        selector_preview.set_position(Vector3::new(0.0, self.selector_preview_height, 0.0));
        selector_preview.set_rotation_degrees(Vector3::ZERO);
    }

    fn update_selection_preview_material(&mut self, can_place: bool) {
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

    fn rotate_preview(&mut self) {
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

        selector_preview.set_rotation_degrees(rotation.degrees());
        selector_preview.set_position(
            rotation.position_offset(structure.bind().size)
                + Vector3::new(0.0, self.selector_preview_height, 0.0),
        );

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
            .can_place(structure_index, grid_cell, rotation)
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
        );

        if let Some(mut model) = instantiated_model {
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

// Mouse-layer interaction
impl BuildingSystem {
    fn get_hovered_object(&self, grid_cell: Vector2i) -> Option<Gd<PlacedStructure>> {
        let placed_structure = self
            .layer_objects
            .as_ref()
            .unwrap()
            .bind()
            .get_placed_structure(grid_cell);

        if placed_structure.is_some() {
            return placed_structure;
        }

        self.layer_ground
            .as_ref()
            .unwrap()
            .bind()
            .get_placed_structure(grid_cell)
    }

    fn update_selecting_hovered_structure(&mut self, structure: Option<Gd<PlacedStructure>>) {
        let BuildingSystemState::Selecting { hovered_structure } = &mut self.state else {
            return;
        };

        if let Some(structure) = hovered_structure.clone() {
            Self::update_structure_material(structure.upcast::<Node>(), |mut material| {
                material.set_transparency(base_material_3d::Transparency::DISABLED);
                material.set_albedo(Color::WHITE);
            });
        }

        *hovered_structure = structure;

        if let Some(structure) = hovered_structure.clone() {
            Self::update_structure_material(structure.upcast::<Node>(), |mut material| {
                material.set_transparency(base_material_3d::Transparency::ALPHA);
                material.set_albedo(Color::WHITE.with_alpha(0.5));
            });
        }
    }

    fn try_destroy_hovered_object(&mut self) {
        let BuildingSystemState::Selecting { hovered_structure } = &mut self.state else {
            return;
        };

        if let Some(mut hovered_structure) = hovered_structure.take() {
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
