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

#[derive(GodotConvert, Var, Export, Default, Copy, Clone, Debug)]
#[godot(via = i8)]
pub(super) enum StructureRotation {
    #[default]
    Up,
    Right,
    Down,
    Left,
}

impl From<StructureRotation> for StructureRotationSerde {
    fn from(value: StructureRotation) -> Self {
        let v = match value {
            StructureRotation::Up => 0,
            StructureRotation::Right => 1,
            StructureRotation::Down => 2,
            StructureRotation::Left => 3,
        };

        Self(v)
    }
}

impl From<StructureRotationSerde> for StructureRotation {
    fn from(value: StructureRotationSerde) -> Self {
        match value.0 {
            0 => StructureRotation::Up,
            1 => StructureRotation::Right,
            2 => StructureRotation::Down,
            _ => StructureRotation::Left,
        }
    }
}

impl StructureRotation {
    pub fn degrees(&self) -> Vector3 {
        let y = match self {
            StructureRotation::Up => 0.0,
            StructureRotation::Right => 90.0,
            StructureRotation::Down => 180.0,
            StructureRotation::Left => 270.0,
        };

        Vector3::new(0.0, y, 0.0)
    }

    pub fn position_offset(&self, structure_size: Vector2i) -> Vector2 {
        let (x, y) = match self {
            StructureRotation::Up => (0.0, 0.0),
            StructureRotation::Right => (0.0, 1.0),
            StructureRotation::Down => (1.0, 1.0),
            StructureRotation::Left => (1.0, 0.0),
        };

        Vector2::new(x * structure_size.y as f32, y * structure_size.x as f32)
    }

    pub fn position_offset_3d(&self, structure_size: Vector2i) -> Vector3 {
        let offset = self.position_offset(structure_size);
        Vector3::new(offset.x, 0.0, offset.y)
    }
}

#[derive(GodotConvert, Var, Export, Default, Copy, Clone, Debug, PartialEq)]
#[godot(via = i16)]
pub(super) enum StructureType {
    #[default]
    Floor,
    Table,
    Wall,
}

impl StructureType {
    pub fn is_in_tile(&self) -> bool {
        matches!(self, Self::Floor | Self::Table)
    }

    pub fn is_in_edge(&self) -> bool {
        !self.is_in_tile()
    }
}

#[derive(GodotClass)]
#[class(tool, init, base=Resource)]
pub(super) struct Structure {
    #[export]
    pub type_: StructureType,

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

impl Structure {
    pub fn is_in_tile(&self) -> bool {
        self.type_.is_in_tile()
    }

    pub fn is_in_edge(&self) -> bool {
        self.type_.is_in_edge()
    }
}

impl Structure {
    pub fn iter_cells(&self, origin: Vector2i, rotation: StructureRotation) -> StructureCellsIter {
        assert!(self.is_in_tile());
        StructureCellsIter::new(origin, self.rotated_size(rotation))
    }

    pub fn iter_inner_cells(
        &self,
        origin: Vector2i,
        rotation: StructureRotation,
    ) -> StructureCellsIter {
        assert!(self.is_in_tile());
        StructureCellsIter::new_inner(origin, self.rotated_size(rotation))
    }

    pub fn rotated_size(&self, rotation: StructureRotation) -> Vector2i {
        assert!(self.is_in_tile());

        match rotation {
            StructureRotation::Up | StructureRotation::Down => self.object_size,
            StructureRotation::Right | StructureRotation::Left => {
                Vector2i::new(self.object_size.y, self.object_size.x)
            }
        }
    }

    pub fn rotate(&self, current_rotation: &mut StructureRotation) {
        assert!(self.is_in_tile());

        *current_rotation = match self.object_rotations {
            StructureRotations::OneWay => StructureRotation::Up,
            StructureRotations::TwoWays => match *current_rotation {
                StructureRotation::Up => StructureRotation::Right,
                _ => StructureRotation::Up,
            },
            StructureRotations::FourWays => match *current_rotation {
                StructureRotation::Up => StructureRotation::Right,
                StructureRotation::Right => StructureRotation::Down,
                StructureRotation::Down => StructureRotation::Left,
                StructureRotation::Left => StructureRotation::Up,
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
