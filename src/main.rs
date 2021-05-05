use std::str::FromStr;
use std::time::Duration;

use lcu_driver::endpoints::summoner::Summoner;
use lcu_driver::{Initialized, LcuDriver};

use crate::models::ddragon_updater::DDragonUpdater;
use crate::models::errors::LeagueHelperError;
use crate::models::ugg::position::Position;
use crate::models::ugg::ugg_build_data::UggBuildData;

mod endpoints;
mod models;
mod util;

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

    let mut previous_champion_id = -1;

    loop {
        match load_champion_runes_and_summoners(
            &lcu_driver,
            &ugg_build_data,
            &my_summoner,
            &mut previous_champion_id,
        )
        .await
        {
            Ok(_) => println!("Loaded champion runes"),
            Err(e) => eprintln!("{}", e),
        }

        tokio::time::sleep(Duration::from_millis(2500)).await;
    }
}

async fn load_champion_runes_and_summoners(
    lcu_driver: &LcuDriver<Initialized>,
    ugg_build_data: &UggBuildData,
    my_summoner: &Summoner,
    previous_champion_id: &mut isize,
) -> Result<()> {
    let champ_select_session = lcu_driver.get_champ_select_session().await?;

    let my_player_selection = champ_select_session
        .my_team
        .iter()
        .find(|p| p.summoner_id == my_summoner.summoner_id)
        .ok_or_else(|| LeagueHelperError::new("Couldn't find current player selection"))?;

    /* Must pick a champion first and
    don't set the same page twice */
    if my_player_selection.champion_id == 0
        || my_player_selection.champion_id == *previous_champion_id
    {
        return Ok(());
    }

    let position = Position::from_str(&my_player_selection.assigned_position).ok();

    let new_runes_page = ugg_build_data
        .get_perks_page(my_player_selection.champion_id, &position)
        .ok_or_else(|| LeagueHelperError::new("Couldn't find a rune page for this champion"))?;

    let new_summoner_spells = ugg_build_data
        .get_summoner_spells(my_player_selection.champion_id, &position)
        .ok_or_else(|| LeagueHelperError::new("Couldn't find summoner spells for this champion"))?;

    let curr_runes_pages = lcu_driver.get_perks_pages().await?.pages;

    //Delete any [LH] pages set previously
    let pages_to_delete = curr_runes_pages
        .iter()
        .filter(|p| p.name.starts_with("[LH]") && p.is_deletable);

    for page in pages_to_delete {
        lcu_driver.delete_perks_page(page.id).await?;
    }

    let perks_inventory = lcu_driver.get_perks_inventory().await?;

    // Delete page if space is required
    if curr_runes_pages.len() as isize == perks_inventory.owned_page_count {
        println!("Deleting rune page so we can create another one");

        //Delete previous page we created or else delete first page
        let page_to_delete =
            curr_runes_pages
                .iter()
                .find(|p| p.name.starts_with("[LH]"))
                .unwrap_or(curr_runes_pages.first().ok_or_else(|| {
                    LeagueHelperError::new("Couldn't find a rune page to delete")
                })?);

        lcu_driver.delete_perks_page(page_to_delete.id).await?;
    }

    lcu_driver.set_perks_page(&new_runes_page).await?;
    lcu_driver
        .set_session_my_selection(&new_summoner_spells.into())
        .await?;

    *previous_champion_id = my_player_selection.champion_id;

    Ok(())
}
