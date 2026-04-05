use super::*;

use serde::Deserialize;
use serde::Serialize;

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub(super) struct StructureRotationSerde(pub u8);

#[derive(Serialize, Deserialize, Debug)]
pub(super) struct PlacedStructureSerde {
    pub index: u32,
    pub rotation: StructureRotationSerde,
    pub origin: (i32, i32),
}

#[derive(Serialize, Deserialize, Debug)]
pub(super) struct BuildingLayerSerde {
    pub structures: Vec<PlacedStructureSerde>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(super) struct BuildingMapSerde {
    pub layer_ground: BuildingLayerSerde,
    pub layer_objects: BuildingLayerSerde,
}

impl BuildingMapSerde {
    pub fn new(layer_ground: &Gd<BuildingLayer>, layer_objects: &Gd<BuildingLayer>) -> Self {
        Self {
            layer_ground: layer_ground.into(),
            layer_objects: layer_objects.into(),
        }
    }
}
