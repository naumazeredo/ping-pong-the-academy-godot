use godot::builtin::*;

pub fn grid_cell_to_global(cell: Vector2i) -> Vector3 {
    Vector3::new(cell.x as f32, 0.0, cell.y as f32)
}
