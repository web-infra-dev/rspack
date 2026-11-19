use std::{fs::Metadata, path::PathBuf};

use rspack_paths::{InternedPath, InternedPathIndexMap, InternedPathSet};

use super::{PathManager, followed_metadata, metadata_if_present, modification_time};
use crate::{EventBatch, FsEvent, FsEventKind};

impl PathManager {
  /// Compare an explicitly watched entry with the last delivered observation.
  /// A context scan may have updated its time table before its own event arrives.
  fn file_difference(
    &self,
    path: &InternedPath,
    present: bool,
    target: Option<&Metadata>,
  ) -> Option<FsEventKind> {
    let known = self
      .baselines
      .get(path)
      .and_then(|baseline| baseline.mtime());
    if !present {
      return known.map(|_| FsEventKind::Remove);
    }
    let mtime = modification_time(target?)?;
    match known {
      Some(previous) if previous == mtime => None,
      Some(_) => Some(FsEventKind::Change),
      None => Some(FsEventKind::Create),
    }
  }

  fn directory_difference(&self, path: &InternedPath, is_dir: bool) -> Option<FsEventKind> {
    match (self.absent_directories.contains(path), is_dir) {
      (true, true) => Some(FsEventKind::Create),
      (false, false) => Some(FsEventKind::Remove),
      _ => None,
    }
  }

  fn subscription_difference(
    &self,
    path: &InternedPath,
    entry: Option<&Metadata>,
    appeared: bool,
  ) -> Option<FsEventKind> {
    let file = self.has_file_or_missing_subscription(path);
    let directory = self.directories.all.contains(path);
    if appeared && entry.is_some() && (file || directory) {
      return Some(FsEventKind::Create);
    }
    let target = followed_metadata(path, entry).ok()?;
    if directory
      && let Some(kind) =
        self.directory_difference(path, target.as_deref().is_some_and(Metadata::is_dir))
    {
      return Some(kind);
    }
    if file {
      return self.file_difference(path, entry.is_some(), target.as_deref());
    }
    None
  }

  /// A directory hint can be the only event for its registered descendants.
  /// Collect differences before processing a removal can clear the subtree's records.
  pub async fn check_registered_children(
    &self,
    directory: &InternedPath,
    appeared: bool,
  ) -> EventBatch {
    let mut pending = self.subscriptions.registered_children(directory);
    let mut changes = EventBatch::new();
    if pending.is_empty() {
      return changes;
    }
    let appeared = match metadata_if_present(directory.symlink_metadata()) {
      Ok(Some(entry)) if !entry.is_symlink() => appeared && entry.is_dir(),
      Ok(None) => false,
      _ => return changes,
    };
    while let Some(path) = pending.pop() {
      if self.is_ignored_path(&path).await {
        continue;
      }
      let Ok(entry) = metadata_if_present(path.symlink_metadata()) else {
        continue;
      };
      if entry.as_ref().is_none_or(|entry| !entry.is_symlink()) {
        pending.extend(self.subscriptions.registered_children(&path));
      }
      if let Some(kind) = self.subscription_difference(&path, entry.as_ref(), appeared) {
        changes.push(FsEvent { path, kind });
      }
    }
    changes
  }

  pub async fn check_subscriptions(&self) -> EventBatch {
    let registered: InternedPathSet = self
      .files
      .all
      .iter()
      .chain(self.missing.all.iter())
      .map(|path| path.clone())
      .collect();
    let mut changes: InternedPathIndexMap<_> = registered
      .iter()
      .filter_map(|path| {
        let entry = metadata_if_present(path.symlink_metadata()).ok()?;
        self
          .subscription_difference(path, entry.as_ref(), false)
          .map(|kind| (path.clone(), kind))
      })
      .collect();
    let contexts: Vec<_> = self
      .directories
      .all
      .iter()
      .map(|path| path.clone())
      .collect();
    let mut pending = Vec::new();
    for path in contexts {
      let Ok(metadata) = metadata_if_present(path.metadata()) else {
        continue;
      };
      let is_dir = metadata.as_ref().is_some_and(Metadata::is_dir);
      if let Some(kind) = self.directory_difference(&path, is_dir) {
        changes.insert(path, kind);
      } else if is_dir {
        pending.push((path, Vec::<PathBuf>::new()));
      }
    }
    let mut visited = InternedPathSet::default();
    while let Some((dir, mut ancestors)) = pending.pop() {
      if !visited.insert(dir.clone()) {
        continue;
      }
      let Some(entries) = self.read_context_directory(&dir, &mut ancestors) else {
        continue;
      };
      let mut previous = self
        .lock_recorded_children()
        .get(&dir)
        .cloned()
        .unwrap_or_default();
      let mut complete = true;
      for entry in entries {
        let Ok(entry) = entry else {
          complete = false;
          continue;
        };
        let path = InternedPath::from(entry.path());
        previous.remove(&path);
        let Some(metadata) = self.context_metadata(&path, &ancestors).await else {
          continue;
        };
        if metadata.is_dir() {
          if self.context_directories.contains(&path) || self.directories.all.contains(&path) {
            pending.push((path, ancestors.clone()));
          } else {
            changes.entry(path).or_insert(FsEventKind::Create);
          }
        } else if !registered.contains(&path)
          && let Some(mtime) = modification_time(&metadata)
        {
          let kind = match self.file_times.get(&path) {
            Some(old) if old.mtime == mtime => continue,
            Some(_) => FsEventKind::Change,
            None => FsEventKind::Create,
          };
          changes.entry(path).or_insert(kind);
        }
      }
      if complete {
        changes.extend(
          previous
            .into_iter()
            .filter(|p| {
              !registered.contains(p)
                && !self.directories.all.contains(p)
                && (self.context_files.contains(p) || self.context_directories.contains(p))
            })
            .map(|path| (path, FsEventKind::Remove)),
        );
      }
    }
    changes
      .into_iter()
      .map(|(path, kind)| FsEvent { path, kind })
      .collect()
  }
}
