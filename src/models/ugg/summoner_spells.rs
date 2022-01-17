use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SummonerSpells {
    pub spell1_id: isize,
    pub spell2_id: isize,
}

impl SummonerSpells {
    pub fn new(first: isize, second: isize) -> Self {
        Self {
            spell1_id: first,
            spell2_id: second,
        }
    }
}
