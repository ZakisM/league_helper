use app_error::{bail, AppError, AppErrorExt, Result};
use regex::Regex;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, USER_AGENT};
use reqwest::Client;

use crate::endpoints::ugg::UggEndpoint;
use crate::models::ddragon_champions::Champion;
use crate::models::ddragon_runes_reforged::RunesData;
use crate::models::ugg::build_data::BuildData;
use crate::models::ugg::position::Position;
use crate::models::ugg::ugg_role_data::UggRoleData;

const OVERVIEW_WORLD: &str = "12";
const OVERVIEW_PLAT_PLUS: &str = "10";

const UGGAPI_VERSION: &str = "1.5";
const UGGOVERVIEW_VERSION: &str = "1.5.0";

#[derive(Debug)]
pub struct UggClient {
    pub client: Client,
    pub patch_version: String,
    pub base_url: String,
}

impl UggClient {
    pub async fn new() -> Result<Self> {
        let headers = HeaderMap::from_iter([
            (USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")),
            (ACCEPT, HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8")),
            (ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.5")),
        ]);

        let client = Client::builder().default_headers(headers).build()?;

        let home_page = client
            .get(UggEndpoint::HomePage.url())
            .send()
            .await?
            .text()
            .await?;

        let version_re = Regex::new(r#"window\.__SSR_DATA__ = (\{.*})"#)?;

        let data_captures = version_re
            .captures(&home_page)
            .and_then(|c| c.get(1))
            .map(|c| c.as_str())
            .context("Failed to find captures in version.json")?;

        let data_json: serde_json::Value = serde_json::from_str(data_captures)?;

        let mut first_patch_version = data_json
            .get("https://static.bigbrain.gg/assets/lol/riot_patch_update/prod/versions.json")
            .and_then(|d| d.get("data"))
            .and_then(|d| d.as_array()?.first()?.as_str())
            .map(|d| d.split_terminator('.'))
            .context("Failed to get patch version data")?;

        let major_version = first_patch_version
            .next()
            .context("Failed to get patch major version")?;
        let minor_version = first_patch_version
            .next()
            .context("Failed to get patch minor version")?;

        let patch_version = format!("{}_{}", major_version, minor_version);

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

        let format_error = |champion_name: &str, position: &Position, e| -> AppError {
            AppError::new(format!("{} {} - {}", champion_name, position, e))
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
            bail!("No build data found.");
        } else {
            Ok(build_data)
        }
    }

    async fn call_endpoint(&self, endpoint: &UggEndpoint<'_>) -> Result<String> {
        let url = format!("{}{}", self.base_url, &endpoint.url());

        println!("Making GET request to endpoint: {}", url);

        let res = self.client.get(url).send().await?.text().await?;

        Ok(res)
    }
}
