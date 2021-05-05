use std::path::{Path, PathBuf};

use futures::stream::{self, StreamExt};
use lcu_driver::endpoints::perks::PerksPage;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;

use crate::models::ddragon_champions::Champion;
use crate::models::ddragon_updater::DDragonUpdater;
use crate::models::league_item_set::LeagueItemSet;
use crate::models::ugg::build_data::BuildData;
use crate::models::ugg::position::Position;
use crate::models::ugg::ugg_client::UggClient;
use crate::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct UggBuildData {
    pub patch_version: String,
    pub builds: Vec<(Champion, Vec<BuildData>)>,
}

impl UggBuildData {
    async fn get_ugg_build_data(
        ddragon: &DDragonUpdater,
        ugg_client: &UggClient,
    ) -> Result<UggBuildData> {
        let required_version = ddragon.version.replace(".", "_");

        if !required_version.starts_with(&ugg_client.patch_version) {
            println!(
                "Ugg data is not up to date with latest patch version. (Ugg: {}) (League: {})",
                ugg_client.patch_version, ddragon.version
            );
        }

        let champion_data = ddragon.download_latest_champions().await?;
        let runes_data = ddragon.download_latest_runes().await?;

        let mut builds = Vec::with_capacity(champion_data.champion_list.len());

        let mut download_job = stream::iter(champion_data.champion_list)
            .map(|champion| async {
                let build_data = ugg_client.get_champion_data(&champion, &runes_data).await;
                (champion, build_data)
            })
            .buffer_unordered(5);

        while let Some((champion, build_data)) = download_job.next().await {
            match build_data {
                Ok(build_data) => {
                    let mut curr_builds = Vec::with_capacity(5);

                    for build in build_data {
                        match build {
                            Ok(build_data) => {
                                curr_builds.push(build_data);
                            }
                            Err(e) => eprintln!("{}", e),
                        }
                    }

                    builds.push((champion, curr_builds));
                }
                Err(e) => eprintln!("{}", e),
            }
        }

        let ugg_build_data = UggBuildData {
            patch_version: ugg_client.patch_version.clone(),
            builds,
        };

        Ok(ugg_build_data)
    }

    fn json_file_path(patch_version: &str) -> PathBuf {
        Path::new(&format!("./ugg-builds-{}.json", patch_version)).to_path_buf()
    }

    pub async fn load(ddragon: &DDragonUpdater) -> Result<Self> {
        if let Ok(data) = tokio::fs::read_to_string(Self::json_file_path(&ddragon.version)).await {
            println!("Loading existing data...");

            let ugg_data = serde_json::from_str(&data)?;

            Ok(ugg_data)
        } else {
            println!("No existing data found...");

            let ugg_client = UggClient::new().await?;

            let data = Self::get_ugg_build_data(ddragon, &ugg_client).await?;

            data.save_to_json(&ddragon.version).await?;

            Ok(data)
        }
    }

    pub async fn save_to_json(&self, patch_version: &str) -> Result<()> {
        let data = serde_json::to_string(&self)?;

        tokio::fs::write(Self::json_file_path(patch_version), data).await?;

        Ok(())
    }

    pub async fn save_item_builds(&self, builds_path: &Path) -> Result<()> {
        for (champion, builds) in &self.builds {
            for build_data in builds {
                // Clone as don't want to modify the original data
                let mut build_data = build_data.clone();

                if let Some(starting_build) = build_data.item_sets.get_mut(0) {
                    starting_build
                        .name
                        .push_str(&format!(" [Skill Order: {}]", build_data.skill_order));
                }

                let build_file_path = builds_path.join(&champion.id).join("Recommended");

                if !build_file_path.exists() {
                    tokio::fs::create_dir_all(&build_file_path).await?;
                }

                let league_item_set = LeagueItemSet::from_build_data(&mut build_data, &champion);

                let league_item_set_json = serde_json::to_vec_pretty(&league_item_set)?;

                let mut build_file = tokio::fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(build_file_path.join(format!(
                        "{}-{}-{}.json",
                        &champion.id, &build_data.position, self.patch_version
                    )))
                    .await?;

                build_file.write_all(&league_item_set_json).await?;

                println!(
                    "Saved build for: {} {}.",
                    champion.name, build_data.position
                );
            }
        }

        Ok(())
    }

    pub fn get_perks_page(
        &self,
        champion_key: isize,
        position: Option<Position>,
    ) -> Option<PerksPage> {
        //TODO: Default position based off games played/same as default used on U.gg
        //TODO: Auto import summoner spells
        //TODO: Ryze doesn't import runes? - Because a page already existed for him, delete all [LH] Pages on startup
        let position = position.unwrap_or(Position::Mid);

        self.builds
            .iter()
            .find(|(champion, _)| champion.key == champion_key)
            .and_then(|(champion, build_data)| {
                build_data
                    .iter()
                    .find(|b| b.position == position)
                    .map(|b| PerksPage {
                        name: format!("[LH] {} {}", champion.name, position),
                        primary_style_id: b.rune_page.primary_tree,
                        selected_perk_ids: b.rune_page.runes.clone(),
                        sub_style_id: b.rune_page.secondary_tree,
                        ..PerksPage::default()
                    })
            })
    }
}
