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

    placed_structures: BTreeMap<Vector2i, Gd<PlacedStructure>>,
    pools: Vec<Gd<ObjectPool>>,

    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for BuildingLayer {
    fn ready(&mut self) {
        godot_print!("BuildingLayer: {}", self.base().get_name());

        // Create object pools
        let mut self_gd = self.to_gd();
        self.pools.reserve_exact(self.structures.len());
        for structure in self.structures.iter_shared() {
            godot_print!("-> structure: {}", structure.get_path());

            let pool = ObjectPool::create(structure.bind().model.clone().unwrap());
            self_gd.add_child(&pool);
            self.pools.push(pool);
        }
    }
}

// Structure, instancing and pooling
impl BuildingLayer {
    pub fn get_structure(&self, structure_index: u32) -> Option<Gd<Structure>> {
        self.structures.get(structure_index as usize)
    }

    // Refactor: should this return an Option?
    pub fn get_or_instantiate_model(&mut self, structure_index: u32) -> Option<Gd<Node3D>> {
        let model = self.pools[structure_index as usize]
            .bind_mut()
            .get_or_instantiate();

        Some(model)
    }

    pub fn return_to_pool<T: Inherits<Node3D>>(&mut self, object: Gd<T>, structure_index: u32) {
        self.pools[structure_index as usize]
            .bind_mut()
            .return_to_pool(object.upcast());
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
    ) -> Option<Gd<PlacedStructure>> {
        let structure = self.get_structure(structure_index)?;

        // Check if the structure can be placed
        self.can_place_from_structure(structure.clone(), cell, rotation, walls_layer)?;

        let instantiated_model = self.get_or_instantiate_model(structure_index)?;
        let cell_position = grid_cell_to_global(cell);

        let mut placed_structure = instantiated_model.cast::<PlacedStructure>();
        placed_structure.bind_mut().init_with(
            self.to_gd().clone(),
            walls_layer.clone(),
            structure.clone(),
            structure_index,
            rotation,
            cell,
        );

        placed_structure.set_rotation_degrees(rotation.degrees());
        placed_structure
            .set_position(cell_position + rotation.position_offset_3d(structure.bind().size));

        // Remove placed structures if replacing
        if self.allow_replace {
            for structure_cell in structure.bind().iter_cells(cell, rotation) {
                self.remove_placed_structure(structure_cell, walls_layer);
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

        placed_structure.reparent(&self.to_gd());

        Some(placed_structure)
    }

    pub fn remove_placed_structure(
        &mut self,
        grid_cell: Vector2i,
        walls_layer: &mut Gd<BuildingWallsLayer>,
    ) {
        let Some(placed_structure) = self.placed_structures.get(&grid_cell) else {
            return;
        };

        let structure = placed_structure.bind().structure.clone().unwrap();
        let index = placed_structure.bind().index;
        let rotation = placed_structure.bind().rotation;
        let origin = placed_structure.bind().origin;

        self.remove_placed_structure_internal(
            placed_structure.clone(),
            &structure,
            index,
            rotation,
            origin,
            walls_layer,
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn remove_placed_structure_internal(
        &mut self,
        mut placed_structure: Gd<PlacedStructure>,
        structure: &Gd<Structure>,
        index: u32,
        rotation: StructureRotation,
        origin: Vector2i,
        walls_layer: &mut Gd<BuildingWallsLayer>,
    ) {
        for structure_cell in structure.bind().iter_cells(origin, rotation) {
            let cell_placed_structure = self.placed_structures.remove(&structure_cell);
            assert!(cell_placed_structure.unwrap() == placed_structure);
        }

        for structure_cell in structure.bind().iter_inner_cells(origin, rotation) {
            walls_layer.bind_mut().free_corner(structure_cell);
        }

        self.return_to_pool(placed_structure.clone(), index);
    }

    pub fn get_placed_structure(&self, cell: Vector2i) -> Option<Gd<PlacedStructure>> {
        self.placed_structures.get(&cell).cloned()
    }

    pub fn clear(&mut self) {
        for pool in self.pools.iter_mut() {
            pool.bind_mut().return_all_to_pool();
        }

        self.placed_structures.clear();
    }
}

// Serialization
impl From<&Gd<BuildingLayer>> for BuildingLayerSerde {
    fn from(value: &Gd<BuildingLayer>) -> Self {
        let mut structures = Vec::new();

        // Filter the structures by their origin
        let mut unique_structures = BTreeSet::new();
        for structure in value.bind().placed_structures.values() {
            let origin = structure.bind().origin;
            if unique_structures.insert(origin) {
                structures.push(structure.into());
            }
        }

        Self { structures }
    }
}
