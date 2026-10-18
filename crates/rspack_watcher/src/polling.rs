use std::{fs::FileType, time::SystemTime};

use rspack_paths::{InternedPath, InternedPathMap, InternedPathSet};

use crate::registry::PathAccessor;

#[derive(Default)]
pub(crate) struct PollingWatcher {
  entries: InternedPathMap<Option<(FileType, Option<SystemTime>)>>,
}

impl PollingWatcher {
  pub fn update(&mut self, accessor: PathAccessor<'_>) {
    let sets = [accessor.files(), accessor.directories(), accessor.missing()];
    let mut changed = InternedPathSet::default();
    for (_, added, removed) in &sets {
      changed.extend(added.iter().chain(removed.iter()).map(|path| path.clone()));
    }
    for path in changed {
      if sets.iter().any(|(all, _, _)| all.contains(&path)) {
        self
          .entries
          .entry(path.clone())
          .or_insert_with(|| snapshot(&path));
      } else {
        self.entries.remove(&path);
      }
    }
  }

  pub fn scan(&mut self) -> InternedPathSet {
    let mut changed = InternedPathSet::default();
    for (path, previous) in &mut self.entries {
      let current = snapshot(path);
      if current != *previous {
        changed.insert(path.clone());
        *previous = current;
      }
    }
    changed
  }
}

fn snapshot(path: &InternedPath) -> Option<(FileType, Option<SystemTime>)> {
  std::fs::metadata(path)
    .or_else(|_| std::fs::symlink_metadata(path))
    .ok()
    .map(|metadata| (metadata.file_type(), metadata.modified().ok()))
}
