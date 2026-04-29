use super::*;

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use godot::classes::*;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(init, base=Node3D)]
pub(super) struct BuildingLayer {
    #[export]
    pub structures: Array<Gd<Structure>>,

    #[export]
    pub allow_replace: bool,

    #[export]
    object_pools: Option<Gd<ObjectPools>>,

    placed_structures: BTreeMap<Vector2i, Gd<StructureInstance>>,

    base: Base<Node3D>,
}

impl BuildingLayer {
    pub fn clear(&mut self) {
        for placed_structure in self.placed_structures.values_mut() {
            placed_structure.bind_mut().destroy();
        }

        self.placed_structures.clear();
    }
}

#[godot_api]
impl INode3D for BuildingLayer {
    fn ready(&mut self) {
        for structure in self.structures.iter_shared() {
            self.object_pools
                .as_mut()
                .unwrap()
                .bind_mut()
                .get_or_create_pool(structure.bind().model.clone().unwrap());
        }
    }
}

// Structure, instancing and pooling
impl BuildingLayer {
    pub fn get_structure(&self, structure_index: u32) -> Option<Gd<Structure>> {
        self.structures.get(structure_index as usize)
    }

    // Refactor: should this return an Option?
    pub fn get_or_instantiate_model(
        &mut self,
        structure_index: u32,
    ) -> Option<Gd<StructureInstance>> {
        self.structures
            .at(structure_index as usize)
            .bind()
            .instantiate(self.object_pools.as_mut().unwrap())
    }
}

// Placing
impl BuildingLayer {
    pub fn can_place_from_structure(
        &self,
        structure: Gd<Structure>,
        cell: Vector2i,
        rotation: StructureRotation,
        walls_layer: &Gd<BuildingWallsLayer>,
    ) -> Option<()> {
        if self.allow_replace {
            return Some(());
        }

        for structure_cell in structure.bind().iter_cells(cell, rotation) {
            if self.placed_structures.contains_key(&structure_cell) {
                return None;
            }
        }

        for structure_cell in structure.bind().iter_inner_cells(cell, rotation) {
            if !walls_layer.bind().is_corner_available(structure_cell) {
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
        walls_layer: &Gd<BuildingWallsLayer>,
    ) -> Option<()> {
        if self.allow_replace {
            return Some(());
        }

        let structure = self.get_structure(structure_index)?;
        self.can_place_from_structure(structure, cell, rotation, walls_layer)
    }

    pub fn try_place(
        &mut self,
        structure_index: u32,
        cell: Vector2i,
        rotation: StructureRotation,
        walls_layer: &mut Gd<BuildingWallsLayer>,
    ) -> Option<Gd<StructureInstance>> {
        let structure = self.get_structure(structure_index)?;

        // Check if the structure can be placed
        self.can_place_from_structure(structure.clone(), cell, rotation, walls_layer)?;

        let instantiated_model = self.get_or_instantiate_model(structure_index)?;
        let cell_position = grid_cell_to_global(cell);

        let mut placed_structure = instantiated_model.cast::<StructureInstance>();
        placed_structure.bind_mut().place_object(
            self.to_gd().clone(),
            walls_layer.clone(),
            structure.clone(),
            structure_index,
            cell,
            rotation,
        );

        placed_structure.reparent(&self.to_gd());
        placed_structure.set_rotation_degrees(rotation.degrees());
        placed_structure.set_position(
            cell_position + rotation.position_offset_3d(structure.bind().object_size),
        );

        // Remove placed structures if replacing
        if self.allow_replace {
            for structure_cell in structure.bind().iter_cells(cell, rotation) {
                self.remove_placed_structure_at(structure_cell);
            }
        }

        // Add placed structures
        for structure_cell in structure.bind().iter_cells(cell, rotation) {
            self.placed_structures
                .insert(structure_cell, placed_structure.clone());
        }

        // Block wall corners
        for structure_cell in structure.bind().iter_inner_cells(cell, rotation) {
            walls_layer.bind_mut().block_corner(structure_cell);
        }

        Some(placed_structure)
    }

    pub(super) fn remove_placed_structure_internal(
        &mut self,
        placed_structure: Gd<StructureInstance>,
        structure: Gd<Structure>,
        origin: Vector2i,
        object_rotation: StructureRotation,
        walls_layer: &mut Gd<BuildingWallsLayer>,
    ) {
        for structure_cell in structure.bind().iter_cells(origin, object_rotation) {
            let cell_placed_structure = self.placed_structures.remove(&structure_cell);
            assert!(cell_placed_structure.unwrap() == placed_structure);
        }

        for structure_cell in structure.bind().iter_inner_cells(origin, object_rotation) {
            walls_layer.bind_mut().free_corner(structure_cell);
        }
    }

    pub fn remove_placed_structure_at(&mut self, grid_cell: Vector2i) {
        let Some(placed_structure) = self.placed_structures.get_mut(&grid_cell) else {
            return;
        };

        placed_structure.bind_mut().destroy();
    }
}

// Serialization
impl From<&Gd<BuildingLayer>> for BuildingLayerSerde {
    fn from(value: &Gd<BuildingLayer>) -> Self {
        let mut structures = Vec::new();

        // Filter the structures by their origin
        let mut unique_structures = BTreeSet::new();
        for structure in value.bind().placed_structures.values() {
            let origin = structure.bind().origin();
            if unique_structures.insert(origin) {
                structures.push(structure.into());
            }
        }

        Self { structures }
    }
}
