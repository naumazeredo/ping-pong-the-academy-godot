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

    pub fn position_offset(&self, structure_size: Vector2i) -> Vector3 {
        let (x, z) = match self {
            StructureRotation::Up => (0.0, 0.0),
            StructureRotation::Right => (0.0, 1.0),
            StructureRotation::Down => (1.0, 1.0),
            StructureRotation::Left => (1.0, 0.0),
        };

        Vector3::new(
            x * structure_size.y as f32,
            0.0,
            z * structure_size.x as f32,
        )
    }
}

#[derive(GodotClass)]
#[class(tool, init, base=Resource)]
pub(super) struct Structure {
    #[export]
    pub model: Option<Gd<PackedScene>>,

    #[export]
    #[init(val = Vector2i::new(1, 1))]
    pub size: Vector2i,

    #[export]
    pub rotations: StructureRotations,
}

impl Structure {
    pub fn iter_cells(&self, origin: Vector2i, rotation: StructureRotation) -> StructureCellsIter {
        StructureCellsIter::new(origin, self.rotated_size(rotation))
    }

    pub fn rotated_size(&self, rotation: StructureRotation) -> Vector2i {
        match rotation {
            StructureRotation::Up | StructureRotation::Down => self.size,
            StructureRotation::Right | StructureRotation::Left => {
                Vector2i::new(self.size.y, self.size.x)
            }
        }
    }

    pub fn rotate(&self, current_rotation: &mut StructureRotation) {
        *current_rotation = match self.rotations {
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

    pub fn try_instantiate(&self) -> Option<Gd<Node3D>> {
        self.model
            .clone()
            .and_then(|model| model.try_instantiate_as::<Node3D>())
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
