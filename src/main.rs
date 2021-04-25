use std::path::Path;

use futures::stream::{self, StreamExt};
use tokio::io::AsyncWriteExt;

use crate::models::ddragon_champions::Champion;
use crate::models::ddragon_updater::DDragonUpdater;
use crate::models::errors::LeagueHelperError;
use crate::models::league_item_set::LeagueItemSet;
use crate::models::ugg_client::{BuildData, UggClient};

mod endpoints;
mod models;

type Result<T> = std::result::Result<T, LeagueHelperError>;

#[cfg(windows)]
const DEFAULT_BUILD_DIR: &str = "C:\\Riot Games\\League of Legends\\Config\\Champions";

#[cfg(mac)]
const DEFAULT_BUILD_DIR: &str = "C:/Riot Games/League of Legends/Config/Champions";

#[tokio::main]
async fn main() -> Result<()> {
    let builds_path = Path::new(DEFAULT_BUILD_DIR);

    if !builds_path.exists() {
        return Err(LeagueHelperError::new("Builds path does not exist"));
    }

    let ddragon = DDragonUpdater::new().await?;
    let ugg_client = UggClient::new().await?;

    if !ddragon
        .version
        .replace(".", "_")
        .starts_with(&ugg_client.patch_version)
    {
        return Err(LeagueHelperError::new(
            "Ugg data is not up to date with latest patch version.",
        ));
    }

    let champion_data = ddragon.download_latest_champions().await?;

    let job = stream::iter(champion_data.champion_list)
        .map(|champion| async {
            let build_data = ugg_client.get_champion_data(champion.key).await;
            (champion, build_data, &ugg_client.patch_version)
        })
        .buffer_unordered(5);

    let task = job.for_each_concurrent(5, |(champion, build_data, patch_version)| async move {
        match build_data {
            Ok(build_data) => {
                for build in build_data {
                    match build {
                        Ok(build_data) => {
                            if let Err(e) =
                                save_build_data(&champion, build_data, builds_path, patch_version)
                                    .await
                            {
                                eprintln!("{}", e);
                            }
                        }
                        Err(e) => eprintln!("{}", e),
                    }
                }
            }
            Err(e) => eprintln!("{}", e),
        }
    });

    task.await;

    Ok(())
}

async fn save_build_data(
    champion: &Champion,
    mut build_data: BuildData,
    builds_path: &Path,
    patch_version: &str,
) -> Result<()> {
    if let Some(starting_build) = build_data.item_sets.get_mut(0) {
        starting_build
            .name
            .push_str(&format!(" [Skill Order: {}]", build_data.skill_order));
    }

    let league_item_set = LeagueItemSet::from_build_data(&mut build_data, &champion);

    let league_item_set_json = serde_json::to_vec_pretty(&league_item_set)?;

    let build_file_path = builds_path.join(&champion.id).join("Recommended");

    if !build_file_path.exists() {
        tokio::fs::create_dir_all(&build_file_path).await?;
    }

    let mut build_file = tokio::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(build_file_path.join(format!(
            "{}-{}-{}.json",
            &champion.id, &build_data.position, patch_version
        )))
        .await?;

    build_file.write_all(&league_item_set_json).await?;

    println!(
        "Downloaded build for: {} [{}].",
        champion.name, build_data.position
    );

    Ok(())
}
