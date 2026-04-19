use super::*;

use serde::Deserialize;
use serde::Serialize;

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub(super) struct StructureRotationSerde(pub u8);

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub(super) struct StructureWallDirectionSerde(pub u8);

#[derive(Serialize, Deserialize, Debug)]
pub(super) struct PlacedStructureSerde {
    pub index: u32,
    pub rotation: Option<StructureRotationSerde>,
    pub direction: Option<StructureWallDirectionSerde>,
    pub origin: (i32, i32),
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub(super) struct BuildingLayerSerde {
    pub structures: Vec<PlacedStructureSerde>,
}

impl BuildingLayerSerde {
    fn is_empty(&self) -> bool {
        self.structures.is_empty()
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub(super) struct BuildingWallsLayerSerde {
    pub walls: Vec<PlacedStructureSerde>,
    pub pillars: Vec<PlacedStructureSerde>,
}

impl BuildingWallsLayerSerde {
    fn is_empty(&self) -> bool {
        self.walls.is_empty() && self.pillars.is_empty()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub(super) struct BuildingMapSerde {
    #[serde(default, skip_serializing_if = "BuildingLayerSerde::is_empty")]
    pub layer_ground: BuildingLayerSerde,
    #[serde(default, skip_serializing_if = "BuildingLayerSerde::is_empty")]
    pub layer_objects: BuildingLayerSerde,
    #[serde(default, skip_serializing_if = "BuildingWallsLayerSerde::is_empty")]
    pub layer_walls: BuildingWallsLayerSerde,
}

impl BuildingMapSerde {
    pub fn new(
        layer_ground: &Gd<BuildingLayer>,
        layer_objects: &Gd<BuildingLayer>,
        layer_walls: &Gd<BuildingWallsLayer>,
    ) -> Self {
        Self {
            layer_ground: layer_ground.into(),
            layer_objects: layer_objects.into(),
            layer_walls: layer_walls.into(),
        }
    }
}
