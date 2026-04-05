use super::*;

use std::collections::HashMap;

use godot::classes::*;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(no_init, base=Node3D)]
pub(super) struct PlacedStructure {
    layer: Gd<BuildingLayer>,
    structure: Gd<Structure>,
    //structure_index: u32,
    pub rotation: StructureRotation,
    pub placed_origin: Vector2i,

    base: Base<Node3D>,
}

impl PlacedStructure {
    pub fn new(
        layer: Gd<BuildingLayer>,
        structure: Gd<Structure>,
        _structure_index: u32,
        rotation: StructureRotation,
        placed_origin: Vector2i,
        model: Gd<Node3D>,
    ) -> Gd<Self> {
        let mut placed = Gd::from_init_fn(|base| Self {
            layer,
            structure,
            //structure_index,
            rotation,
            placed_origin,
            base,
        });

        placed.add_child(&model);
        placed
    }

    pub fn destroy(&mut self) {
        let mut layer = self.layer.clone();

        let structure = self.structure.clone();
        let rotation = self.rotation;
        let cell = self.placed_origin;

        let gd = self.to_gd();
        for structure_cell in structure.bind().iter_cells(cell, rotation) {
            let cell_placed_structure = layer.bind_mut().placed_structures.remove(&structure_cell);
            assert!(cell_placed_structure.unwrap() == gd);
        }

        self.base_mut().queue_free();
    }
}

#[derive(GodotClass)]
#[class(init, base=Node3D)]
pub(super) struct BuildingLayer {
    #[export]
    pub structures: Array<Gd<Structure>>,

    #[export]
    pub allow_replace: bool,

    // TODO: create a PlacedStructure here instead of a Node3D
    pub placed_structures: HashMap<Vector2i, Gd<PlacedStructure>>,

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

    pub fn instantiate_model(&self, structure_index: u32) -> Option<Gd<Node3D>> {
        if let Some(model) = self
            .get_structure(structure_index)
            .and_then(|structure| structure.bind().try_instantiate())
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

        let structure = self.get_structure(structure_index)?;
        self.can_place_from_structure(structure, cell, rotation)
    }

    pub fn try_place(
        &mut self,
        structure_index: u32,
        cell: Vector2i,
        rotation: StructureRotation,
    ) -> Option<Gd<PlacedStructure>> {
        let structure = self.get_structure(structure_index)?;

        // Check if the structure can be placed
        self.can_place_from_structure(structure.clone(), cell, rotation)?;

        let instantiated_model = structure.bind().try_instantiate()?;
        let cell_position = Vector3::new(cell.x as f32, 0.0, cell.y as f32);

        let mut placed_structure = PlacedStructure::new(
            self.to_gd().clone(),
            structure.clone(),
            structure_index,
            rotation,
            cell,
            instantiated_model,
        );

        placed_structure.set_rotation_degrees(rotation.degrees());
        placed_structure
            .set_position(cell_position + rotation.position_offset(structure.bind().size));

        for structure_cell in structure.bind().iter_cells(cell, rotation) {
            self.placed_structures
                .insert(structure_cell, placed_structure.clone());
        }

        self.base_mut().add_child(&placed_structure);

        Some(placed_structure)
    }

    pub fn get_placed_structure(&self, cell: Vector2i) -> Option<Gd<PlacedStructure>> {
        self.placed_structures.get(&cell).cloned()
    }
}
