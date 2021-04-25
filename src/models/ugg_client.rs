use json::JsonValue;
use regex::Regex;
use reqwest::Client;

use crate::endpoints::ugg::UggEndpoint;
use crate::models::errors::LeagueHelperError;
use crate::Result;

const OVERVIEW_WORLD: &str = "12";
const OVERVIEW_PLAT_PLUS: &str = "10";

const UGGAPI_VERSION: &str = "1.1";
const UGGOVERVIEW_VERSION: &str = "1.4.0";

#[derive(Debug, strum::Display, Eq, PartialEq)]
pub enum Position {
    Unknown = 0,
    Jungle = 1,
    Support = 2,
    Bottom = 3,
    Top = 4,
    Mid = 5,
}

impl From<usize> for Position {
    fn from(v: usize) -> Self {
        match v {
            x if x == Position::Jungle as usize => Position::Jungle,
            x if x == Position::Support as usize => Position::Support,
            x if x == Position::Bottom as usize => Position::Bottom,
            x if x == Position::Top as usize => Position::Top,
            x if x == Position::Mid as usize => Position::Mid,
            _ => Position::Unknown,
        }
    }
}

#[derive(Debug)]
pub struct RoleData<'a>(&'a JsonValue);

impl<'a> RoleData<'a> {
    pub fn get_rune_page(&self) -> Result<RunePage> {
        let data = self.0;

        let primary_tree = data[0][0][2]
            .as_usize()
            .ok_or_else(|| LeagueHelperError::new("Failed to read runes primary tree"))?;

        let secondary_tree = data[0][0][3]
            .as_usize()
            .ok_or_else(|| LeagueHelperError::new("Failed to read runes secondary tree"))?;

        let mut runes = data[0][0][4]
            .members()
            .map(|v| v.as_usize().unwrap_or(0))
            .collect::<Vec<_>>();

        let mut stat_shards = data[0][8][2]
            .members()
            .map(|v| {
                v.as_str()
                    .and_then(|v| v.parse::<usize>().ok())
                    .unwrap_or(0)
            })
            .collect::<Vec<_>>();

        runes.append(&mut stat_shards);

        Ok(RunePage {
            runes,
            primary_tree,
            secondary_tree,
        })
    }

    pub fn get_item_set(&self) -> Result<Vec<ItemSet>> {
        let data = self.0;

        let mut sets = Vec::new();

        let mut add_with_win_rate = |name: &str, index: usize| -> Result<()> {
            let games_won = data[0][index][0].as_f64().ok_or_else(|| {
                LeagueHelperError::new(format!("Failed to read item set: {} for games_won", name))
            })?;

            let games_played = data[0][index][1].as_f64().ok_or_else(|| {
                LeagueHelperError::new(format!(
                    "Failed to read item set: {} for games_played",
                    name
                ))
            })?;

            sets.push(ItemSet {
                name: format!(
                    "{} - {:.2}% win rate",
                    name,
                    (games_played / games_won) * 100_f64
                ),
                items: data[0][index][2]
                    .members()
                    .map(|v| v.as_usize().unwrap_or(0))
                    .collect(),
            });

            Ok(())
        };

        add_with_win_rate("Starting Build", 2)?;
        add_with_win_rate("Core Build", 3)?;

        let set_names = ["Fourth", "Fifth", "Sixth"];

        data[0][5].members().enumerate().for_each(|(i, v)| {
            sets.push(ItemSet {
                name: format!(
                    "{} Item Options (ordered by games played)",
                    set_names.get(i).unwrap_or(&"Unknown")
                ),
                items: v.members().map(|v| v[0].as_usize().unwrap_or(0)).collect(),
            });
        });

        Ok(sets)
    }

    pub fn get_skill_order(&self) -> Result<String> {
        let data = self.0;

        let skill_order = data[0][4][3]
            .as_str()
            .ok_or_else(|| LeagueHelperError::new("Failed to read skill order"))?
            .chars()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(">");

        Ok(skill_order)
    }

    pub fn get_summoner_spells(&self) -> Result<SummonerSpells> {
        let data = self.0;

        let first = data[0][1][2][0]
            .as_usize()
            .ok_or_else(|| LeagueHelperError::new("Failed to read first summoner spell"))?;

        let second = data[0][1][2][1]
            .as_usize()
            .ok_or_else(|| LeagueHelperError::new("Failed to read second summoner spell"))?;

        Ok(SummonerSpells { first, second })
    }
}

#[derive(Debug)]
pub struct SummonerSpells {
    first: usize,
    second: usize,
}

#[derive(Debug)]
pub struct ItemSet {
    pub name: String,
    pub items: Vec<usize>,
}

#[derive(Debug)]
pub struct RunePage {
    pub runes: Vec<usize>,
    pub primary_tree: usize,
    pub secondary_tree: usize,
}

#[derive(Debug)]
pub struct BuildData {
    pub position: Position,
    pub rune_page: RunePage,
    pub item_sets: Vec<ItemSet>,
    pub skill_order: String,
    pub summoner_spells: SummonerSpells,
}

#[derive(Debug)]
pub struct UggClient {
    pub client: Client,
    pub patch_version: String,
    pub base_url: String,
}

impl UggClient {
    pub async fn new() -> Result<Self> {
        let client = Client::new();

        let home_page = client
            .get(UggEndpoint::HomePage.url())
            .send()
            .await?
            .text()
            .await?;

        let script_re = Regex::new(r#"src="(.*?/main\..*?\.js)""#)?;

        let script_url = script_re
            .captures(&home_page)
            .and_then(|c| c.get(1))
            .ok_or_else(|| LeagueHelperError::new("Failed to find latest ugg script"))?
            .as_str();

        let version_re = Regex::new(r#"\[\{value:"(\d+_\d+)""#)?;

        let script_page = client.get(script_url).send().await?.text().await?;

        let patch_version = version_re
            .captures(&script_page)
            .and_then(|c| c.get(1))
            .ok_or_else(|| LeagueHelperError::new("Failed to read patch version"))?
            .as_str()
            .to_owned();

        let base_url = UggEndpoint::BaseUrl(UGGAPI_VERSION).url();

        Ok(UggClient {
            client,
            patch_version,
            base_url,
        })
    }

    pub async fn get_champion_data(&self, champion_id: u32) -> Result<Vec<Result<BuildData>>> {
        let res = self
            .call_endpoint(&UggEndpoint::ChampionData(
                &self.patch_version,
                champion_id,
                UGGOVERVIEW_VERSION,
            ))
            .await?;

        let data = json::parse(&res)?;

        let data = &data[OVERVIEW_WORLD][OVERVIEW_PLAT_PLUS];

        let build_data = data
            .entries()
            .map(|(k, v)| {
                let key_int = k.parse::<usize>()?;
                let position = Position::from(key_int);
                let role_data = RoleData(v);

                let rune_page = role_data.get_rune_page()?;
                let item_set = role_data.get_item_set()?;
                let skill_order = role_data.get_skill_order()?;
                let summoner_spells = role_data.get_summoner_spells()?;

                Ok(BuildData {
                    position,
                    rune_page,
                    item_sets: item_set,
                    skill_order,
                    summoner_spells,
                })
            })
            .collect::<Vec<_>>();

        if build_data.is_empty() {
            Err(LeagueHelperError::new("No build data found."))
        } else {
            Ok(build_data)
        }
    }

    async fn call_endpoint(&self, endpoint: &UggEndpoint<'_>) -> Result<String> {
        let res = self
            .client
            .get(format!("{}{}", self.base_url, &endpoint.url()))
            .send()
            .await?
            .text()
            .await?;

        Ok(res)
    }
}
