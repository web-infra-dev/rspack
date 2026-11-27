use std::{sync::Arc, time::SystemTime};

use rspack_paths::InternedPath;

use super::{PathManager, entries::is_absent, time_info::ReportedBaseline};
use crate::{FsEventKind, observation::Mutation, watcher_manager::ScannedEntry};

impl PathManager {
  pub(crate) async fn recheck_scanned_entry(
    &self,
    path: &InternedPath,
    original: ScannedEntry,
  ) -> Option<ScannedEntry> {
    let (entry, changed_type) = self.observe_live(path, || {
      if !self.files.all.contains(path)
        && !self.missing.all.contains(path)
        && !self.may_contain_records(path)
        && !self.watchers.has_member(path, |_| true)
      {
        return ((None, false), Mutation::Unchanged);
      }
      match std::fs::symlink_metadata(path) {
        Ok(metadata) => {
          let mtime = metadata.modified().or_else(|_| metadata.created()).ok();
          let directory = metadata.is_dir();
          let entry = if directory {
            ScannedEntry::Directory(mtime)
          } else {
            ScannedEntry::File
          };
          let replaced = directory != original.directory()
            || original.kind() == FsEventKind::Remove
            || metadata.is_symlink();
          let updated = mtime.is_some_and(|mtime| {
            self
              .file_times
              .get(path)
              .is_some_and(|old| old.mtime != mtime)
              && self.store_file_time(path, mtime, false, true)
          });
          (
            (Some(entry), replaced),
            if updated {
              Mutation::Entry
            } else {
              Mutation::Metadata
            },
          )
        }
        Err(error) if is_absent(&error) => {
          self.forget_removed_path(path);
          (
            (
              Some(ScannedEntry::Missing {
                directory: original.directory(),
              }),
              false,
            ),
            Mutation::Subtree,
          )
        }
        Err(_) => ((None, false), Mutation::Unchanged),
      }
    });
    if changed_type {
      self.set_context_entry(path, false, None).await;
    }
    entry
  }

  pub(crate) fn is_scanned_file_recipient(&self, subscriber: &Arc<InternedPath>) -> bool {
    self.files.is_scan_subscriber(subscriber) || self.missing.is_scan_subscriber(subscriber)
  }

  pub(crate) fn scan_event_for(
    &self,
    path: &InternedPath,
    subscriber: &Arc<InternedPath>,
    entry: ScannedEntry,
  ) -> Option<FsEventKind> {
    let dep = subscriber.as_ref();
    let missing = self.missing.is_scan_subscriber(subscriber);
    if dep != path {
      if !missing && !self.watchers.is_context_subscriber(subscriber) {
        return None;
      }
      return Some(
        if self.absent_directories.contains(dep) || (missing && self.is_known_absent(dep)) {
          FsEventKind::Remove
        } else {
          FsEventKind::Change
        },
      );
    }
    let eligible = missing
      || match entry {
        ScannedEntry::File => self.files.is_scan_subscriber(subscriber),
        ScannedEntry::Directory(_) => self.watchers.is_context_subscriber(subscriber),
        ScannedEntry::Missing { .. } => {
          self.files.is_scan_subscriber(subscriber)
            || self.watchers.is_context_subscriber(subscriber)
        }
      };
    eligible.then(|| entry.kind())
  }

  pub(crate) fn report_scan_delivery(
    &self,
    path: &InternedPath,
    recipients: &[Arc<InternedPath>],
    scanned_mtime: Option<SystemTime>,
    entry: ScannedEntry,
  ) {
    let _index = self.lock_index();
    match entry {
      ScannedEntry::Directory(Some(mtime)) if Some(mtime) == scanned_mtime => {
        self.missing.with_scan_subscriber(path, recipients, || {
          self.file_times.update(path, |record| {
            if record.time.is_some_and(|time| time.mtime == mtime) {
              record.report_change(mtime);
            }
          })
        });
      }
      ScannedEntry::Missing { .. } => {
        let mark_absent = || {
          self.file_times.update(path, |record| {
            if record.time.is_none() {
              record.reported = ReportedBaseline::Absent;
            }
          })
        };
        self
          .files
          .with_scan_subscriber(path, recipients, mark_absent);
        self
          .missing
          .with_scan_subscriber(path, recipients, mark_absent);
      }
      _ => {}
    }
  }
}
