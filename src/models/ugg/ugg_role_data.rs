use json::JsonValue;

use crate::models::ddragon_runes_reforged::RunesData;
use crate::models::errors::{ErrorExt, LeagueHelperError};
use crate::models::ugg::item_set::ItemSet;
use crate::models::ugg::rune_page::RunePage;
use crate::models::ugg::summoner_spells::SummonerSpells;
use crate::util::calc_win_rate;
use crate::Result;

#[derive(Debug)]
pub struct UggRoleData<'a>(pub &'a JsonValue);

impl<'a> UggRoleData<'a> {
    pub fn get_rune_page(&self, runes_data: &RunesData) -> Result<RunePage> {
        let data = self.0;

        let primary_tree = data[0][0][2]
            .as_isize()
            .context("Failed to read runes primary tree")?;

        let secondary_tree = data[0][0][3]
            .as_isize()
            .context("Failed to read runes secondary tree")?;

        let valid_primary_tree_runes = runes_data
            .runes_data
            .iter()
            .find(|r| r.id == primary_tree)
            .map(|r| &r.slots)
            .context("Primary tree rune key was not valid")?;

        let valid_secondary_tree_runes = runes_data
            .runes_data
            .iter()
            .find(|r| r.id == secondary_tree)
            .map(|r| &r.slots)
            .context("Secondary tree rune key was not valid")?;

        let mut runes = data[0][0][4]
            .members()
            .map(|v| v.as_isize().unwrap_or(0))
            .collect::<Vec<_>>();

        let mut i = 0;

        //Sort and validate the runes based off of DDragon Runes Reforged
        for slot in valid_primary_tree_runes {
            let current_index = runes
                .iter()
                .position(|rune| slot.runes.iter().map(|r| r.id).any(|r| &r == rune))
                .context("Ugg rune is not a valid league rune")?;

            if current_index != i {
                runes.swap(current_index, i);
            }

            i += 1;
        }

        for slot in valid_secondary_tree_runes.iter().skip(1) {
            let current_index = runes
                .iter()
                .position(|rune| slot.runes.iter().map(|r| r.id).any(|r| &r == rune));

            if let Some(current_index) = current_index {
                if current_index != i {
                    runes.swap(current_index, i);
                }

                i += 1;
            }
        }

        let mut stat_shards = data[0][8][2]
            .members()
            .map(|v| {
                v.as_str()
                    .and_then(|v| v.parse::<isize>().ok())
                    .unwrap_or(0)
            })
            .collect::<Vec<_>>();

        runes.append(&mut stat_shards);

        if runes.len() != 9 {
            return Err(LeagueHelperError::new(
                "Could not find a complete rune page",
            ));
        }

        // Rune page win rate
        let games_played = data[0][0][0]
            .as_isize()
            .context("Failed to read runes games played")?;

        let games_won = data[0][0][1]
            .as_isize()
            .context("Failed to read runes games won")?;

        let win_rate = calc_win_rate(games_won as f32, games_played as f32);

        Ok(RunePage {
            runes,
            primary_tree,
            secondary_tree,
            win_rate,
        })
    }

    pub fn get_item_set(&self) -> Result<Vec<ItemSet>> {
        let data = self.0;

        let mut sets = Vec::new();

        let mut add_with_win_rate = |name: &str, index: usize| -> Result<()> {
            let games_won = data[0][index][0]
                .as_f64()
                .context(format!("Failed to read item set: {} for games_won", name))?;

            let games_played = data[0][index][1].as_f64().context(format!(
                "Failed to read item set: {} for games_played",
                name
            ))?;

            sets.push(ItemSet {
                name: format!(
                    "{} - {:.2}% win rate",
                    name,
                    (games_played / games_won) * 100_f64
                ),
                items: data[0][index][2]
                    .members()
                    .map(|v| v.as_isize().unwrap_or(0))
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
                items: v.members().map(|v| v[0].as_isize().unwrap_or(0)).collect(),
            });
        });

        Ok(sets)
    }

    pub fn get_skill_order(&self) -> Result<String> {
        let data = self.0;

        let skill_order = data[0][4][3]
            .as_str()
            .context("Failed to read skill order")?
            .chars()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(">");

        Ok(skill_order)
    }

    pub fn get_summoner_spells(&self) -> Result<SummonerSpells> {
        let data = self.0;

        let first = data[0][1][2][0]
            .as_isize()
            .context("Failed to read first summoner spell")?;

        let second = data[0][1][2][1]
            .as_isize()
            .context("Failed to read second summoner spell")?;

        Ok(SummonerSpells { first, second })
    }
}
