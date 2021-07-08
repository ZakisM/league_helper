const UGG_URL: &str = "https://u.gg";

pub enum UggEndpoint<'a> {
    HomePage,
    BaseUrl(&'a str),
    ChampionData(&'a str, isize, &'a str),
}

impl UggEndpoint<'_> {
    pub fn url(&self) -> String {
        match self {
            UggEndpoint::HomePage => UGG_URL.to_owned(),
            UggEndpoint::BaseUrl(api_version) => {
                format!("https://stats2.u.gg/lol/{}", api_version)
            }
            UggEndpoint::ChampionData(patch_version, champion_key, overview_version) => {
                format!(
                    "/overview/{}/ranked_solo_5x5/{}/{}.json",
                    patch_version, champion_key, overview_version
                )
            }
        }
    }
}
