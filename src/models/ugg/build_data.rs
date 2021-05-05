use std::cmp::Ordering;

use float_ord::FloatOrd;
use serde::{Deserialize, Serialize};

use crate::models::ugg::item_set::ItemSet;
use crate::models::ugg::position::Position;
use crate::models::ugg::rune_page::RunePage;
use crate::models::ugg::summoner_spells::SummonerSpells;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BuildData {
    pub position: Position,
    pub rune_page: RunePage,
    pub item_sets: Vec<ItemSet>,
    pub skill_order: String,
    pub summoner_spells: SummonerSpells,
}

impl std::cmp::Eq for BuildData {}

impl std::cmp::PartialEq for BuildData {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position
    }
}

impl std::cmp::Ord for BuildData {
    fn cmp(&self, other: &Self) -> Ordering {
        FloatOrd(other.rune_page.win_rate).cmp(&FloatOrd(self.rune_page.win_rate))
    }
}

impl std::cmp::PartialOrd for BuildData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
