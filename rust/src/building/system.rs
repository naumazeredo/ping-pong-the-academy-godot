use super::*;

use godot::classes::*;
use godot::prelude::*;

#[derive(Copy, Clone, PartialEq, Eq)]
enum SelectedLayer {
    Ground,
    Objects,
}

#[derive(Clone)]
enum BuildingSystemState {
    Selecting,
    //Selected
    Placing {
        structure: Gd<Structure>,
        layer: SelectedLayer,
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

    #[export_group(name = "Build Animation", prefix = "build_")]
    #[export]
    #[init(val = tween::EaseType::OUT)]
    build_easing: tween::EaseType,
    #[export]
    #[init(val = tween::TransitionType::BACK)]
    build_transition_type: tween::TransitionType,
    #[export]
    #[init(val = 0.3)]
    build_duration: f64,

    #[init(val = BuildingSystemState::Selecting)]
    state: BuildingSystemState,

    selector_preview_height: f32,

    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for BuildingSystem {
    fn ready(&mut self) {
        self.selector_preview_height = self.selector_preview.as_ref().unwrap().get_position().y;
    }

    fn process(&mut self, delta: f64) {
        // Draw grid and get `grid_cell` and
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

        let handle_placing_inputs = match self.state {
            BuildingSystemState::Selecting => true,

            BuildingSystemState::Placing { .. } => {
                // Handle selector mesh and placing logic
                if let Some(grid_cell) = maybe_grid_cell {
                    // Update preview
                    // TODO: only do this when moving to a new grid cell since this is a bit too costly right now
                    self.update_selection_preview_material(self.can_place(grid_cell));

                    // Structure rotation
                    if Input::singleton().is_action_just_pressed("rotate") {
                        self.rotate_preview();
                    }

                    // Check if is building
                    if Input::singleton().is_action_just_pressed("build") {
                        self.try_place(grid_cell);
                    }
                }

                true
            }
        };

        if handle_placing_inputs {
            // Handle placing state inputs
            // XXX: right now this has to be called after the `update_selection_preview_material`, otherwise we can end up updating
            // the mesh material of the wrong mesh. We should have a simple, safe way to address this and avoid having this limitation
            if Input::singleton().is_action_just_pressed("stop_placing") {
                self.stop_placing();
            }

            if Input::singleton().is_action_just_pressed("start_placing_ground") {
                self.start_placing(SelectedLayer::Ground);
            }

            if Input::singleton().is_action_just_pressed("start_placing_objects") {
                self.start_placing(SelectedLayer::Objects);
            }
        }
    }
}

impl BuildingSystem {
    fn start_placing(&mut self, layer: SelectedLayer) {
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

        // Reset position and rotation
        selector_preview.set_position(Vector3::new(0.0, self.selector_preview_height, 0.0));
        selector_preview.set_rotation_degrees(Vector3::ZERO);
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

            if can_place {
                material.set_transparency(base_material_3d::Transparency::DISABLED);
                material.set_albedo(Color::WHITE);
            } else {
                material.set_transparency(base_material_3d::Transparency::ALPHA);
                material.set_albedo(Color::RED.with_alpha(0.5));
            };
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

        let instantiated_model = self.get_building_layer(layer).bind_mut().try_place(
            structure_index,
            grid_cell,
            rotation,
        );

        if let Some(mut model) = instantiated_model {
            let target_position = model.get_position();
            model.set_position(Vector3::new(
                target_position.x,
                self.selector_preview_height,
                target_position.z,
            ));

            let mut tween = model.get_tree().create_tween();
            tween.set_ease(self.build_easing);
            tween.set_trans(self.build_transition_type);
            tween.tween_property(
                &model.clone().upcast::<Node>(),
                "position",
                &target_position.to_variant(),
                self.build_duration,
            );

            true
        } else {
            godot_print!("Could not place");
            false
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
