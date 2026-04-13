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
    pub model: Gd<Node3D>,

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

        model.reparent(&placed);
        placed
    }

    pub fn destroy(&mut self) {
        let mut layer = self.layer.clone();
        layer.bind_mut().remove_placed_structure_internal(
            self.to_gd(),
            self.structure.clone(),
            self.index,
            self.rotation,
            self.origin,
            self.model.clone(),
        );
    }
}

impl PlacedStructure {
    pub fn rotated_size(&self) -> Vector2i {
        self.structure.bind().rotated_size(self.rotation)
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
