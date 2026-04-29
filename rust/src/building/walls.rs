use super::*;

use godot::classes::*;
use godot::prelude::*;

use std::collections::BTreeMap;
use std::collections::BTreeSet;

#[derive(GodotConvert, Var, Export, Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
#[godot(via = i8)]
pub enum WallDirection {
    Horizontal,
    Vertical,
}

impl WallDirection {
    pub fn as_vector2(self) -> Vector2 {
        match self {
            Self::Horizontal => Vector2::RIGHT,
            Self::Vertical => Vector2::DOWN,
        }
    }
}

impl From<WallDirection> for StructureWallDirectionSerde {
    fn from(value: WallDirection) -> Self {
        let v = match value {
            WallDirection::Horizontal => 0,
            WallDirection::Vertical => 1,
        };

        Self(v)
    }
}

impl From<StructureWallDirectionSerde> for WallDirection {
    fn from(value: StructureWallDirectionSerde) -> Self {
        match value.0 {
            0 => WallDirection::Horizontal,
            _ => WallDirection::Vertical,
        }
    }
}

#[derive(GodotClass)]
#[class(init, base=Node3D)]
pub(super) struct BuildingWallsLayer {
    #[export]
    pub structures: Array<Gd<Structure>>,

    placed_wall_structures_h: BTreeMap<Vector2i, Gd<StructureInstance>>,
    placed_wall_structures_v: BTreeMap<Vector2i, Gd<StructureInstance>>,
    placed_pillar_structures: BTreeMap<Vector2i, Gd<StructureInstance>>,

    // This is updated when we place objects
    blocked_corners: BTreeSet<Vector2i>,

    #[export]
    object_pools: Option<Gd<ObjectPools>>,

    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for BuildingWallsLayer {
    fn ready(&mut self) {
        for structure in self.structures.iter_shared() {
            self.object_pools
                .as_mut()
                .unwrap()
                .bind_mut()
                .get_or_create_pool(structure.bind().model.clone().unwrap());

            self.object_pools
                .as_mut()
                .unwrap()
                .bind_mut()
                .get_or_create_pool(structure.bind().wall_pillar.clone().unwrap());
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
        let blocked = self.placed_wall_structures_h.contains_key(&corner)
            || self
                .placed_wall_structures_h
                .contains_key(&(corner + Vector2i::LEFT))
            || self.placed_wall_structures_v.contains_key(&corner)
            || self
                .placed_wall_structures_v
                .contains_key(&(corner + Vector2i::UP))
            || self.placed_pillar_structures.contains_key(&corner);

        !blocked
    }

    pub fn get_end_corner(
        start_corner: Vector2i,
        wall_direction: Option<WallDirection>,
    ) -> Vector2i {
        match wall_direction {
            Some(WallDirection::Horizontal) => start_corner + Vector2i::RIGHT,
            Some(WallDirection::Vertical) => start_corner + Vector2i::DOWN,
            None => start_corner,
        }
    }
}

// Structure, instancing and pooling
impl BuildingWallsLayer {
    pub fn get_structure(&self, structure_index: u32) -> Option<Gd<Structure>> {
        self.structures.get(structure_index as usize)
    }

    pub fn get_or_instantiate_model(
        &mut self,
        structure_index: u32,
        is_pillar: bool,
    ) -> Option<Gd<StructureInstance>> {
        self.structures
            .at(structure_index as usize)
            .bind()
            .instantiate_wall(is_pillar, self.object_pools.as_mut().unwrap())
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

    pub fn create_wall_structures(
        &mut self,
        structure_index: u32,
        start_corner: Vector2i,
        end_corner: Vector2i,
        is_pillar_out: Option<&mut bool>,
        keep_global_position: bool,
    ) -> Option<Vec<Gd<StructureInstance>>> {
        let is_pillar = start_corner == end_corner;
        if let Some(v) = is_pillar_out {
            *v = is_pillar;
        }

        let offset = if keep_global_position {
            Vector2i::ZERO
        } else {
            start_corner
        };

        if is_pillar {
            let mut model =
                self.get_or_instantiate_model(structure_index, true /* is_pillar */)?;

            model.set_position(grid_cell_to_global(start_corner - offset));
            model.set_rotation_degrees(Vector3::ZERO);

            return Some(vec![model]);
        }

        // If not a pillar
        let mut placed_structures = Vec::new();

        // Create new walls
        let corner_iter = CornerIter::new(start_corner, end_corner);

        // XXX: `windows` is not implemented for iterators for some reason
        let corners: Vec<_> = corner_iter.collect();

        placed_structures.reserve_exact(corners.len().saturating_sub(1));

        for window in corners.windows(2) {
            let [corner_0, corner_1] = *window else {
                unreachable!()
            };

            let Some(mut model) =
                self.get_or_instantiate_model(structure_index, false /* is_pillar */)
            else {
                unreachable!()
            };

            let corner = BuildingWallsLayer::wall_start_corner(corner_0, corner_1);
            model.set_position(grid_cell_to_global(corner - offset));
            model.set_rotation_degrees(BuildingWallsLayer::wall_rotation(corner_0, corner_1));

            placed_structures.push(model);
        }

        Some(placed_structures)
    }

    pub fn try_place(
        &mut self,
        structure_index: u32,
        start_corner: Vector2i,
        end_corner: Vector2i,
    ) -> Option<Vec<Gd<StructureInstance>>> {
        let models = self.create_wall_structures(
            structure_index,
            start_corner,
            end_corner,
            None,
            true, /* keep_global_position */
        )?;
        self.try_place_from_preview(structure_index, start_corner, end_corner, &models)
    }

    pub fn try_place_from_preview(
        &mut self,
        structure_index: u32,
        start_corner: Vector2i,
        end_corner: Vector2i,
        models: &[Gd<StructureInstance>],
    ) -> Option<Vec<Gd<StructureInstance>>> {
        let end_corner = Self::real_end_corner(start_corner, end_corner);
        if !self.can_place_no_end_check(start_corner, end_corner) {
            return None;
        }

        let structure = self.get_structure(structure_index)?;

        let mut placed_wall_structures = Vec::new();

        if start_corner == end_corner {
            // Ignore placement if there are other walls/pillars in the same position
            if !self.is_corner_available(start_corner) {
                return None;
            }

            assert!(models.len() == 1);

            let instantiated_model = models[0].clone();

            let mut placed_structure = instantiated_model.cast::<StructureInstance>();
            placed_structure.bind_mut().place_wall(
                self.to_gd(),
                structure,
                structure_index,
                start_corner,
                None,
            );

            placed_structure.reparent(&self.to_gd());
            placed_structure.set_position(grid_cell_to_global(start_corner));

            self.placed_pillar_structures
                .insert(start_corner, placed_structure.clone());

            placed_wall_structures.push(placed_structure);
        } else {
            // Refactor: this code is close to `create_wall_structures`. Is there a way to unify them?

            let corner_iter = CornerIter::new(start_corner, end_corner);
            // XXX: `windows` is not implemented for iterators for some reason
            let corners: Vec<_> = corner_iter.collect();

            assert!(models.len() + 1 == corners.len());

            placed_wall_structures.reserve_exact(models.len());

            for (window, instantiated_model) in corners.windows(2).zip(models.iter()) {
                let [corner_0, corner_1] = *window else {
                    unreachable!()
                };

                let wall_direction = Self::wall_direction(corner_0, corner_1);
                let wall_start_corner = Self::wall_start_corner(corner_0, corner_1);

                let mut placed_structure = instantiated_model.clone().cast::<StructureInstance>();
                placed_structure.bind_mut().place_wall(
                    self.to_gd(),
                    structure.clone(),
                    structure_index,
                    wall_start_corner,
                    Some(wall_direction),
                );

                placed_structure.reparent(&self.to_gd());
                placed_structure.set_position(grid_cell_to_global(wall_start_corner));
                placed_structure.set_rotation_degrees(Self::wall_rotation(corner_0, corner_1));

                let old_wall = if let WallDirection::Horizontal = wall_direction {
                    self.placed_wall_structures_h
                        .insert(wall_start_corner, placed_structure.clone())
                } else {
                    self.placed_wall_structures_v
                        .insert(wall_start_corner, placed_structure.clone())
                };

                // Return old wall to the pool
                if let Some(mut old_wall) = old_wall {
                    old_wall.bind_mut().destroy_with_layer_cleanup();
                }

                // Remove pillar if any
                self.remove_placed_structure_at(wall_start_corner, None);

                placed_wall_structures.push(placed_structure);
            }

            // Remove last pillar if any
            self.remove_placed_structure_at(end_corner, None);
        }

        Some(placed_wall_structures)
    }

    pub fn remove_placed_structure_at(
        &mut self,
        start_corner: Vector2i,
        wall_direction: Option<WallDirection>,
    ) {
        match wall_direction {
            Some(WallDirection::Horizontal) => self.placed_wall_structures_h.remove(&start_corner),
            Some(WallDirection::Vertical) => self.placed_wall_structures_v.remove(&start_corner),
            None => self.placed_pillar_structures.remove(&start_corner),
        };
    }

    pub fn clear(&mut self) {
        macro_rules! clear_container {
            ($container:ident) => {
                let mut placed_structures = std::mem::take(&mut self.$container);
                for placed_structure in placed_structures.values_mut() {
                    placed_structure.bind_mut().destroy();
                }
            };
        }

        clear_container!(placed_wall_structures_h);
        clear_container!(placed_wall_structures_v);
        clear_container!(placed_pillar_structures);
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

// Serialization
impl From<&Gd<BuildingWallsLayer>> for BuildingWallsLayerSerde {
    fn from(value: &Gd<BuildingWallsLayer>) -> Self {
        let mut walls = Vec::new();
        for structure in value.bind().placed_wall_structures_h.values() {
            walls.push(structure.into());
        }

        for structure in value.bind().placed_wall_structures_v.values() {
            walls.push(structure.into());
        }

        let mut pillars = Vec::new();
        for structure in value.bind().placed_pillar_structures.values() {
            pillars.push(structure.into());
        }

        Self { walls, pillars }
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
