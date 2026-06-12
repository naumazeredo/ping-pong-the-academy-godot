pub struct PlayerNames {
    male_first_names: StringTable,
    female_first_names: StringTable,
    last_names: StringTable,
}

impl Default for PlayerNames {
    fn default() -> Self {
        Self {
            male_first_names: StringTable::load("-Hugo-".to_owned()),
            female_first_names: StringTable::load("-Bruna-".to_owned()),
            last_names: StringTable::load("-Calderano-\n-Takahashi-".to_owned()),
        }
    }
}

impl PlayerNames {
    pub fn load(
        &mut self,
        male_first_names_list: String,
        female_first_names_list: String,
        last_names_list: String,
    ) {
        self.male_first_names = StringTable::load(male_first_names_list);
        self.female_first_names = StringTable::load(female_first_names_list);
        self.last_names = StringTable::load(last_names_list);
    }

    pub fn get_male_first_name(&self, rng_sample: u32) -> &str {
        self.male_first_names
            .get(rng_sample as usize % self.male_first_names.len())
    }

    pub fn get_female_first_name(&self, rng_sample: u32) -> &str {
        self.female_first_names
            .get(rng_sample as usize % self.female_first_names.len())
    }

    pub fn get_last_name(&self, rng_sample: u32) -> &str {
        self.last_names
            .get(rng_sample as usize % self.last_names.len())
    }

    pub fn len(&self) -> (usize, usize, usize) {
        (
            self.male_first_names.len(),
            self.female_first_names.len(),
            self.last_names.len(),
        )
    }
}

#[derive(Default)]
struct StringTable {
    buffer: String,
    items: Vec<std::ops::Range<usize>>,
}

impl StringTable {
    fn load(text: String) -> Self {
        let mut items = Vec::new();

        let mut start = 0;
        for line in text.lines() {
            let end = start + line.len();
            items.push(start..end);
            start = end + 1;
        }

        Self {
            buffer: text,
            items,
        }
    }

    fn get(&self, index: usize) -> &str {
        let range = self.items[index].clone();
        &self.buffer[range]
    }

    fn len(&self) -> usize {
        self.items.len()
    }
}
