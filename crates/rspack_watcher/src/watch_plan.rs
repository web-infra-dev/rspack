use notify::RecursiveMode;
use rspack_paths::{InternedPath, InternedPathMap, InternedPathSet};

use crate::WatchPattern;

/// Backend watch roots, with one entry per path and recursive requirements merged.
/// Dependency and event-alias paths remain in PathManager; preparing a plan never
/// changes the paths reported to callers.
pub(crate) struct WatchPlan {
  patterns: InternedPathMap<RecursiveMode>,
}

impl WatchPlan {
  pub(crate) fn new(
    patterns: impl IntoIterator<Item = WatchPattern>,
    follow_symlinks: bool,
  ) -> Self {
    let mut plan = Self {
      patterns: InternedPathMap::default(),
    };
    // Merge repeated requirements before doing filesystem work on their roots.
    for pattern in patterns {
      plan.insert(pattern.path, pattern.mode);
    }
    if follow_symlinks && !cfg!(target_os = "windows") {
      // inotify shares descriptors for aliases. FSEvents keys its recursive
      // modes by canonical path. Both need physical roots merged before watch().
      // Windows prefixes events with the supplied root, so keep that spelling
      // consistent with the link index instead.
      for (path, mode) in std::mem::take(&mut plan.patterns) {
        let path = dunce::canonicalize(&path).map_or(path, InternedPath::from);
        plan.insert(path, mode);
      }
      // Recursive inotify watches share descriptors with explicit child watches.
      // Keep only the ancestor so adding or removing a child cannot weaken it.
      let recursive_roots: InternedPathSet = plan
        .patterns
        .iter()
        .filter(|(_, mode)| **mode == RecursiveMode::Recursive)
        .map(|(path, _)| path.clone())
        .collect();
      plan.patterns.retain(|path, _| {
        !path
          .ancestors()
          .skip(1)
          .any(|parent| recursive_roots.contains(&InternedPath::from(parent)))
      });
    }
    plan
  }

  fn insert(&mut self, path: InternedPath, mode: RecursiveMode) {
    self
      .patterns
      .entry(path)
      .and_modify(|current| {
        if mode == RecursiveMode::Recursive {
          *current = mode;
        }
      })
      .or_insert(mode);
  }

  pub(crate) fn into_patterns(self) -> InternedPathMap<RecursiveMode> {
    self.patterns
  }
}
