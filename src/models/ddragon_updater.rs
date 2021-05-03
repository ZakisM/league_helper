use reqwest::Client;
use serde::de::DeserializeOwned;

use crate::endpoints::ddragon::DDragonEndpoint;
use crate::models::ddragon_champions::ChampionData;
use crate::models::ddragon_runes_reforged::{RuneData, RunesData};
use crate::Result;

#[derive(Debug)]
pub struct DDragonUpdater {
    client: Client,
    pub version: String,
}

impl DDragonUpdater {
    pub async fn new() -> Result<Self> {
        let client = Client::new();

        let res = client
            .get(&DDragonEndpoint::Version.url())
            .send()
            .await?
            .text()
            .await?;

        let version = serde_json::from_str::<Vec<String>>(&res)?
            .get(0)
            .expect("Missing version data from DDragon.")
            .to_string();

        Ok(DDragonUpdater { client, version })
    }

    pub async fn download_latest_champions(&self) -> Result<ChampionData> {
        let mut data: ChampionData = self
            .call_endpoint(&DDragonEndpoint::ChampionData(&self.version))
            .await?;

        data.champion_list.sort_by(|a, b| a.id.cmp(&b.id));

        Ok(data)
    }

    pub async fn download_latest_runes(&self) -> Result<RunesData> {
        let data: Vec<RuneData> = self
            .call_endpoint(&DDragonEndpoint::RunesData(&self.version))
            .await?;

        let data = RunesData { runes_data: data };

        Ok(data)
    }

    async fn call_endpoint<T: DeserializeOwned>(
        &self,
        endpoint: &DDragonEndpoint<'_>,
    ) -> Result<T> {
        let res = self
            .client
            .get(&endpoint.url())
            .send()
            .await?
            .text()
            .await?;

        Ok(serde_json::from_str::<T>(&res)?)
    }
}
