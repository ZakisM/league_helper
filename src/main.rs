use std::str::FromStr;
use std::time::Duration;

use lcu_driver::LcuDriver;

use crate::models::ddragon_updater::DDragonUpdater;
use crate::models::errors::LeagueHelperError;
use crate::models::ugg::position::Position;
use crate::models::ugg::ugg_build_data::UggBuildData;

mod endpoints;
mod models;

type Result<T> = std::result::Result<T, LeagueHelperError>;

#[tokio::main]
async fn main() -> Result<()> {
    let ddragon = DDragonUpdater::new().await?;

    let ugg_build_data = UggBuildData::load(&ddragon).await?;

    let lcu_driver = LcuDriver::connect_wait().await;

    let builds_path = lcu_driver
        .league_install_dir()
        .await
        .join("Config")
        .join("Champions");

    if !builds_path.exists() {
        return Err(LeagueHelperError::new("Builds path does not exist"));
    }

    ugg_build_data.save_item_builds(&builds_path).await?;

    let my_summoner = lcu_driver.get_current_summoner().await?;

    loop {
        match lcu_driver.get_champ_select_session().await {
            Ok(champ_select_session) => {
                let my_player_selection = champ_select_session
                    .my_team
                    .iter()
                    .find(|p| p.summoner_id == my_summoner.summoner_id);

                if let Some(my_player_selection) = my_player_selection {
                    let position = Position::from_str(&my_player_selection.assigned_position).ok();
                    let perks_page =
                        ugg_build_data.get_perks_page(my_player_selection.champion_id, position);

                    if let Some(perks_page) = perks_page {
                        if let Err(e) = lcu_driver.set_perks_page(&perks_page).await {
                            eprintln!("{}", e);
                        }
                    }
                }
            }
            Err(e) => eprintln!("{}", e),
        }

        tokio::time::sleep(Duration::from_millis(2500)).await;
    }
}
