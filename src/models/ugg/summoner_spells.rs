use lcu_driver::endpoints::champ_select::MySelection;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SummonerSpells {
    pub first: isize,
    pub second: isize,
}

impl From<&SummonerSpells> for MySelection {
    fn from(ss: &SummonerSpells) -> Self {
        Self {
            spell_1_id: ss.first,
            spell_2_id: ss.second,
            ..Self::default()
        }
    }
}
