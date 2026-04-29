use super::*;

use godot::classes::*;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(init, base=Node3D)]
pub(super) struct StructureInstance {
    #[export]
    static_body: Option<Gd<StaticBody3D>>,

    is_placed: bool,
    pool: Option<Gd<ObjectPool>>,

    object_layer: Option<Gd<BuildingLayer>>,
    walls_layer: Option<Gd<BuildingWallsLayer>>,
    structure: Option<Gd<Structure>>,
    structure_index: u32,
    origin: Vector2i,

    // Object
    object_rotation: StructureRotation,

    // Wall
    wall_direction: Option<WallDirection>,

    base: Base<Node3D>,
}

impl StructureInstance {
    pub fn assign_pool(&mut self, pool: Gd<ObjectPool>) {
        self.pool = Some(pool);
    }

    pub fn place_object(
        &mut self,
        object_layer: Gd<BuildingLayer>,
        walls_layer: Gd<BuildingWallsLayer>,
        structure: Gd<Structure>,
        structure_index: u32,
        origin: Vector2i,
        rotation: StructureRotation,
    ) {
        self.is_placed = true;

        self.object_layer = Some(object_layer);
        self.walls_layer = Some(walls_layer);

        self.structure = Some(structure);
        self.structure_index = structure_index;
        self.origin = origin;

        self.object_rotation = rotation;
    }

    pub fn place_wall(
        &mut self,
        walls_layer: Gd<BuildingWallsLayer>,
        structure: Gd<Structure>,
        structure_index: u32,
        origin: Vector2i,
        direction: Option<WallDirection>,
    ) {
        self.is_placed = true;

        self.walls_layer = Some(walls_layer);

        self.structure = Some(structure);
        self.structure_index = structure_index;
        self.origin = origin;

        self.wall_direction = direction;
    }

    pub fn destroy(&mut self) {
        let self_gd = self.to_gd();
        if self.is_placed {
            if let Some(object_layer) = self.object_layer.as_mut() {
                // Object
                object_layer.bind_mut().remove_placed_structure_internal(
                    self_gd.clone(),
                    self.structure.clone().unwrap(),
                    self.origin,
                    self.object_rotation,
                    self.walls_layer.as_mut().unwrap(),
                );
            } else {
                // Wall
                self.walls_layer
                    .as_mut()
                    .unwrap()
                    .bind_mut()
                    .remove_placed_structure_at(self.origin, self.wall_direction);
            }
        }

        // Unset fields
        self.is_placed = false;
        self.object_layer = None;
        self.walls_layer = None;
        self.structure = None;
        self.structure_index = 0;
        self.origin = Vector2i::ZERO;
        self.object_rotation = StructureRotation::default();
        self.wall_direction = None;

        if let Some(pool) = &mut self.pool {
            pool.bind_mut().return_to_pool(self_gd);
        } else {
            self.base_mut().queue_free();
        }
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

impl StructureInstance {
    pub fn structure_type(&self) -> StructureType {
        self.structure.as_ref().unwrap().bind().type_
    }

    pub fn origin(&self) -> Vector2i {
        self.origin
    }

    pub fn placing_position(&self) -> Vector2 {
        let offset = if self.structure_type().is_in_tile() {
            Vector2::ZERO
        } else {
            0.5 * self
                .wall_direction
                .map(|v| v.as_vector2())
                .unwrap_or(Vector2::ZERO)
        };

        self.origin.cast_float() + offset
    }

    pub fn size(&self) -> Vector2 {
        if self.structure_type().is_in_tile() {
            self.structure
                .as_ref()
                .unwrap()
                .bind()
                .rotated_size(self.object_rotation)
                .cast_float()
        } else {
            self.wall_direction
                .map(|v| v.as_vector2())
                .unwrap_or(Vector2::ZERO)
        }
    }
}

// Serialization
impl From<&Gd<StructureInstance>> for PlacedStructureSerde {
    fn from(value: &Gd<StructureInstance>) -> Self {
        let is_in_tile = value.bind().structure.as_ref().unwrap().bind().is_in_tile();

        let index = value.bind().structure_index;
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
