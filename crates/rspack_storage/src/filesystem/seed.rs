use std::{
  collections::BTreeMap,
  fs,
  io::ErrorKind,
  path::{Path, PathBuf},
};

use rspack_error::Result;

use super::CacheDirectory;

fn generations(directory: &Path, stamps: &mut BTreeMap<PathBuf, Vec<u8>>) -> Result<()> {
  for entry in fs::read_dir(directory)? {
    let entry = entry?;
    if entry.file_name() == ".temp" || entry.file_name() == "_stale" {
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
    if entry.file_name() == ".temp" || entry.file_name() == "_stale" {
      continue;
    }
    let kind = entry.file_type()?;
    let destination = target.join(entry.file_name());
    if kind.is_dir() {
      copy_directory(&entry.path(), &destination)?;
    } else if kind.is_file() {
      reflink_copy::reflink_or_copy(entry.path(), destination)?;
    } else {
      return Err(rspack_error::error!(
        "cache donor contains a non-regular file"
      ));
    }
  }
  Ok(())
}

#[cfg(target_os = "macos")]
fn remove_transient(directory: &Path) -> Result<()> {
  for entry in fs::read_dir(directory)? {
    let entry = entry?;
    if entry.file_name() == ".temp" || entry.file_name() == "_stale" {
      if entry.file_type()?.is_dir() {
        fs::remove_dir_all(entry.path())?;
      } else {
        fs::remove_file(entry.path())?;
      }
    } else if entry.file_type()?.is_dir() {
      remove_transient(&entry.path())?;
    }
  }
  Ok(())
}

fn clone_directory(source: &Path, target: &Path) -> Result<&'static str> {
  #[cfg(target_os = "macos")]
  if reflink_copy::reflink(source, target).is_ok() {
    remove_transient(target)?;
    return Ok("directory reflink");
  }
  copy_directory(source, target)?;
  Ok("per-file reflink or copy")
}

/// Seeds only an absent compiler unit. The donor is never opened as writable storage.
pub fn seed_cache(location: &Path, donor: &Path, unit: &CacheDirectory) -> Result<&'static str> {
  let destination = location.join(unit.as_str());
  if destination.try_exists()? {
    return Ok("skipped: compiler unit already exists");
  }
  let source = donor.join(unit.as_str());
  if !source.is_dir() {
    return Ok("skipped: donor compiler unit is absent");
  }
  fs::create_dir_all(location)?;
  for _ in 0..3 {
    // A dead owner's unfinished commit is also unsafe without recovery, which would write to the donor.
    if source.join(".temp/state.lock").try_exists()?
      || source.join(".temp/commit.lock").try_exists()?
    {
      return Ok("skipped: donor has a pending transaction");
    }
    let mut before = BTreeMap::new();
    before.insert(donor.join("_meta"), fs::read(donor.join("_meta"))?);
    generations(&source, &mut before)?;
    let staging = tempfile::Builder::new()
      .prefix(&format!("{}.seeding-{}-", unit, std::process::id()))
      .tempdir_in(location)?;
    let staged_unit = staging.path().join(unit.as_str());
    let outcome = clone_directory(&source, &staged_unit)?;
    let mut after = BTreeMap::new();
    after.insert(donor.join("_meta"), fs::read(donor.join("_meta"))?);
    generations(&source, &mut after)?;
    if before != after
      || source.join(".temp/state.lock").try_exists()?
      || source.join(".temp/commit.lock").try_exists()?
    {
      continue;
    }
    match fs::rename(&staged_unit, &destination) {
      Ok(()) => return Ok(outcome),
      Err(error)
        if matches!(
          error.kind(),
          ErrorKind::AlreadyExists | ErrorKind::DirectoryNotEmpty
        ) =>
      {
        return Ok("skipped: another compiler seeded the unit");
      }
      Err(error) => return Err(error.into()),
    }
  }
  Ok("skipped: donor changed during three seeding attempts")
}
