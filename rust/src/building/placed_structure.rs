use super::*;

use godot::classes::*;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(init, base=Node3D)]
pub(super) struct PlacedStructure {
    #[export]
    pub static_body: Option<Gd<StaticBody3D>>,

    walls_layer: Option<Gd<BuildingWallsLayer>>,
    pub structure: Option<Gd<Structure>>,
    pub index: u32,
    pub origin: Vector2i,

    // Object
    pub object_layer: Option<Gd<BuildingLayer>>,
    pub object_rotation: StructureRotation,

    // Wall
    wall_direction: Option<WallDirection>,

    base: Base<Node3D>,
}

impl PlacedStructure {
    pub fn init_object(
        &mut self,
        layer: Gd<BuildingLayer>,
        walls_layer: Gd<BuildingWallsLayer>,
        structure: Gd<Structure>,
        index: u32,
        rotation: StructureRotation,
        origin: Vector2i,
    ) {
        self.object_layer = Some(layer);
        self.walls_layer = Some(walls_layer);
        self.structure = Some(structure);
        self.index = index;
        self.object_rotation = rotation;
        self.origin = origin;
    }

    pub fn init_wall(
        &mut self,
        walls_layer: Gd<BuildingWallsLayer>,
        structure: Gd<Structure>,
        index: u32,
        origin: Vector2i,
        direction: Option<WallDirection>,
    ) {
        self.walls_layer = Some(walls_layer);
        self.structure = Some(structure);
        self.index = index;
        self.origin = origin;
        self.wall_direction = direction;
    }

    pub fn destroy(&mut self) {
        let mut layer = self.object_layer.clone();
        layer
            .as_mut()
            .unwrap()
            .bind_mut()
            .remove_placed_structure_internal(
                self.to_gd(),
                self.structure.as_ref().unwrap(),
                self.index,
                self.object_rotation,
                self.origin,
                self.walls_layer.as_mut().unwrap(),
            );
    }

    // Refactor?: ideally this object shouldn't know about the BuildingSystem. But to do the same in the BuildingSystem
    // we will need to create signals in this class to that are emitted when static_body.mouse_entered/exited triggers
    // and still be able to pass the self object (which right now is giving me an error)
    pub fn connect_building_system(&self, building_system: &mut Gd<BuildingSystem>) {
        // Link signals
        let structure = self.to_gd().clone();
        let static_body = self.static_body.as_ref().unwrap();
        static_body
            .signals()
            .mouse_entered()
            .connect_other(building_system, move |this| {
                this.on_mouse_enter_placed_structure(structure.clone());
            });

        let structure = self.to_gd().clone();
        let static_body = self.static_body.as_ref().unwrap();
        static_body
            .signals()
            .mouse_exited()
            .connect_other(building_system, move |this| {
                this.on_mouse_exit_placed_structure(structure.clone());
            });
    }
}

impl PlacedStructure {
    pub fn rotated_size(&self) -> Vector2i {
        self.structure
            .as_ref()
            .unwrap()
            .bind()
            .rotated_size(self.object_rotation)
    }
}

// Serialization
impl From<&Gd<PlacedStructure>> for PlacedStructureSerde {
    fn from(value: &Gd<PlacedStructure>) -> Self {
        let is_in_tile = value.bind().structure.as_ref().unwrap().bind().is_in_tile();

        let index = value.bind().index;
        let origin = value.bind().origin;

        let rotation = if is_in_tile {
            Some(value.bind().object_rotation)
        } else {
            None
        };

        let direction = if is_in_tile {
            None
        } else {
            value.bind().wall_direction
        };

        Self {
            index,
            rotation: rotation.map(|v| v.into()),
            direction: direction.map(|v| v.into()),
            origin: (origin.x, origin.y),
        }
    }
}
