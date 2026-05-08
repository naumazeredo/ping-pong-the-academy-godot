use godot::prelude::*;

#[derive(GodotConvert, Var, Export, Default, Copy, Clone, Debug)]
#[godot(via = i8)]
pub enum Direction {
    #[default]
    Up,
    Right,
    Down,
    Left,
}

impl From<Direction> for DirectionSerde {
    fn from(value: Direction) -> Self {
        let v = match value {
            Direction::Up => 0,
            Direction::Right => 1,
            Direction::Down => 2,
            Direction::Left => 3,
        };

        Self(v)
    }
}

impl From<DirectionSerde> for Direction {
    fn from(value: DirectionSerde) -> Self {
        match value.0 {
            0 => Direction::Up,
            1 => Direction::Right,
            2 => Direction::Down,
            _ => Direction::Left,
        }
    }
}

impl Direction {
    pub fn to_degrees(self) -> f32 {
        match self {
            Direction::Up => 0.0,
            Direction::Right => 90.0,
            Direction::Down => 180.0,
            Direction::Left => 270.0,
        }
    }

    pub fn to_degrees_vector(self) -> Vector3 {
        Vector3::new(0.0, self.to_degrees(), 0.0)
    }

    pub fn position_offset(&self, structure_size: Vector2i) -> Vector2 {
        let (x, y) = match self {
            Direction::Up => (0.0, 0.0),
            Direction::Right => (0.0, 1.0),
            Direction::Down => (1.0, 1.0),
            Direction::Left => (1.0, 0.0),
        };

        Vector2::new(x * structure_size.y as f32, y * structure_size.x as f32)
    }

    pub fn position_offset_3d(&self, structure_size: Vector2i) -> Vector3 {
        let offset = self.position_offset(structure_size);
        Vector3::new(offset.x, 0.0, offset.y)
    }
}

// Serialization
use serde::Deserialize;
use serde::Serialize;

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub struct DirectionSerde(pub u8);
