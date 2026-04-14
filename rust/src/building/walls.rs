use super::*;

use godot::classes::*;
use godot::prelude::*;

use std::collections::BTreeMap;
use std::collections::BTreeSet;

#[derive(GodotClass)]
#[class(tool, init, base=Resource)]
pub(super) struct WallStructure {
    #[export]
    pub pillar: Option<Gd<PackedScene>>,

    #[export]
    pub model: Option<Gd<PackedScene>>,
}

#[derive(GodotClass)]
#[class(no_init, base=Node3D)]
pub(super) struct PlacedWallStructure {
    layer: Gd<BuildingWallsLayer>,
    structure: Gd<WallStructure>,
    index: u32,
    direction: Option<WallDirection>,
    origin: Vector2i,
    model: Gd<Node3D>,

    base: Base<Node3D>,
}

impl PlacedWallStructure {
    pub fn new(
        layer: Gd<BuildingWallsLayer>,
        structure: Gd<WallStructure>,
        index: u32,
        direction: Option<WallDirection>,
        origin: Vector2i,
        mut model: Gd<Node3D>,
    ) -> Gd<Self> {
        let mut placed = Gd::from_init_fn(|base| Self {
            layer,
            structure,
            index,
            direction,
            origin,
            model: model.clone(),
            base,
        });

        if direction.is_none() {
            placed.set_name(&format!("placed_pillar_{}", model.get_name()));
        } else {
            placed.set_name(&format!("placed_{}", model.get_name()));
        }

        model.reparent(&placed);
        model.set_position(Vector3::ZERO);
        model.set_rotation_degrees(Vector3::ZERO);
        placed
    }

    /*
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
    */

    pub fn is_pillar(&self) -> bool {
        self.direction.is_none()
    }
}

#[derive(GodotConvert, Var, Export, Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
#[godot(via = i8)]
pub enum WallDirection {
    Horizontal,
    Vertical,
}

#[derive(GodotClass)]
#[class(init, base=Node3D)]
pub(super) struct BuildingWallsLayer {
    #[export]
    pub structures: Array<Gd<WallStructure>>,

    placed_wall_structures: BTreeMap<(Vector2i, WallDirection), Gd<PlacedWallStructure>>,
    placed_pillar_structures: BTreeMap<Vector2i, Gd<PlacedWallStructure>>,

    // This is updated when we place objects
    blocked_corners: BTreeSet<Vector2i>,

    pillar_pools: Vec<Gd<ObjectPool>>,
    wall_pools: Vec<Gd<ObjectPool>>,

    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for BuildingWallsLayer {
    fn ready(&mut self) {
        // Create object pools
        let mut self_gd = self.to_gd();
        self.pillar_pools.reserve_exact(self.structures.len());
        self.wall_pools.reserve_exact(self.structures.len());
        for structure in self.structures.iter_shared() {
            macro_rules! add_pool {
                ($pools:ident, $model:ident) => {
                    let pool = ObjectPool::create(structure.bind().$model.clone().unwrap());
                    self_gd.add_child(&pool);
                    self.$pools.push(pool);
                };
            }

            add_pool!(pillar_pools, pillar);
            add_pool!(wall_pools, model);
        }
    }
}

// Utils
impl BuildingWallsLayer {
    pub fn real_end_corner(start_corner: Vector2i, end_corner: Vector2i) -> Vector2i {
        if start_corner.x == end_corner.x || start_corner.y == end_corner.y {
            return end_corner;
        }

        if start_corner.x.abs_diff(end_corner.x) >= start_corner.y.abs_diff(end_corner.y) {
            Vector2i::new(end_corner.x, start_corner.y)
        } else {
            Vector2i::new(start_corner.x, end_corner.y)
        }
    }

    pub fn wall_start_corner(corner_0: Vector2i, corner_1: Vector2i) -> Vector2i {
        corner_0.min(corner_1)
    }

    pub fn wall_rotation(corner_0: Vector2i, corner_1: Vector2i) -> Vector3 {
        match BuildingWallsLayer::wall_direction(corner_0, corner_1) {
            WallDirection::Vertical => Vector3::ZERO,
            WallDirection::Horizontal => Vector3::new(0.0, 90.0, 0.0),
        }
    }

    pub fn wall_direction(corner_0: Vector2i, corner_1: Vector2i) -> WallDirection {
        if corner_0.x == corner_1.x {
            WallDirection::Vertical
        } else {
            WallDirection::Horizontal
        }
    }

    pub fn is_corner_available(&self, corner: Vector2i) -> bool {
        let blocked = self
            .placed_wall_structures
            .contains_key(&(corner, WallDirection::Horizontal))
            || self
                .placed_wall_structures
                .contains_key(&(corner + Vector2i::LEFT, WallDirection::Horizontal))
            || self
                .placed_wall_structures
                .contains_key(&(corner, WallDirection::Vertical))
            || self
                .placed_wall_structures
                .contains_key(&(corner + Vector2i::UP, WallDirection::Vertical))
            || self.placed_pillar_structures.contains_key(&corner);

        !blocked
    }
}

// Structure, instancing and pooling
impl BuildingWallsLayer {
    pub fn get_structure(&self, structure_index: u32) -> Option<Gd<WallStructure>> {
        self.structures.get(structure_index as usize)
    }

    pub fn get_or_instantiate_model(
        &mut self,
        structure_index: u32,
        is_pillar: bool,
    ) -> Option<Gd<Node3D>> {
        let model = self
            .get_pool(structure_index, is_pillar)
            .bind_mut()
            .get_or_instantiate();

        Some(model)
    }

    pub fn return_to_pool<T: Inherits<Node3D>>(
        &mut self,
        object: Gd<T>,
        structure_index: u32,
        is_pillar: bool,
    ) {
        self.get_pool(structure_index, is_pillar)
            .bind_mut()
            .return_to_pool(object.upcast());
    }

    fn get_pool(&mut self, structure_index: u32, is_pillar: bool) -> Gd<ObjectPool> {
        if is_pillar {
            self.pillar_pools[structure_index as usize].clone()
        } else {
            self.wall_pools[structure_index as usize].clone()
        }
    }
}

// Placing
impl BuildingWallsLayer {
    fn can_place_no_end_check(&self, start_corner: Vector2i, end_corner: Vector2i) -> bool {
        let corner_iter = CornerIter::new(start_corner, end_corner);
        for corner in corner_iter.into_iter() {
            if self.blocked_corners.contains(&corner) {
                return false;
            }
        }

        true
    }

    pub fn can_place(&self, start_corner: Vector2i, end_corner: Vector2i) -> bool {
        let end_corner = Self::real_end_corner(start_corner, end_corner);
        self.can_place_no_end_check(start_corner, end_corner)
    }

    pub fn try_place_from_preview(
        &mut self,
        structure_index: u32,
        start_corner: Vector2i,
        end_corner: Vector2i,
        models: &Vec<Gd<Node3D>>,
    ) -> Option<Vec<Gd<PlacedWallStructure>>> {
        let end_corner = Self::real_end_corner(start_corner, end_corner);
        if !self.can_place_no_end_check(start_corner, end_corner) {
            return None;
        }

        let mut placed_wall_structures = Vec::new();

        if start_corner == end_corner {
            assert!(models.len() == 1);

            let instantiated_model = models[0].clone();

            let Some(structure) = self.get_structure(structure_index) else {
                return None;
            };

            let mut placed_structure = PlacedWallStructure::new(
                self.to_gd(),
                structure,
                structure_index,
                None,
                start_corner,
                instantiated_model,
            );

            let position = grid_cell_to_global(start_corner);
            placed_structure.set_position(position);

            self.placed_pillar_structures
                .insert(start_corner, placed_structure.clone());

            self.base_mut().add_child(&placed_structure);

            godot_print!(
                "placed pillar structures: {}",
                self.placed_pillar_structures.len()
            );

            placed_wall_structures.push(placed_structure);
        } else {
            // Refactor: this code is a close copy to the `BuildingSystem::update_placing_walls`. We should be able
            // to unify them in some way

            let corner_iter = CornerIter::new(start_corner, end_corner);
            // XXX: `windows` is not implemented for iterators for some reason
            let corners: Vec<_> = corner_iter.collect();

            assert!(models.len() + 1 == corners.len());

            let Some(structure) = self.get_structure(structure_index) else {
                return None;
            };

            placed_wall_structures.reserve_exact(models.len());

            for (window, instantiated_model) in corners.windows(2).zip(models.iter()) {
                let [corner_0, corner_1] = *window else {
                    unreachable!()
                };

                let wall_direction = Self::wall_direction(corner_0, corner_1);
                let wall_start_corner = Self::wall_start_corner(corner_0, corner_1);

                let mut placed_structure = PlacedWallStructure::new(
                    self.to_gd(),
                    structure.clone(),
                    structure_index,
                    Some(wall_direction),
                    wall_start_corner,
                    instantiated_model.clone(),
                );

                let position = grid_cell_to_global(wall_start_corner);
                placed_structure.set_position(position);
                placed_structure.set_rotation_degrees(Self::wall_rotation(corner_0, corner_1));

                self.placed_wall_structures.insert(
                    (wall_start_corner, wall_direction),
                    placed_structure.clone(),
                );

                self.base_mut().add_child(&placed_structure);
                placed_wall_structures.push(placed_structure);
            }

            godot_print!(
                "placed wall structures: {}",
                self.placed_wall_structures.len()
            );
        }

        Some(placed_wall_structures)
    }
}

// Object placing
impl BuildingWallsLayer {
    pub fn block_corner(&mut self, corner: Vector2i) {
        self.blocked_corners.insert(corner);
    }

    pub fn free_corner(&mut self, corner: Vector2i) {
        self.blocked_corners.remove(&corner);
    }
}

// Utils
// TODO: allow L-shaped iterations? `initial_direction: WallDirection`
pub struct CornerIter {
    current: Vector2i,
    end: Vector2i,
    has_ended: bool,
}

impl CornerIter {
    pub fn new(start: Vector2i, end: Vector2i) -> Self {
        Self {
            current: start,
            end,
            has_ended: false,
        }
    }
}

impl Iterator for CornerIter {
    type Item = Vector2i;

    fn next(&mut self) -> Option<Self::Item> {
        if self.has_ended {
            return None;
        }

        let ret = self.current;
        if self.current == self.end {
            self.has_ended = true;
        } else {
            if self.current.x.abs_diff(self.end.x) >= self.current.y.abs_diff(self.end.y) {
                // Moving on the x axis
                let dir = (self.end.x - self.current.x).signum();
                self.current.x += dir;
            } else {
                // Moving on the y axis
                let dir = (self.end.y - self.current.y).signum();
                self.current.y += dir;
            }
        }

        Some(ret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corner_iter_same() {
        let start = Vector2i::new(0, 0);
        let end = Vector2i::new(0, 0);
        let mut it = CornerIter::new(start, end);
        assert_eq!(it.next(), Some(Vector2i::new(0, 0)));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn corner_iter_positive_x() {
        let start = Vector2i::new(0, 0);
        let end = Vector2i::new(5, 0);
        let mut it = CornerIter::new(start, end);
        assert_eq!(it.next(), Some(Vector2i::new(0, 0)));
        assert_eq!(it.next(), Some(Vector2i::new(1, 0)));
        assert_eq!(it.next(), Some(Vector2i::new(2, 0)));
        assert_eq!(it.next(), Some(Vector2i::new(3, 0)));
        assert_eq!(it.next(), Some(Vector2i::new(4, 0)));
        assert_eq!(it.next(), Some(Vector2i::new(5, 0)));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn corner_iter_negative_x() {
        let start = Vector2i::new(0, 0);
        let end = Vector2i::new(-5, 0);
        let mut it = CornerIter::new(start, end);
        assert_eq!(it.next(), Some(Vector2i::new(0, 0)));
        assert_eq!(it.next(), Some(Vector2i::new(-1, 0)));
        assert_eq!(it.next(), Some(Vector2i::new(-2, 0)));
        assert_eq!(it.next(), Some(Vector2i::new(-3, 0)));
        assert_eq!(it.next(), Some(Vector2i::new(-4, 0)));
        assert_eq!(it.next(), Some(Vector2i::new(-5, 0)));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn corner_iter_positive_y() {
        let start = Vector2i::new(0, 0);
        let end = Vector2i::new(0, 5);
        let mut it = CornerIter::new(start, end);
        assert_eq!(it.next(), Some(Vector2i::new(0, 0)));
        assert_eq!(it.next(), Some(Vector2i::new(0, 1)));
        assert_eq!(it.next(), Some(Vector2i::new(0, 2)));
        assert_eq!(it.next(), Some(Vector2i::new(0, 3)));
        assert_eq!(it.next(), Some(Vector2i::new(0, 4)));
        assert_eq!(it.next(), Some(Vector2i::new(0, 5)));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn corner_iter_negative_y() {
        let start = Vector2i::new(0, 0);
        let end = Vector2i::new(0, -5);
        let mut it = CornerIter::new(start, end);
        assert_eq!(it.next(), Some(Vector2i::new(0, 0)));
        assert_eq!(it.next(), Some(Vector2i::new(0, -1)));
        assert_eq!(it.next(), Some(Vector2i::new(0, -2)));
        assert_eq!(it.next(), Some(Vector2i::new(0, -3)));
        assert_eq!(it.next(), Some(Vector2i::new(0, -4)));
        assert_eq!(it.next(), Some(Vector2i::new(0, -5)));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn corner_iter_positive_xy() {
        let start = Vector2i::new(0, 0);
        let end = Vector2i::new(3, 5);
        let mut it = CornerIter::new(start, end);
        assert_eq!(it.next(), Some(Vector2i::new(0, 0)));
        assert_eq!(it.next(), Some(Vector2i::new(0, 1)));
        assert_eq!(it.next(), Some(Vector2i::new(0, 2)));
        assert_eq!(it.next(), Some(Vector2i::new(1, 2)));
        assert_eq!(it.next(), Some(Vector2i::new(1, 3)));
        assert_eq!(it.next(), Some(Vector2i::new(2, 3)));
        assert_eq!(it.next(), Some(Vector2i::new(2, 4)));
        assert_eq!(it.next(), Some(Vector2i::new(3, 4)));
        assert_eq!(it.next(), Some(Vector2i::new(3, 5)));
        assert_eq!(it.next(), None);
    }
}
