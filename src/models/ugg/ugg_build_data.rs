use std::fmt::Write;
use std::fs;
use std::io::{Read, Write as StdIOWrite};
use std::path::{Path, PathBuf};

use futures::stream::{self, StreamExt};
use lcu_driver::endpoints::perks::PerksPage;
use serde::{Deserialize, Serialize};

use crate::models::ddragon_champions::Champion;
use crate::models::ddragon_updater::DDragonUpdater;
use crate::models::errors::LeagueHelperError;
use crate::models::league_item_set::LeagueItemSet;
use crate::models::ugg::build_data::BuildData;
use crate::models::ugg::position::Position;
use crate::models::ugg::summoner_spells::SummonerSpells;
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
        let required_version = ddragon.version.replace('.', "_");

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

                    curr_builds.sort();
                    builds.push((champion, curr_builds));
                }
                Err(e) => eprintln!(
                    "Failed to download build data for {} due to: {}",
                    champion.name, e
                ),
            }
        }

        builds.sort();

        let ugg_build_data = UggBuildData {
            patch_version: ugg_client.patch_version.clone(),
            builds,
        };

        Ok(ugg_build_data)
    }

    fn json_file_path(patch_version: &str) -> PathBuf {
        Path::new(&format!("./ugg-builds-{}.json.sz", patch_version)).to_path_buf()
    }

    pub async fn load(ddragon: &DDragonUpdater) -> Result<Self> {
        if let Ok(compressed_data) = tokio::fs::read(Self::json_file_path(&ddragon.version)).await {
            let mut data = String::with_capacity(compressed_data.len());

            let mut reader = snap::read::FrameDecoder::new(&*compressed_data);

            reader.read_to_string(&mut data)?;

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

        let mut compressed_writer = snap::write::FrameEncoder::new(Vec::with_capacity(data.len()));

        compressed_writer.write_all(data.as_bytes())?;

        let compressed_data = compressed_writer
            .into_inner()
            .map_err(|e| LeagueHelperError::Other(e.to_string()))?;

        tokio::fs::write(Self::json_file_path(patch_version), compressed_data).await?;

        Ok(())
    }

    pub fn delete_old_item_builds(&self, builds_path: &Path) -> Result<()> {
        for entry in fs::read_dir(&builds_path)? {
            let entry = entry?;

            if entry.file_type()?.is_dir() {
                self.delete_old_item_builds(&entry.path())?;
            } else if let Some(true) = entry
                .file_name()
                .to_str()
                .map(|f| f.starts_with("LH_") && f.ends_with(".json"))
            {
                let path = entry.path();

                println!("Deleting: {}", path.display());

                fs::remove_file(path)?;
            }
        }

        Ok(())
    }

    pub fn save_item_builds(&self, builds_path: &Path) -> Result<()> {
        for (champion, builds) in &self.builds {
            for build_data in builds {
                // Clone as don't want to modify the original data
                let mut build_data = build_data.clone();

                if let Some(starting_build) = build_data.item_sets.get_mut(0) {
                    write!(
                        starting_build.name,
                        " [Skill Order: {}]",
                        build_data.skill_order
                    )?;
                }

                let build_file_path = builds_path.join(&champion.id).join("Recommended");

                if !build_file_path.exists() {
                    fs::create_dir_all(&build_file_path)?;
                }

                let league_item_set = LeagueItemSet::from_build_data(&mut build_data, champion);

                let league_item_set_json = serde_json::to_vec_pretty(&league_item_set)?;

                let mut build_file =
                    fs::OpenOptions::new()
                        .write(true)
                        .create(true)
                        .open(build_file_path.join(format!(
                            "LH_{}-{}-{}.json",
                            &champion.id, &build_data.position, self.patch_version
                        )))?;

                build_file.write_all(&league_item_set_json)?;

                println!(
                    "Saved build for: {} {}.",
                    champion.name, build_data.position
                );
            }
        }

        Ok(())
    }

    pub fn get_perks_page(&self, champion_key: isize, position: &Position) -> Option<PerksPage> {
        self.builds
            .iter()
            .find(|(champion, _)| champion.key == champion_key)
            .and_then(|(champion, build_data)| {
                let build_data = build_data.iter().find(|b| b.position == *position);

                build_data.map(|b| PerksPage {
                    name: format!("[LH] {} {}", champion.name, b.position),
                    primary_style_id: b.rune_page.primary_tree,
                    selected_perk_ids: b.rune_page.runes.clone(),
                    sub_style_id: b.rune_page.secondary_tree,
                    ..PerksPage::default()
                })
            })
    }

    pub fn get_summoner_spells(
        &self,
        champion_key: isize,
        position: &Position,
    ) -> Option<&SummonerSpells> {
        self.builds
            .iter()
            .find(|(champion, _)| champion.key == champion_key)
            .and_then(|(_, build_data)| {
                let build_data = build_data.iter().find(|b| b.position == *position);

                build_data.map(|b| &b.summoner_spells)
            })
    }
}
