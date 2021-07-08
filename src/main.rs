use std::str::FromStr;
use std::time::Duration;

use lcu_driver::endpoints::gameflow::{GameFlowPhase, GameFlowSession};
use lcu_driver::endpoints::summoner::Summoner;
use lcu_driver::{Initialized, LcuDriver};

use crate::models::ddragon_updater::DDragonUpdater;
use crate::models::errors::{ErrorExt, LeagueHelperError};
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

    // lcu_driver
    //     .connect_websocket()
    //     .await
    //     .expect("Failed to connect WS");

    let builds_path = lcu_driver.league_install_dir().await.join("Config");

    if !builds_path.exists() {
        return Err(LeagueHelperError::new("Builds path does not exist"));
    }

    let builds_path = builds_path.join("Champions");

    ugg_build_data
        .delete_old_item_builds(&builds_path, None)
        .await?;
    ugg_build_data.save_item_builds(&builds_path).await?;

    let my_summoner = lcu_driver.get_current_summoner().await?;

    let mut previous_champion_id = -1;

    loop {
        match lcu_driver.get_gameflow_session().await {
            Ok(game_flow_session) => match &game_flow_session.phase {
                GameFlowPhase::Lobby => {
                    println!("In lobby...");
                }
                GameFlowPhase::InProgress => {
                    println!("Waiting for game to end...");
                    tokio::time::sleep(Duration::from_secs(60)).await;
                }
                GameFlowPhase::ChampSelect => {
                    if let Err(e) = load_champion_runes_and_summoners(
                        &lcu_driver,
                        &game_flow_session,
                        &ugg_build_data,
                        &my_summoner,
                        &mut previous_champion_id,
                    )
                    .await
                    {
                        eprintln!("{}", e);
                    }
                }
                _ => (),
            },
            Err(e) => {
                eprintln!("{}", e);
                previous_champion_id = -1;
            }
        }

        tokio::time::sleep(Duration::from_millis(2500)).await;
    }
}

async fn load_champion_runes_and_summoners(
    lcu_driver: &LcuDriver<Initialized>,
    game_flow_session: &GameFlowSession,
    ugg_build_data: &UggBuildData,
    my_summoner: &Summoner,
    previous_champion_id: &mut isize,
) -> Result<()> {
    let champ_select_session = lcu_driver.get_champ_select_session().await?;

    let my_player_selection = champ_select_session
        .my_team
        .iter()
        .find(|p| p.summoner_id == my_summoner.summoner_id)
        .context("Couldn't find current player selection")?;

    /* Must pick a champion first and
    don't set the same page twice */
    if my_player_selection.champion_id == 0
        || my_player_selection.champion_id == *previous_champion_id
    {
        return Ok(());
    }

    println!("Loading runes");

    let position = Position::from_str(&my_player_selection.assigned_position).ok();

    let new_runes_page = ugg_build_data
        .get_perks_page(my_player_selection.champion_id, &position)
        .context("Couldn't find a rune page for this champion")?;

    let mut new_summoner_spells = ugg_build_data
        .get_summoner_spells(my_player_selection.champion_id, &position)
        .context("Couldn't find summoner spells for this champion")?
        .to_owned();

    let game_mode = &game_flow_session.map.game_mode;

    if let Some(disallowed_spells) = game_mode.disallowed_summoner_spells() {
        if disallowed_spells.is_empty() {
            new_summoner_spells.first = my_player_selection.spell1_id;
            new_summoner_spells.second = my_player_selection.spell2_id;
        } else {
            for spell in disallowed_spells {
                if new_summoner_spells.first == spell {
                    new_summoner_spells.first = my_player_selection.spell1_id;
                } else if new_summoner_spells.second == spell {
                    new_summoner_spells.second = my_player_selection.spell2_id;
                }
            }
        }
    }

    let new_summoner_spells = new_summoner_spells.to_my_selection(
        my_player_selection.selected_skin_id,
        my_player_selection.ward_skin_id,
    );

    let curr_runes_pages = lcu_driver
        .get_perks_pages()
        .await?
        .pages
        .into_iter()
        .filter(|p| p.is_deletable)
        .collect::<Vec<_>>();

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

        //Delete delete first page
        let page_to_delete = curr_runes_pages
            .first()
            .context("Couldn't find first rune page to delete")?;

        lcu_driver.delete_perks_page(page_to_delete.id).await?;
    }

    lcu_driver.set_perks_page(&new_runes_page).await?;
    lcu_driver
        .set_session_my_selection(&new_summoner_spells)
        .await?;

    *previous_champion_id = my_player_selection.champion_id;

    Ok(())
}
