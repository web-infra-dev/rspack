use std::{path::PathBuf, time::SystemTime};

use rspack_paths::{InternedPath, InternedPathSet};

use super::{PathManager, followed_metadata, modification_time};
use crate::{EventBatch, FsEvent, FsEventKind};

impl PathManager {
  /// Compare an explicitly watched entry with the last delivered observation.
  /// A context scan may have updated its time table before its own event arrives.
  fn check_file(&self, path: &InternedPath) -> Option<FsEventKind> {
    let mtime = match std::fs::symlink_metadata(path) {
      Ok(own) => Some(modification_time(followed_metadata(path, &own)?.as_ref())?),
      Err(error)
        if matches!(
          error.kind(),
          std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory
        ) =>
      {
        None
      }
      Err(_) => return None,
    };
    self.file_difference(path, mtime)
  }

  pub(crate) fn file_difference(
    &self,
    path: &InternedPath,
    mtime: Option<SystemTime>,
  ) -> Option<FsEventKind> {
    let known = self.baselines.get(path).map_or_else(
      || self.file_times.get(path).map(|time| time.mtime),
      |baseline| *baseline,
    );
    let Some(mtime) = mtime else {
      return known.map(|_| FsEventKind::Remove);
    };
    match known {
      Some(previous) if previous == mtime => None,
      Some(_) => Some(FsEventKind::Change),
      None => Some(FsEventKind::Create),
    }
  }

  pub async fn check_subscriptions(&self) -> EventBatch {
    let registered: InternedPathSet = self
      .files
      .all
      .iter()
      .chain(self.missing.all.iter())
      .map(|path| path.clone())
      .collect();
    let mut changes: EventBatch = registered
      .iter()
      .filter_map(|path| {
        self.check_file(path).map(|kind| FsEvent {
          path: path.clone(),
          kind,
        })
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
      let absent = self.absent_directories.contains(&path);
      match path.metadata() {
        Ok(metadata) if metadata.is_dir() => {
          if absent {
            changes.push(FsEvent {
              path,
              kind: FsEventKind::Create,
            });
          } else {
            pending.push((path, Vec::<PathBuf>::new()));
          }
        }
        Err(error)
          if !absent
            && matches!(
              error.kind(),
              std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory
            ) =>
        {
          changes.push(FsEvent {
            path,
            kind: FsEventKind::Remove,
          })
        }
        _ => {}
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
            changes.push(FsEvent {
              path,
              kind: FsEventKind::Create,
            });
          }
        } else if !registered.contains(&path)
          && let Some(mtime) = modification_time(&metadata)
        {
          let kind = match self.file_times.get(&path) {
            Some(old) if old.mtime == mtime => continue,
            Some(_) => FsEventKind::Change,
            None => FsEventKind::Create,
          };
          changes.push(FsEvent { path, kind });
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
            .map(|path| FsEvent {
              path,
              kind: FsEventKind::Remove,
            }),
        );
      }
    }
    changes
  }
}
