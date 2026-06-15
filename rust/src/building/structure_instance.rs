use super::*;

#[derive(GodotClass)]
#[class(init, base=Node3D)]
pub struct StructureInstance {
    #[export]
    static_body: Option<Gd<StaticBody3D>>,

    #[export]
    collision_shape: Option<Gd<CollisionShape3D>>,

    is_placed: bool,
    pool: Option<Gd<ObjectPool>>,

    object_layer: Option<Gd<BuildingLayer>>,
    pub walls_layer: Option<Gd<BuildingWallsLayer>>,
    pub structure: Option<Gd<Structure>>,
    pub structure_index: u32,
    pub origin: Vector2i,

    // Object
    pub object_rotation: Direction,

    // Wall
    pub wall_direction: Option<WallDirection>,

    // Connect handles
    connect_handles: Option<[ConnectHandle; 2]>,

    base: Base<Node3D>,
}

impl StructureInstance {
    pub fn assign_pool(&mut self, pool: Gd<ObjectPool>) {
        self.pool = Some(pool);
    }

    pub fn unset_fields(&mut self) {
        self.is_placed = false;
        self.object_layer = None;
        self.walls_layer = None;
        self.structure = None;
        self.structure_index = 0;
        self.origin = Vector2i::ZERO;
        self.object_rotation = Direction::default();
        self.wall_direction = None;
    }

    pub fn enable_collision(&mut self) {
        self.collision_shape
            .as_mut()
            .unwrap()
            .set_deferred("disabled", &false.to_variant());
    }

    pub fn disable_collision(&mut self) {
        self.collision_shape
            .as_mut()
            .unwrap()
            .set_deferred("disabled", &true.to_variant());
    }

    pub fn place_object(
        &mut self,
        object_layer: Gd<BuildingLayer>,
        walls_layer: Gd<BuildingWallsLayer>,
        structure: Gd<Structure>,
        structure_index: u32,
        origin: Vector2i,
        rotation: Direction,
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

    pub fn destroy_with_layer_cleanup(&mut self) {
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

        self.destroy();
    }

    pub fn destroy(&mut self) {
        self.disconnect_building_system();

        self.disable_collision();

        let self_gd = self.to_gd();
        if let Some(pool) = &mut self.pool {
            pool.bind_mut().return_to_pool(self_gd);
        } else {
            self.base_mut().queue_free();
        }
    }

    // Refactor?: ideally this object shouldn't know about the BuildingSystem. But to do the same in the BuildingSystem
    // we will need to create signals in this class to that are emitted when static_body.mouse_entered/exited triggers
    // and still be able to pass the self object (which right now is giving me an error)
    pub fn connect_building_system(&mut self, building_system: &mut Gd<BuildingSystem>) {
        // Link signals
        let structure = self.to_gd().clone();
        let static_body = self.static_body.as_ref().unwrap();
        let mouse_entered_handle =
            static_body
                .signals()
                .mouse_entered()
                .connect_other(building_system, move |this| {
                    this.on_mouse_enter_placed_structure(structure.clone());
                });

        let structure = self.to_gd().clone();
        let static_body = self.static_body.as_ref().unwrap();
        let mouse_exited_handle =
            static_body
                .signals()
                .mouse_exited()
                .connect_other(building_system, move |this| {
                    this.on_mouse_exit_placed_structure(structure.clone());
                });

        // Store connect handles to disconnect if structure gets destroyed
        self.connect_handles = Some([mouse_entered_handle, mouse_exited_handle]);
    }

    fn disconnect_building_system(&mut self) {
        if let Some(connect_handles) = self.connect_handles.take() {
            for handle in connect_handles.into_iter() {
                handle.disconnect();
            }
        }
    }
}

impl StructureInstance {
    pub fn structure_variant(&self) -> StructureVariant {
        let Some(structure) = self.structure.as_ref() else {
            error!("no structure variant set");
            panic!();
        };

        structure.bind().variant
    }

    pub fn origin(&self) -> Vector2i {
        self.origin
    }

    pub fn player_positions_and_directions_in_table(&self) -> [(Vector2, Direction); 2] {
        assert!(self.structure_variant() == StructureVariant::Table);

        match self.object_rotation {
            Direction::Up => {
                let player_0_pos = self.origin.cast_float() + Vector2::new(1.0, 0.0);
                let player_1_pos = player_0_pos + Vector2::new(0.0, 3.0);
                [
                    (player_0_pos, Direction::Up),
                    (player_1_pos, Direction::Down),
                ]
            }

            _ => [(Vector2::ZERO, Direction::Up); 2],
        }
    }

    pub fn placing_position(&self) -> Vector2 {
        let offset = if self.structure_variant().is_in_tile() {
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
        if self.structure_variant().is_in_tile() {
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
