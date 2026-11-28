#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
use std::{path::Path, sync::Arc, time::SystemTime};

use dashmap::mapref::one::{Ref, RefMut};
use rspack_paths::{InternedPath, InternedPathDashMap};

use crate::directory_watcher::{DirectoryWatcher, WatcherSubKind};

#[derive(Clone, Default)]
pub struct DirectorySubscriptions {
  pub explicit: bool,
  pub parent: Option<InternedPath>,
}

pub trait ContextLookup {
  fn subscriptions(&self, path: &InternedPath) -> DirectorySubscriptions;
  fn logical_directories(&self, path: &InternedPath) -> Vec<InternedPath> {
    vec![path.clone()]
  }
}

pub fn normalize_dir_path(path: &InternedPath) -> InternedPath {
  let b = path.as_os_str().as_encoded_bytes();
  if !matches!(b.last(), Some(b'/' | b'\\')) {
    return path.clone();
  }
  #[cfg(windows)]
  {
    InternedPath::from(path.as_path().components().collect::<std::path::PathBuf>())
  }
  #[cfg(unix)]
  {
    let sep = |c: u8| matches!(c, b'/' | b'\\');
    if b.iter().all(|&c| sep(c)) {
      return InternedPath::from(Path::new("/"));
    }
    let drv =
      b.get(1) == Some(&b':') && b[0].is_ascii_alphabetic() && b.get(2).is_some_and(|&c| sep(c));
    let len = if drv && b[3..].iter().all(|&c| sep(c)) {
      3
    } else {
      b.iter().rposition(|&c| !sep(c)).map_or(0, |i| i + 1)
    };
    InternedPath::from(Path::new(std::ffi::OsStr::from_bytes(&b[..len])))
  }
}

#[derive(Default)]
pub struct WatcherManager {
  pub directories: Arc<InternedPathDashMap<DirectoryWatcher>>,
}

impl WatcherManager {
  fn dw_mut(&self, n: &InternedPath) -> RefMut<'_, InternedPath, DirectoryWatcher> {
    let d = &self.directories;
    d.entry(n.clone())
      .or_insert_with(|| DirectoryWatcher::new(n.clone()))
  }
  pub fn get(&self, p: &InternedPath) -> Option<Ref<'_, InternedPath, DirectoryWatcher>> {
    self.directories.get(&normalize_dir_path(p))
  }
  fn update_dw(&self, p: &InternedPath, f: impl FnOnce(&mut DirectoryWatcher)) {
    let (d, norm) = (&self.directories, normalize_dir_path(p));
    d.remove_if_mut(&norm, |_, w| {
      f(w);
      w.is_unused()
    });
  }
  pub fn watch_file(&self, p: &InternedPath, k: WatcherSubKind, t: u64) -> Option<InternedPath> {
    let dir = DirectoryWatcher::owner(p)?;
    self.dw_mut(&dir).add_sub(p, k, p, t);
    Some(dir)
  }
  pub fn unwatch_file(&self, p: &InternedPath, k: WatcherSubKind) {
    if let Some(dir) = DirectoryWatcher::owner(p) {
      self.update_dw(&dir, |dw| dw.remove_sub(p, k, p));
    }
  }
  pub fn watch_directory(&self, dir: &InternedPath, t: u64) -> InternedPath {
    let norm = normalize_dir_path(dir);
    let mut dw = self.dw_mut(&norm);
    dw.add_sub(&norm, WatcherSubKind::Directory, dir, t);
    dw.set_nested_watching(true);
    dw.exists_on_disk = norm.is_dir();
    norm
  }
  pub fn unwatch_directory(&self, dir: &InternedPath) {
    let norm = normalize_dir_path(dir);
    self.update_dw(&norm, |dw| {
      dw.remove_sub(&norm, WatcherSubKind::Directory, dir);
      (!dw.is_active_directory()).then(|| dw.set_nested_watching(false));
    });
  }
  pub(crate) fn clear_event_time(&self, p: &InternedPath) {
    self.update_dw(p, |w| w.last_watch_event = 0);
  }
  pub(crate) fn event_time(&self, p: &InternedPath) -> Option<u64> {
    self.get(p).map(|w| w.last_watch_event).filter(|&t| t > 0)
  }
  pub(crate) fn set_last_watch_event(&self, p: &InternedPath, now: u64) {
    if let Some(mut w) = self.directories.get_mut(&normalize_dir_path(p))
      && w.is_active_directory()
    {
      w.last_watch_event = now;
    }
  }
  pub(crate) fn is_explicit_context(&self, p: &InternedPath) -> bool {
    self.get(p).is_some_and(|w| w.has_explicit_directory_sub())
  }
  pub(crate) fn is_watched_directory(&self, p: &InternedPath) -> bool {
    self.get(p).is_some_and(|w| w.is_active_directory())
  }
  pub(crate) fn record_dir(&self, dir: &InternedPath, b: SystemTime, i: bool) -> bool {
    let norm = normalize_dir_path(dir);
    let Some(parent) = DirectoryWatcher::owner(&norm) else {
      return false;
    };
    let added = self.dw_mut(&parent).set_directory(&norm, b, i);
    let mut child = self.dw_mut(&norm);
    (child.parent_context, child.exists_on_disk) = (Some(parent), true);
    child.set_nested_watching(true);
    added
  }
  pub(crate) fn put_file(&self, p: &InternedPath, m: Option<SystemTime>, i: bool, e: bool) -> bool {
    let norm = normalize_dir_path(p);
    let Some(parent) = DirectoryWatcher::owner(&norm) else {
      return false;
    };
    let mut dw = self.dw_mut(&parent);
    let updated = m.is_none_or(|t| dw.set_file_time(&norm, t, i, e));
    dw.context_files.insert(norm);
    updated
  }
  pub(crate) fn has_member(&self, p: &InternedPath) -> bool {
    let norm = normalize_dir_path(p);
    DirectoryWatcher::owner(&norm)
      .and_then(|d| self.directories.get(&d))
      .is_some_and(|w| w.directories.contains(&norm) || w.context_files.contains(&norm))
  }
  pub(crate) fn del_member(&self, p: &InternedPath, i: bool, k: Option<bool>) -> (bool, bool) {
    let norm = normalize_dir_path(p);
    let (mut wf, mut wd) = (false, false);
    if let Some(parent) = DirectoryWatcher::owner(&norm) {
      self.update_dw(&parent, |w| {
        if k == Some(false) {
          wd = w.directories.contains(&norm);
          wf = w.context_files.contains(&norm);
          w.set_missing(&norm, i);
        } else {
          wd = w.directories.remove(&norm);
          wd.then(|| w.directory_safe_times.remove(&norm));
          wf = k == Some(true) && w.context_files.remove(&norm);
        }
      });
    }
    if wd || k.is_none() {
      self.unsubscribe_parent(&norm);
    }
    (wf, wd)
  }
  pub(crate) fn unsubscribe_parent(&self, p: &InternedPath) {
    self.update_dw(p, |w| {
      w.parent_context = None;
      (!w.has_explicit_directory_sub()).then(|| w.set_nested_watching(false));
    });
  }
  pub(crate) fn cleanup_if_unused(&self, p: &InternedPath) {
    self.update_dw(p, |_| {});
  }
  pub(crate) fn on_directory_removed(&self, dir: &InternedPath) {
    let mut removed = Vec::new();
    self.update_dw(dir, |dw| removed = dw.on_directory_removed().0);
    removed.iter().for_each(|sub| self.unsubscribe_parent(sub));
  }
}

impl ContextLookup for WatcherManager {
  fn subscriptions(&self, p: &InternedPath) -> DirectorySubscriptions {
    let Some(w) = self.get(p) else {
      return Default::default();
    };
    let (explicit, parent) = (w.has_explicit_directory_sub(), w.parent_context.clone());
    DirectorySubscriptions { explicit, parent }
  }
  fn logical_directories(&self, p: &InternedPath) -> Vec<InternedPath> {
    self.get(p).map_or(vec![], |w| w.logical_directories())
  }
}
