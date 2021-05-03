use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SummonerSpells {
    pub first: isize,
    pub second: isize,
}
