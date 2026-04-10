use super::*;

use godot::classes::*;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(no_init, base=Node3D)]
pub(super) struct PlacedStructure {
    pub layer: Gd<BuildingLayer>,
    pub structure: Gd<Structure>,
    pub index: u32,
    pub rotation: StructureRotation,
    pub origin: Vector2i,
    model: Gd<Node3D>,

    base: Base<Node3D>,
}

impl PlacedStructure {
    pub fn new(
        layer: Gd<BuildingLayer>,
        structure: Gd<Structure>,
        index: u32,
        rotation: StructureRotation,
        origin: Vector2i,
        mut model: Gd<Node3D>,
    ) -> Gd<Self> {
        let mut placed = Gd::from_init_fn(|base| Self {
            layer,
            structure,
            index,
            rotation,
            origin,
            model: model.clone(),
            base,
        });

        placed.set_name(&format!("placed_{}", model.get_name()));

        model
            .reparent_ex(&placed)
            .keep_global_transform(true)
            .done();
        placed
    }

    pub fn destroy(&mut self) {
        let mut layer = self.layer.clone();

        let structure = self.structure.clone();
        let rotation = self.rotation;
        let cell = self.origin;

        let gd = self.to_gd();
        for structure_cell in structure.bind().iter_cells(cell, rotation) {
            let cell_placed_structure = layer.bind_mut().placed_structures.remove(&structure_cell);
            assert!(cell_placed_structure.unwrap() == gd);
        }

        layer
            .bind_mut()
            .return_to_pool(self.model.clone(), self.index);

        self.base_mut().queue_free();
    }
}

// Serialization
impl From<&Gd<PlacedStructure>> for PlacedStructureSerde {
    fn from(value: &Gd<PlacedStructure>) -> Self {
        let index = value.bind().index;
        let rotation = value.bind().rotation;
        let origin = value.bind().origin;

        Self {
            index,
            rotation: rotation.into(),
            origin: (origin.x, origin.y),
        }
    }
}
