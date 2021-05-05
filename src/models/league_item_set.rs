use serde::{Deserialize, Serialize};

use crate::models::ddragon_champions::Champion;
use crate::models::ugg::build_data::BuildData;
use crate::models::ugg::item_set::ItemSet;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeagueItemSet<'a> {
    pub title: String,
    #[serde(rename = "type")]
    pub type_field: &'a str,
    pub map: &'a str,
    pub mode: &'a str,
    pub priority: bool,
    pub sortrank: i64,
    pub blocks: Vec<Block<'a>>,
    pub champion_key: &'a str,
}

impl<'a> Default for LeagueItemSet<'a> {
    fn default() -> Self {
        Self {
            title: "".to_string(),
            type_field: "custom",
            map: "any",
            mode: "any",
            priority: false,
            sortrank: 9999999999,
            blocks: vec![],
            champion_key: "",
        }
    }
}

impl<'a> LeagueItemSet<'a> {
    pub fn from_build_data(build_data: &'a mut BuildData, champion: &'a Champion) -> Self {
        Self {
            title: format!("[LH] - {} {}", champion.name, build_data.position),
            blocks: build_data.item_sets.iter().map(Block::from).collect(),
            champion_key: &champion.id,
            ..Self::default()
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block<'a> {
    pub rec_math: bool,
    pub min_summoner_level: i64,
    pub max_summoner_level: i64,
    pub show_if_summoner_spell: &'a str,
    pub hide_if_summoner_spell: &'a str,
    #[serde(rename = "type")]
    pub type_field: &'a str,
    pub items: Vec<Item>,
}

impl<'a> Default for Block<'a> {
    fn default() -> Self {
        Self {
            rec_math: false,
            min_summoner_level: -1,
            max_summoner_level: -1,
            show_if_summoner_spell: "",
            hide_if_summoner_spell: "",
            type_field: "",
            items: vec![],
        }
    }
}

impl<'a> From<&'a ItemSet> for Block<'a> {
    fn from(item_set: &'a ItemSet) -> Self {
        Self {
            type_field: &item_set.name,
            items: item_set.items.iter().map(Item::from).collect(),
            ..Self::default()
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub id: String,
    pub count: i64,
}

impl From<&isize> for Item {
    fn from(item_id: &isize) -> Self {
        Self {
            id: item_id.to_string(),
            count: 1,
        }
    }
}
