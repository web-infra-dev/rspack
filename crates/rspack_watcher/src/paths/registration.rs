use std::{
  sync::{Arc, Weak},
  time::SystemTime,
};

use rspack_paths::InternedPath;
use rspack_util::time::system_time_to_millis;

use super::{PathManager, PathTracker, entries::is_absent};
use crate::{
  FsEvent, FsEventKind,
  directory_watcher::FileObservation,
  observation::{ActiveWalk, Mutation},
  time::{mtime_accuracy, mtime_safe_time},
};

#[derive(Clone, Copy)]
enum Kind {
  File,
  Directory,
  Missing,
}

pub(crate) struct Registration {
  pub(crate) path: InternedPath,
  kind: Kind,
  subscriber: Weak<InternedPath>,
}

pub(crate) struct Backfill {
  direct: Weak<InternedPath>,
  pub(crate) recipients: Vec<Weak<InternedPath>>,
}

impl Backfill {
  pub(crate) fn active(&self, paths: &PathManager) -> bool {
    self
      .direct
      .upgrade()
      .is_some_and(|sub| paths.current_subscriber(&sub))
  }
  pub(crate) fn covers(&self, paths: &PathManager, path: &InternedPath) -> bool {
    self
      .recipients
      .iter()
      .filter_map(Weak::upgrade)
      .any(|s| s.as_ref() == path && paths.current_subscriber(&s))
  }
}

impl PathManager {
  pub(crate) fn current_subscriber(&self, sub: &Arc<InternedPath>) -> bool {
    self.files.is_scan_subscriber(sub)
      || self.missing.is_scan_subscriber(sub)
      || self.directories.is_scan_subscriber(sub)
      || self.watchers.is_context_subscriber(sub)
  }

  fn subscribers(&self, path: &InternedPath) -> [Option<Weak<InternedPath>>; 3] {
    [
      self.files.scan_subscriber(path),
      self.directories.scan_subscriber(path),
      self.missing.scan_subscriber(path),
    ]
  }

  pub(crate) fn is_registered(&self, path: &InternedPath) -> bool {
    self.files.all.contains(path)
      || self.directories.all.contains(path)
      || self.missing.all.contains(path)
  }

  fn tracker(&self, kind: Kind) -> &PathTracker {
    match kind {
      Kind::File => &self.files,
      Kind::Directory => &self.directories,
      Kind::Missing => &self.missing,
    }
  }

  pub(crate) fn added_registrations(&self, mut add: impl FnMut(Registration)) {
    for kind in [Kind::File, Kind::Directory, Kind::Missing] {
      let tracker = self.tracker(kind);
      for path in tracker.added.iter() {
        if let Some(subscriber) = tracker.scan_subscriber(&path) {
          add(Registration {
            path: path.clone(),
            kind,
            subscriber,
          });
        }
      }
    }
  }

  pub(crate) fn scan_registered(
    &self,
    entry: Registration,
    start: SystemTime,
    batch: &ActiveWalk,
  ) -> Option<(FsEvent, Backfill)> {
    let path = &entry.path;
    let subscriber = entry.subscriber.upgrade()?;
    let read = || {
      let metadata = path.metadata();
      let absent = metadata.as_ref().is_err_and(is_absent)
        && (!matches!(entry.kind, Kind::File)
          || std::fs::symlink_metadata(path).is_err_and(|e| is_absent(&e)));
      (
        metadata
          .ok()
          .and_then(|m| m.modified().or_else(|_| m.created()).ok()),
        absent,
      )
    };
    let ((mtime, missing_on_disk), mut state) = loop {
      let observation = read();
      let mut state = self.watchers.lock_observations();
      if !state.accepts(batch, path) || !self.tracker(entry.kind).is_scan_subscriber(&subscriber) {
        return None;
      }
      if !state.take_metadata_recheck(batch, path) {
        break (observation, state);
      }
    };
    let changed = mtime.is_some_and(|mtime| {
      mtime_accuracy(mtime);
      mtime_safe_time(mtime) >= system_time_to_millis(start)
    });
    let absent = missing_on_disk
      && !self.missing.all.contains(path)
      && (!matches!(entry.kind, Kind::File) || !self.is_known_absent(path));
    if missing_on_disk && (absent || self.file_times.get(path).is_some()) {
      self.forget_removed_path(path);
      state.observe_live(path, Mutation::Subtree);
    } else if !matches!(entry.kind, Kind::Directory)
      && let Some(mtime) = mtime
    {
      self.record(path, || {
        self
          .watchers
          .set_file_time(path, Some(mtime), FileObservation::Backfill { changed })
      });
      state.observe_live(path, Mutation::Metadata);
    }
    let kind = if absent {
      FsEventKind::Remove
    } else if changed {
      if matches!(entry.kind, Kind::Missing) {
        FsEventKind::Create
      } else {
        FsEventKind::Change
      }
    } else {
      return None;
    };
    let mut recipients = Vec::from_iter(self.subscribers(path).into_iter().flatten());
    for parent in path.ancestors().skip(1) {
      let parent = InternedPath::from(parent);
      recipients.extend(
        [
          self.directories.scan_subscriber(&parent),
          self.missing.scan_subscriber(&parent),
        ]
        .into_iter()
        .flatten(),
      );
    }
    Some((
      FsEvent {
        path: entry.path,
        kind,
      },
      Backfill {
        direct: Arc::downgrade(&subscriber),
        recipients,
      },
    ))
  }
}
