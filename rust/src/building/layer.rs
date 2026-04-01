use super::*;

use std::collections::HashMap;

use godot::classes::*;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(init, base=Node3D)]
pub(super) struct BuildingLayer {
    #[export]
    pub structures: Array<Gd<Structure>>,

    #[export]
    pub allow_replace: bool,

    // TODO: create a PlacedStructure here instead of a Node3D
    pub placed_structures: HashMap<Vector2i, Gd<Node3D>>,

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
    pub fn get_structure(&self, structure_index: u32) -> Option<Gd<Structure>> {
        self.structures.get(structure_index as usize)
    }

    pub fn instantiate_model_from_structure(structure: Gd<Structure>) -> Option<Gd<Node3D>> {
        if let Some(model) = structure.bind().model.clone() {
            model.try_instantiate_as::<Node3D>()
        } else {
            None
        }
    }

    pub fn instantiate_model(&self, structure_index: u32) -> Option<Gd<Node3D>> {
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

    pub fn can_place_from_structure(
        &self,
        structure: Gd<Structure>,
        cell: Vector2i,
        rotation: StructureRotation,
    ) -> Option<()> {
        if self.allow_replace {
            return Some(());
        }

        for structure_cell in structure.bind().iter_cells(cell, rotation) {
            if self.placed_structures.contains_key(&structure_cell) {
                return None;
            }
        }

        Some(())
    }

    pub fn can_place(
        &self,
        structure_index: u32,
        cell: Vector2i,
        rotation: StructureRotation,
    ) -> Option<()> {
        if self.allow_replace {
            return Some(());
        }

        let Some(structure) = self.get_structure(structure_index) else {
            return None;
        };

        self.can_place_from_structure(structure, cell, rotation)
    }

    pub fn try_place(
        &mut self,
        structure_index: u32,
        cell: Vector2i,
        rotation: StructureRotation,
    ) -> Option<Gd<Node3D>> {
        // TODO: verify if the structure can be placed

        let Some(structure) = self.get_structure(structure_index) else {
            return None;
        };

        if !self
            .can_place_from_structure(structure.clone(), cell, rotation)
            .is_some()
        {
            return None;
        }

        let Some(mut instantiated_model) =
            Self::instantiate_model_from_structure(structure.clone())
        else {
            return None;
        };

        let cell_position = Vector3::new(cell.x as f32, 0.0, cell.y as f32);
        instantiated_model.set_rotation_degrees(rotation.degrees());
        instantiated_model
            .set_position(cell_position + rotation.position_offset(structure.bind().size));

        for structure_cell in structure.bind().iter_cells(cell, rotation) {
            self.placed_structures
                .insert(structure_cell, instantiated_model.clone());
        }

        self.base_mut()
            .add_child(&instantiated_model.clone().upcast::<Node>());

        Some(instantiated_model)
    }
}
