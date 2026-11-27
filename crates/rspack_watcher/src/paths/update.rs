use std::path::PathBuf;

use rspack_error::Result;
use rspack_paths::InternedPath;

use super::PathTracker;
use crate::{ignored::IgnoredMatcher, observation::Observations, watcher_manager::WatcherManager};

/// `PathUpdater` is used to update collections of registered paths (files, directories, and missing paths)
/// by specifying which paths have been added and which have been removed. It holds vectors of paths to be
/// added and removed, and provides functionality to apply these changes to a path tracker. This struct
/// facilitates batch updates to the path sets, ensuring that additions and removals are processed efficiently.
pub(super) struct PathUpdater<'a> {
  pub added: Vec<InternedPath>,
  pub removed: Vec<InternedPath>,
  base_dir: PathBuf,
  manager: Option<(&'a WatcherManager, bool)>,
}

impl<Added, Removed> From<(Added, Removed)> for PathUpdater<'_>
where
  Added: Iterator<Item = InternedPath>,
  Removed: Iterator<Item = InternedPath>,
{
  fn from((added, removed): (Added, Removed)) -> Self {
    Self {
      added: added.collect(),
      removed: removed.collect(),
      base_dir: std::env::current_dir().unwrap_or_default(),
      manager: None,
    }
  }
}

impl<'a> PathUpdater<'a> {
  pub(super) fn with_manager(mut self, manager: &'a WatcherManager, context: bool) -> Self {
    self.manager = Some((manager, context));
    self
  }

  /// Update the paths in the given set. Returns the paths it newly holds and
  /// those it no longer does.
  pub(super) async fn update(
    self,
    watch_tracker: &PathTracker,
    ignored: &IgnoredMatcher,
  ) -> Result<(Vec<InternedPath>, Vec<InternedPath>)> {
    let added_paths = self.added;
    let removed_paths = self.removed;
    let mut registered = Vec::new();
    let mut unregistered = Vec::new();

    for added in added_paths {
      // Absolutize before the ignored check so registration filters on the
      // same absolute path the watcher reports for events.
      let added = if added.is_absolute() {
        added
      } else {
        InternedPath::from(self.base_dir.join(added.as_ref()))
      };

      // Skip ignored paths AND anything inside an ignored directory.
      if ignored
        .is_ignored(added.to_str().expect("Path should be valid UTF-8"))
        .await
      {
        continue;
      }

      if change_registration(
        self.manager,
        &added,
        || watch_tracker.add(added.clone()),
        |manager, state| manager.subscribe_context(&added, state),
      ) {
        registered.push(added);
      }
    }

    for removed in removed_paths {
      let removed = if removed.is_absolute() {
        removed
      } else {
        InternedPath::from(self.base_dir.join(removed.as_ref()))
      };
      if change_registration(
        self.manager,
        &removed,
        || watch_tracker.remove(removed.clone()),
        |manager, state| manager.unsubscribe_context(&removed, state),
      ) {
        unregistered.push(removed);
      }
    }
    Ok((registered, unregistered))
  }
}

// A targeted scan can check its subscription and update the record in one observation lock.
fn change_registration(
  manager: Option<(&WatcherManager, bool)>,
  path: &InternedPath,
  change: impl FnOnce() -> bool,
  update_context: impl FnOnce(&WatcherManager, &mut Observations),
) -> bool {
  let Some((manager, context)) = manager else {
    return change();
  };
  let mut state = manager.lock_observations();
  let changed = change();
  if changed {
    if context {
      update_context(manager, &mut state);
    } else {
      state.observe_live(path, crate::observation::Mutation::Metadata);
    }
  }
  changed
}
