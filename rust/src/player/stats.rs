type Stat = u16;

struct PlayerStats {
    technique: TechniqueStats,
    physical: PhysicalStats,
    mental: MentalStats,
}

struct TechniqueStats {
    core: TechniqueCoreStats,
    serve: TechniqueServeStats,
}

struct TechniqueCoreStats {
    short_game: Stat,
    long_game: Stat,
    looping: Stat,
    blocking: Stat,
    smash: Stat,
}

struct TechniqueServeStats {
    spin: Stat,
    accuracy: Stat,
    deception: Stat,
}

struct PhysicalStats {
    movement: Stat,
    stamina: Stat,
    reflexes: Stat,
}

struct MentalStats {
    motivation: Stat,
    discipline: Stat,
    confidence: Stat,
    composure: Stat,
    game_sense: Stat,
}
