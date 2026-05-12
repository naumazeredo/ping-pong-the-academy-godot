use super::*;

use godot::classes::*;
use godot::prelude::*;

#[derive(GodotConvert, Var, Export, Default, Copy, Clone)]
#[godot(via = i8)]
pub(super) enum StructureRotations {
    #[default]
    OneWay,
    TwoWays,
    FourWays,
}

#[derive(GodotConvert, Var, Export, Default, Copy, Clone, Debug, PartialEq)]
#[godot(via = i16)]
pub(super) enum StructureVariant {
    #[default]
    Floor,
    Table,
    Wall,
}

impl StructureVariant {
    pub fn is_in_tile(&self) -> bool {
        matches!(self, Self::Floor | Self::Table)
    }

    pub fn is_in_edge(&self) -> bool {
        !self.is_in_tile()
    }
}

#[derive(GodotClass)]
#[class(init, base=Resource)]
pub(super) struct Structure {
    #[export]
    pub variant: StructureVariant,

    #[export]
    pub model: Option<Gd<PackedScene>>,

    #[export_group(name = "Object", prefix = "object_")]
    #[export]
    #[init(val = Vector2i::new(1, 1))]
    pub object_size: Vector2i,

    #[export]
    pub object_rotations: StructureRotations,

    #[export_group(name = "Wall", prefix = "wall_")]
    #[export]
    pub wall_pillar: Option<Gd<PackedScene>>,
}

// Instancing
impl Structure {
    pub fn instantiate(&self, object_pools: &mut Gd<ObjectPools>) -> Option<Gd<StructureInstance>> {
        Some(
            object_pools
                .bind_mut()
                .get_or_create_pool(self.model.as_ref().unwrap().clone())
                .bind_mut()
                .get_or_instantiate(),
        )
    }

    pub fn instantiate_wall(
        &self,
        is_pillar: bool,
        object_pools: &mut Gd<ObjectPools>,
    ) -> Option<Gd<StructureInstance>> {
        let model = if is_pillar {
            self.wall_pillar.as_ref().unwrap().clone()
        } else {
            self.model.as_ref().unwrap().clone()
        };

        Some(
            object_pools
                .bind_mut()
                .get_or_create_pool(model)
                .bind_mut()
                .get_or_instantiate(),
        )
    }
}

impl Structure {
    pub fn is_in_tile(&self) -> bool {
        self.variant.is_in_tile()
    }

    // XXX: to be deleted when we start using it
    #[expect(dead_code)]
    pub fn is_in_edge(&self) -> bool {
        self.variant.is_in_edge()
    }
}

impl Structure {
    pub fn iter_cells(&self, origin: Vector2i, rotation: Direction) -> StructureCellsIter {
        assert!(self.is_in_tile());
        StructureCellsIter::new(origin, self.rotated_size(rotation))
    }

    pub fn iter_inner_cells(&self, origin: Vector2i, rotation: Direction) -> StructureCellsIter {
        assert!(self.is_in_tile());
        StructureCellsIter::new_inner(origin, self.rotated_size(rotation))
    }

    pub fn rotated_size(&self, rotation: Direction) -> Vector2i {
        assert!(self.is_in_tile());

        match rotation {
            Direction::Up | Direction::Down => self.object_size,
            Direction::Right | Direction::Left => {
                Vector2i::new(self.object_size.y, self.object_size.x)
            }
        }
    }

    pub fn rotate(&self, current_rotation: &mut Direction) {
        assert!(self.is_in_tile());

        *current_rotation = match self.object_rotations {
            StructureRotations::OneWay => Direction::Up,
            StructureRotations::TwoWays => match *current_rotation {
                Direction::Up => Direction::Right,
                _ => Direction::Up,
            },
            StructureRotations::FourWays => match *current_rotation {
                Direction::Up => Direction::Right,
                Direction::Right => Direction::Down,
                Direction::Down => Direction::Left,
                Direction::Left => Direction::Up,
            },
        }
    }
}

pub(super) struct StructureCellsIter {
    origin: Vector2i,
    size: Vector2i,

    // Not offset by the origin
    current: Vector2i,
}

impl StructureCellsIter {
    pub fn new(origin: Vector2i, size: Vector2i) -> Self {
        Self {
            origin,
            size,
            current: Vector2i::ZERO,
        }
    }

    pub fn new_inner(origin: Vector2i, size: Vector2i) -> Self {
        Self {
            origin: origin + Vector2i::ONE,
            size: size - Vector2i::ONE,
            current: Vector2i::ZERO,
        }
    }
}

impl Iterator for StructureCellsIter {
    type Item = Vector2i;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.y >= self.size.y {
            return None;
        }

        let ret = self.current + self.origin;

        self.current.x += 1;
        if self.current.x >= self.size.x {
            self.current.x = 0;
            self.current.y += 1;
        }

        Some(ret)
    }
}
