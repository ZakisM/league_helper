use std::cmp::Ordering;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ChampionData {
    #[serde(rename = "type")]
    pub data_type: String,
    pub format: String,
    pub version: String,
    #[serde(deserialize_with = "champion_list_deserializer")]
    #[serde(rename = "data")]
    pub champion_list: Vec<Champion>,
}

fn champion_list_deserializer<'de, D>(deserializer: D) -> Result<Vec<Champion>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let data: HashMap<String, Champion> = HashMap::deserialize(deserializer)?;

    Ok(data.into_values().collect())
}

fn string_to_isize<'de, D>(deserializer: D) -> Result<isize, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(s.parse()
        .expect("Failed to deserialize champion key to isize."))
}

fn isize_to_string<S>(key: &isize, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::ser::Serializer,
{
    serializer.serialize_str(&key.to_string())
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Champion {
    pub version: String,
    pub id: String,
    #[serde(deserialize_with = "string_to_isize")]
    #[serde(serialize_with = "isize_to_string")]
    pub key: isize,
    pub name: String,
    pub title: String,
    pub blurb: String,
    pub info: Info,
    pub image: Image,
    pub tags: Vec<String>,
    pub partype: String,
    pub stats: Stats,
}

impl std::cmp::Eq for Champion {}

impl std::cmp::PartialEq for Champion {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl std::cmp::Ord for Champion {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key.cmp(&other.key)
    }
}

impl std::cmp::PartialOrd for Champion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Serialize, Deserialize, PartialOrd, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Info {
    pub attack: i64,
    pub defense: i64,
    pub magic: i64,
    pub difficulty: i64,
}

#[derive(Debug, Serialize, Deserialize, PartialOrd, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    pub full: String,
    pub sprite: String,
    pub group: String,
    pub x: i64,
    pub y: i64,
    pub w: i64,
    pub h: i64,
}

#[derive(Debug, Serialize, Deserialize, PartialOrd, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
    pub hp: f64,
    pub hpperlevel: i64,
    pub mp: f64,
    pub mpperlevel: f64,
    pub movespeed: i64,
    pub armor: f64,
    pub armorperlevel: f64,
    pub spellblock: f64,
    pub spellblockperlevel: f64,
    pub attackrange: i64,
    pub hpregen: f64,
    pub hpregenperlevel: f64,
    pub mpregen: f64,
    pub mpregenperlevel: f64,
    pub crit: i64,
    pub critperlevel: i64,
    pub attackdamage: f64,
    pub attackdamageperlevel: f64,
    pub attackspeedperlevel: f64,
    pub attackspeed: f64,
}
