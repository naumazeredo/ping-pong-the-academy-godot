pub struct Attribute {
    pub value: u16,
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

pub struct PlayerAttributes {
    pub technique: TechniqueAttributes,
    pub physical: PhysicalAttributes,
    pub mental: MentalAttributes,
}

pub struct TechniqueAttributes {
    pub core: TechniqueCoreAttributes,
    pub serve: TechniqueServeAttributes,
}

pub struct TechniqueCoreAttributes {
    pub short_game: Attribute,
    pub long_game: Attribute,
    pub looping: Attribute,
    pub blocking: Attribute,
    pub smash: Attribute,
}

pub struct TechniqueServeAttributes {
    pub spin: Attribute,
    pub accuracy: Attribute,
    pub deception: Attribute,
}

pub struct PhysicalAttributes {
    pub movement: Attribute,
    pub stamina: Attribute,
    pub reflexes: Attribute,
}

pub struct MentalAttributes {
    pub motivation: Attribute,
    pub discipline: Attribute,
    pub confidence: Attribute,
    pub composure: Attribute,
    pub game_sense: Attribute,
}

impl PlayerAttributes {
    pub const BASE: Self = Self {
        technique: TechniqueAttributes {
            core: TechniqueCoreAttributes {
                short_game: Attribute::BASE,
                long_game: Attribute::BASE,
                looping: Attribute::BASE,
                blocking: Attribute::BASE,
                smash: Attribute::BASE,
            },
            serve: TechniqueServeAttributes {
                spin: Attribute::BASE,
                accuracy: Attribute::BASE,
                deception: Attribute::BASE,
            },
        },
        physical: PhysicalAttributes {
            movement: Attribute::BASE,
            stamina: Attribute::BASE,
            reflexes: Attribute::BASE,
        },
        mental: MentalAttributes {
            motivation: Attribute::BASE,
            discipline: Attribute::BASE,
            confidence: Attribute::BASE,
            composure: Attribute::BASE,
            game_sense: Attribute::BASE,
        },
    };
}
