use std::{path::PathBuf, sync::Arc};

use itertools::Itertools;
use rspack_error::{Result, ToStringResultToRspackResultExt};
use rspack_fs::IntermediateFileSystem;
use rspack_paths::Utf8PathBuf;
use rspack_storage::{SeedStage, StagedUnit};
use tokio::task::spawn_blocking;

use super::{
  storage::{CacheDirectory, create_storage},
  validation::CacheValidation,
};
use crate::{CompilationLogger, Logger, PersistentCacheOptions, StorageOptions};

/// Seeds an absent compiler unit from a sibling cache before the first load.
#[derive(Debug)]
pub struct CacheSeed {
  location: Utf8PathBuf,
  unit: CacheDirectory,
  donors: Vec<PathBuf>,
  fs: Arc<dyn IntermediateFileSystem>,
}

impl CacheSeed {
  pub fn new(
    option: &PersistentCacheOptions,
    unit: &CacheDirectory,
    fs: Arc<dyn IntermediateFileSystem>,
  ) -> Option<Self> {
    if option.readonly || option.seed_donors.is_empty() {
      return None;
    }
    let StorageOptions::FileSystem { location, .. } = &option.storage;
    Some(Self {
      location: location.clone(),
      unit: unit.clone(),
      donors: option
        .seed_donors
        .iter()
        .map(|donor| donor.as_std_path().to_path_buf())
        .collect(),
      fs,
    })
  }

  pub async fn run(self, validation: &CacheValidation, logger: &CompilationLogger) {
    match self.seed(validation, logger).await {
      Ok(Some(outcome)) => logger.info(outcome),
      Ok(None) => {}
      Err(error) => logger.info(format!("portable cache not seeded: {error}")),
    }
  }

  async fn seed(
    self,
    validation: &CacheValidation,
    logger: &CompilationLogger,
  ) -> Result<Option<String>> {
    let Self {
      location,
      unit,
      donors,
      fs,
    } = self;
    logger.debug(format!(
      "portable cache seed candidates: {}",
      donors.iter().map(|donor| donor.display()).join(", ")
    ));
    let ranked = {
      let location = location.clone();
      let unit = unit.clone();
      spawn_blocking(move || {
        rspack_storage::sweep_seed_staging(location.as_std_path());
        (!location.join(unit.as_str()).exists())
          .then(|| rspack_storage::rank_seed_donors(donors, &unit))
      })
      .await
      .to_rspack_result()?
    };
    let Some(ranked) = ranked else {
      logger.debug("portable cache seed skipped: compiler unit already exists");
      return Ok(None);
    };
    let mut skipped = Vec::new();
    for donor in ranked {
      logger.debug(format!("portable cache seed trying {}", donor.display()));
      let stage = {
        let location = location.clone();
        let donor = donor.clone();
        let unit = unit.clone();
        spawn_blocking(move || rspack_storage::stage_seed(location.as_std_path(), &donor, &unit))
          .await
          .to_rspack_result()?
      };
      let staged = match stage {
        Ok(SeedStage::Staged(staged)) => staged,
        Ok(SeedStage::Skipped(reason)) => {
          skipped.push(format!("{} ({reason})", donor.display()));
          continue;
        }
        Err(error) => {
          skipped.push(format!("{} ({error})", donor.display()));
          continue;
        }
      };
      if !matches_version(&staged, &unit, &fs, validation).await {
        skipped.push(format!(
          "{} (cache version does not match)",
          donor.display()
        ));
        spawn_blocking(move || drop(staged))
          .await
          .to_rspack_result()?;
        continue;
      }
      logger.debug(format!("portable cache seed donor: {}", donor.display()));
      let seeded = spawn_blocking(move || staged.commit())
        .await
        .to_rspack_result()??;
      return Ok(Some(if seeded {
        format!("portable cache seeded from {}", donor.display())
      } else {
        "portable cache not seeded: another compiler seeded this unit first".to_string()
      }));
    }
    Ok(Some(if skipped.is_empty() {
      "portable cache not seeded: no candidate contains this compiler's cache".to_string()
    } else {
      format!("portable cache not seeded: {}", skipped.join("; "))
    }))
  }
}

async fn matches_version(
  staged: &StagedUnit,
  unit: &CacheDirectory,
  fs: &Arc<dyn IntermediateFileSystem>,
  validation: &CacheValidation,
) -> bool {
  let Ok(directory) = Utf8PathBuf::from_path_buf(staged.directory().to_path_buf()) else {
    return false;
  };
  let storage = create_storage(
    StorageOptions::FileSystem {
      directory: directory.clone(),
      location: directory,
    },
    unit.clone(),
    0,
    fs.clone(),
  );
  validation.matches_version(&*storage).await
}
