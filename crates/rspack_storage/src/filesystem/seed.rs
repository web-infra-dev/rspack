use std::{
  collections::BTreeMap,
  fs,
  io::ErrorKind,
  path::{Path, PathBuf},
  time::{Duration, SystemTime},
};

use rspack_error::Result;
use tempfile::TempDir;

use super::{CacheDirectory, db::StateLock};

const STAGING_MARKER: &str = ".seeding-";
const STAGING_MAX_AGE: Duration = Duration::from_secs(24 * 60 * 60);
const ATTEMPTS: usize = 3;

fn is_transient(name: &std::ffi::OsStr) -> bool {
  name == ".temp" || name == "_stale"
}

/// Removes staging directories whose seeding process has exited. When the process
/// table cannot be queried, removes only directories older than a day.
pub fn sweep_seed_staging(location: &Path) {
  let Ok(entries) = fs::read_dir(location) else {
    return;
  };
  for entry in entries.flatten() {
    let name = entry.file_name();
    let Some((unit, owner)) = name
      .to_str()
      .and_then(|name| name.split_once(STAGING_MARKER))
    else {
      continue;
    };
    let Some(pid) = owner
      .split_once('-')
      .and_then(|(pid, _)| pid.parse::<u32>().ok())
    else {
      continue;
    };
    if !CacheDirectory::is_valid(unit) || pid == std::process::id() {
      continue;
    }
    let abandoned = StateLock::process_liveness(pid).map_or_else(
      || {
        entry
          .metadata()
          .and_then(|metadata| metadata.modified())
          .ok()
          .and_then(|modified| SystemTime::now().duration_since(modified).ok())
          .is_some_and(|age| age > STAGING_MAX_AGE)
      },
      |running| !running,
    );
    if abandoned {
      let _ = fs::remove_dir_all(entry.path());
    }
  }
}

fn newest_scope_meta(unit: &Path) -> Option<SystemTime> {
  fs::read_dir(unit)
    .ok()?
    .flatten()
    .filter(|entry| !is_transient(&entry.file_name()))
    .filter_map(|entry| {
      fs::metadata(entry.path().join("_meta"))
        .ok()?
        .modified()
        .ok()
    })
    .max()
}

/// Keeps donors that contain the compiler unit, most recently written first.
pub fn rank_seed_donors(donors: Vec<PathBuf>, unit: &CacheDirectory) -> Vec<PathBuf> {
  let mut ranked: Vec<_> = donors
    .into_iter()
    .filter_map(|donor| Some((newest_scope_meta(&donor.join(unit.as_str()))?, donor)))
    .collect();
  ranked.sort_by_key(|(modified, _)| std::cmp::Reverse(*modified));
  ranked.into_iter().map(|(_, donor)| donor).collect()
}

fn generations(directory: &Path, stamps: &mut BTreeMap<PathBuf, Vec<u8>>) -> Result<()> {
  for entry in fs::read_dir(directory)? {
    let entry = entry?;
    if is_transient(&entry.file_name()) {
      continue;
    }
    let kind = entry.file_type()?;
    if kind.is_symlink() {
      return Err(rspack_error::error!("cache donor contains a symbolic link"));
    }
    if kind.is_dir() {
      generations(&entry.path(), stamps)?;
    } else if entry.file_name() == "_meta" {
      stamps.insert(entry.path(), fs::read(entry.path())?);
    }
  }
  Ok(())
}

fn copy_directory(source: &Path, target: &Path) -> Result<()> {
  fs::create_dir(target)?;
  for entry in fs::read_dir(source)? {
    let entry = entry?;
    if is_transient(&entry.file_name()) {
      continue;
    }
    let kind = entry.file_type()?;
    let destination = target.join(entry.file_name());
    if kind.is_dir() {
      copy_directory(&entry.path(), &destination)?;
    } else if kind.is_file() {
      fs::copy(entry.path(), destination)?;
    } else {
      return Err(rspack_error::error!(
        "cache donor contains a non-regular file"
      ));
    }
  }
  Ok(())
}

fn pending_transaction(unit: &Path) -> Result<Option<&'static str>> {
  let temp = unit.join(".temp");
  if temp.join("commit.lock").try_exists()? {
    return Ok(Some("donor has an unfinished commit"));
  }
  match fs::read(temp.join("state.lock")) {
    Ok(bytes) => Ok(
      StateLock::decode(&bytes)
        .is_none_or(|lock| lock.liveness() != Some(false))
        .then_some("donor has a running transaction"),
    ),
    Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
    Err(error) => Err(error.into()),
  }
}

#[derive(Debug)]
pub enum SeedStage {
  Staged(StagedUnit),
  Skipped(&'static str),
}

/// A consistent copy of a donor unit. Dropping it removes the copy.
#[derive(Debug)]
pub struct StagedUnit {
  staging: TempDir,
  unit: CacheDirectory,
  destination: PathBuf,
}

impl StagedUnit {
  /// Storage directory whose compiler unit is the staged copy.
  pub fn directory(&self) -> &Path {
    self.staging.path()
  }

  /// Moves the staged unit into place. Returns `false` when another compiler seeded it first.
  pub fn commit(self) -> Result<bool> {
    match fs::rename(
      self.staging.path().join(self.unit.as_str()),
      &self.destination,
    ) {
      Ok(()) => Ok(true),
      Err(error)
        if matches!(
          error.kind(),
          ErrorKind::AlreadyExists | ErrorKind::DirectoryNotEmpty
        ) || self.destination.is_dir() =>
      {
        Ok(false)
      }
      Err(error) => Err(error.into()),
    }
  }
}

/// Copies an absent compiler unit from a donor without opening the donor as storage.
pub fn stage_seed(location: &Path, donor: &Path, unit: &CacheDirectory) -> Result<SeedStage> {
  let destination = location.join(unit.as_str());
  if destination.try_exists()? {
    return Ok(SeedStage::Skipped("compiler unit already exists"));
  }
  let source = donor.join(unit.as_str());
  fs::create_dir_all(location)?;
  for _ in 0..ATTEMPTS {
    if let Some(reason) = pending_transaction(&source)? {
      return Ok(SeedStage::Skipped(reason));
    }
    let mut before = BTreeMap::new();
    generations(&source, &mut before)?;
    let staging = tempfile::Builder::new()
      .prefix(&format!("{unit}{STAGING_MARKER}{}-", std::process::id()))
      .tempdir_in(location)?;
    copy_directory(&source, &staging.path().join(unit.as_str()))?;
    let mut after = BTreeMap::new();
    generations(&source, &mut after)?;
    if let Some(reason) = pending_transaction(&source)? {
      return Ok(SeedStage::Skipped(reason));
    }
    if before == after {
      return Ok(SeedStage::Staged(StagedUnit {
        staging,
        unit: unit.clone(),
        destination,
      }));
    }
  }
  Ok(SeedStage::Skipped(
    "donor changed during every copy attempt",
  ))
}
