use std::{collections::HashSet, path::PathBuf};

use notify::{Event, RecommendedWatcher, Watcher};

use crate::watcher::{trigger, FsEventKind};

pub struct DiskWatcher {
  inner: RecommendedWatcher,
  watch_paths: HashSet<PathBuf>,
}

impl DiskWatcher {
  pub fn new<'a>(follow_symlinks: bool, trigger: trigger::Trigger) -> Self {
    let config = notify::Config::default().with_follow_symlinks(follow_symlinks);
    let inner = RecommendedWatcher::new(
      move |result: notify::Result<Event>| match result {
        Ok(event) => {
          let kind = match event.kind {
            notify::EventKind::Modify(_) => FsEventKind::Change,
            notify::EventKind::Remove(_) => FsEventKind::Delete,
            // TODO: Handle other kinds of events if needed
            _ => return, // Ignore other kinds of events
          };
          let paths = event.paths;
          for path in paths {
            trigger.on_event(&path, kind);
          }
        }

        Err(e) => {
          eprintln!("Error occurred while watching disk: {}", e);
        }
      },
      config,
    )
    .expect("Failed to create disk watcher");

    DiskWatcher {
      inner,
      watch_paths: HashSet::new(),
    }
  }

  pub fn watch(&mut self, path: &PathBuf, mode: notify::RecursiveMode) -> rspack_error::Result<()> {
    let mut wathing = false;
    for p in self.watch_paths.iter() {
      if p == path {
        wathing = true;
      }
      self.inner.unwatch(p).map_err(|e| rspack_error::error!(e))?;
    }

    if !wathing {
      self
        .inner
        .watch(path, mode)
        .map_err(|e| rspack_error::error!(e))?;
    }

    Ok(())
  }
}
