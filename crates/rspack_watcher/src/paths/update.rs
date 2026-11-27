use std::path::PathBuf;

use rspack_error::Result;
use rspack_paths::InternedPath;

use super::PathTracker;
use crate::{ignored::IgnoredMatcher, watcher_manager::WatcherManager};

/// `PathUpdater` is used to update collections of registered paths (files, directories, and missing paths)
/// by specifying which paths have been added and which have been removed. It holds vectors of paths to be
/// added and removed, and provides functionality to apply these changes to a path tracker. This struct
/// facilitates batch updates to the path sets, ensuring that additions and removals are processed efficiently.
pub(super) struct PathUpdater<'a> {
  pub added: Vec<InternedPath>,
  pub removed: Vec<InternedPath>,
  base_dir: PathBuf,
  contexts: Option<&'a WatcherManager>,
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
      contexts: None,
    }
  }
}

impl<'a> PathUpdater<'a> {
  pub(super) fn with_contexts(mut self, contexts: &'a WatcherManager) -> Self {
    self.contexts = Some(contexts);
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

      if watch_tracker.add(added.clone()) {
        if let Some(contexts) = self.contexts {
          contexts.subscribe_context(&added);
        }
        registered.push(added);
      }
    }

    for removed in removed_paths {
      let removed = if removed.is_absolute() {
        removed
      } else {
        InternedPath::from(self.base_dir.join(removed.as_ref()))
      };
      if watch_tracker.remove(removed.clone()) {
        if let Some(contexts) = self.contexts {
          contexts.unsubscribe_context(&removed);
        }
        unregistered.push(removed);
      }
    }
    Ok((registered, unregistered))
  }
}
