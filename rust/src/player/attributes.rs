use super::*;

#[derive(Copy, Clone, Debug)]
pub struct Attribute {
    pub value: u16,
    // TODO: add `progression`, `threshold_to_next` and `easiness` to be able to define the attribute progression.
    // e.g.: at lower values, the threshold to next level is 100. If the player has some difficulty with this attribute,
    //       it bumps to 105, if they have ease, it decreases to 95.
    // pub progression: u32,
    // pub threshold_to_next: u32,
    // pub easiness: ProgressionEasiness,
    //
    // pub enum ProgressionEasiness {
    //   VeryFast, // 90%
    //   Fast,     // 95%
    //   Normal,   // 100%
    //   Slow,     // 105%
    //   VerySlow, // 110%
    // }
}

impl Attribute {
    pub const BASE: Self = Self { value: 10 };

    pub fn new(value: u16) -> Self {
        Self { value }
    }

    pub fn improve(&mut self, delta: u16) {
        self.value = self.value.saturating_add(delta);
    }

    pub fn decline(&mut self, delta: u16) {
        self.value = self.value.saturating_sub(delta);
    }
}

impl Default for Attribute {
    fn default() -> Self {
        Self::BASE
    }
}

#[derive(Debug, Default)]
pub struct PlayerAttributes {
    pub technique: TechniqueAttributes,
    pub physical: PhysicalAttributes,
    pub mental: MentalAttributes,
}

#[derive(Debug, Default)]
pub struct TechniqueAttributes {
    pub core: TechniqueCoreAttributes,
    pub serve: TechniqueServeAttributes,
}

#[derive(Debug, Default)]
pub struct TechniqueCoreAttributes {
    pub short_game: Attribute,
    //pub long_game: Attribute,
    pub r#loop: Attribute,
    pub block: Attribute,
    pub smash: Attribute,
}

#[derive(Debug, Default)]
pub struct TechniqueServeAttributes {
    pub spin: Attribute,
    pub accuracy: Attribute,
    pub deception: Attribute,
}

#[derive(Debug, Default)]
pub struct PhysicalAttributes {
    pub movement: Attribute,
    pub stamina: Attribute,
    pub reflexes: Attribute,
}

#[derive(Debug, Default)]
pub struct MentalAttributes {
    pub motivation: Attribute,
    pub discipline: Attribute,
    pub confidence: Attribute,
    pub composure: Attribute,
    pub game_sense: Attribute,
}

impl PlayerAttributes {
    pub fn generate(rng: &mut Gd<RandomNumberGenerator>) -> Self {
        Self {
            technique: TechniqueAttributes {
                core: TechniqueCoreAttributes {
                    short_game: Attribute::new(
                        rng.randfn_ex().mean(10.0).deviation(2.0).done().round() as u16,
                    ),
                    r#loop: Attribute::new(
                        rng.randfn_ex().mean(10.0).deviation(2.0).done().round() as u16,
                    ),
                    block: Attribute::new(
                        rng.randfn_ex().mean(10.0).deviation(2.0).done().round() as u16
                    ),
                    smash: Attribute::new(
                        rng.randfn_ex().mean(10.0).deviation(2.0).done().round() as u16
                    ),
                },
                serve: TechniqueServeAttributes {
                    spin: Attribute::new(
                        rng.randfn_ex().mean(10.0).deviation(2.0).done().round() as u16
                    ),
                    accuracy: Attribute::new(
                        rng.randfn_ex().mean(10.0).deviation(2.0).done().round() as u16,
                    ),
                    deception: Attribute::new(
                        rng.randfn_ex().mean(10.0).deviation(2.0).done().round() as u16,
                    ),
                },
            },
            physical: PhysicalAttributes {
                movement: Attribute::new(
                    rng.randfn_ex().mean(10.0).deviation(2.0).done().round() as u16
                ),
                stamina: Attribute::new(
                    rng.randfn_ex().mean(10.0).deviation(2.0).done().round() as u16
                ),
                reflexes: Attribute::new(
                    rng.randfn_ex().mean(10.0).deviation(2.0).done().round() as u16
                ),
            },
            mental: MentalAttributes {
                motivation: Attribute::new(
                    rng.randfn_ex().mean(10.0).deviation(2.0).done().round() as u16,
                ),
                discipline: Attribute::new(
                    rng.randfn_ex().mean(10.0).deviation(2.0).done().round() as u16,
                ),
                confidence: Attribute::new(
                    rng.randfn_ex().mean(10.0).deviation(2.0).done().round() as u16,
                ),
                composure: Attribute::new(
                    rng.randfn_ex().mean(10.0).deviation(2.0).done().round() as u16
                ),
                game_sense: Attribute::new(
                    rng.randfn_ex().mean(10.0).deviation(2.0).done().round() as u16,
                ),
            },
        }
    }
}
