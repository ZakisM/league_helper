use regex::Regex;
use reqwest::Client;

use crate::endpoints::ugg::UggEndpoint;
use crate::models::ddragon_champions::Champion;
use crate::models::ddragon_runes_reforged::RunesData;
use crate::models::errors::LeagueHelperError;
use crate::models::ugg::build_data::BuildData;
use crate::models::ugg::position::Position;
use crate::models::ugg::ugg_role_data::UggRoleData;
use crate::Result;

const OVERVIEW_WORLD: &str = "12";
const OVERVIEW_PLAT_PLUS: &str = "10";

const UGGAPI_VERSION: &str = "1.1";
const UGGOVERVIEW_VERSION: &str = "1.4.0";

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

    pub async fn get_champion_data(
        &self,
        champion: &Champion,
        runes_data: &RunesData,
    ) -> Result<Vec<Result<BuildData>>> {
        let res = self
            .call_endpoint(&UggEndpoint::ChampionData(
                &self.patch_version,
                champion.key,
                UGGOVERVIEW_VERSION,
            ))
            .await?;

        let data = json::parse(&res)?;

        let data = &data[OVERVIEW_WORLD][OVERVIEW_PLAT_PLUS];

        let format_error = |champion_name: &str, position: &Position, e| -> LeagueHelperError {
            LeagueHelperError::new(format!("{} [{}] - {}", champion_name, position, e))
        };

        let build_data = data
            .entries()
            .map(|(k, v)| {
                let key_int = k.parse::<isize>()?;
                let position = Position::from(key_int);
                let role_data = UggRoleData(v);

                let rune_page = role_data
                    .get_rune_page(runes_data)
                    .map_err(|e| format_error(&champion.name, &position, e))?;
                let item_set = role_data
                    .get_item_set()
                    .map_err(|e| format_error(&champion.name, &position, e))?;
                let skill_order = role_data
                    .get_skill_order()
                    .map_err(|e| format_error(&champion.name, &position, e))?;
                let summoner_spells = role_data
                    .get_summoner_spells()
                    .map_err(|e| format_error(&champion.name, &position, e))?;

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
