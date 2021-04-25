const DDRAGON_URL: &str = "https://ddragon.leagueoflegends.com";

#[allow(unused)]
pub enum DDragonEndpoint<'a> {
    Version,
    ChampionData(&'a str),
}

impl DDragonEndpoint<'_> {
    pub fn url(&self) -> String {
        match self {
            DDragonEndpoint::Version => format!("{}/api/versions.json", DDRAGON_URL),
            DDragonEndpoint::ChampionData(version) => {
                format!("{}/cdn/{}/data/en_US/champion.json", DDRAGON_URL, version)
            }
        }
    }
}
