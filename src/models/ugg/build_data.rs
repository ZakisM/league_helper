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
