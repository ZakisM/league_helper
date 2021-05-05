use lcu_driver::endpoints::champ_select::MySelection;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SummonerSpells {
    pub first: isize,
    pub second: isize,
}

impl SummonerSpells {
    pub fn to_my_selection(&self, selected_skin_id: isize, ward_skin_id: isize) -> MySelection {
        MySelection {
            selected_skin_id,
            spell_1_id: self.first,
            spell_2_id: self.second,
            ward_skin_id,
        }
    }
}
