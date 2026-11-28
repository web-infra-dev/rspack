use std::time::SystemTime;

use rspack_paths::{InternedPath, InternedPathMap, InternedPathSet};
use rspack_util::time::current_time;

use crate::{
  paths::{FileRecord, ReportedBaseline},
  time::{fs_accuracy, mtime_accuracy, mtime_to_millis},
  watcher_manager::normalize_dir_path,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatcherSubKind {
  File,
  Missing,
  Directory,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatcherSub {
  pub kind: WatcherSubKind,
  pub logical_path: InternedPath,
  pub start_time: u64,
}

pub struct DirectoryWatcher {
  pub path: InternedPath,
  pub(crate) files: InternedPathMap<FileRecord>,
  pub directories: InternedPathSet,
  pub directory_safe_times: InternedPathMap<u64>,
  pub removed_times: InternedPathMap<u64>,
  pub watchers: InternedPathMap<Vec<WatcherSub>>,
  pub nested_watching: bool,
  pub exists_on_disk: bool,
  pub last_watch_event: u64,
  #[allow(dead_code)]
  pub initial_scan: bool,
  #[allow(dead_code)]
  pub initial_scan_finished: Option<u64>,
  pub refs: usize,
  pub context_files: InternedPathSet,
  pub parent_context: Option<InternedPath>,
}

impl DirectoryWatcher {
  pub fn new(path: InternedPath) -> Self {
    let path = normalize_dir_path(&path);
    let exists_on_disk = path.is_dir();
    Self {
      path,
      exists_on_disk,
      initial_scan: true,
      files: Default::default(),
      directories: Default::default(),
      directory_safe_times: Default::default(),
      removed_times: Default::default(),
      watchers: Default::default(),
      nested_watching: false,
      last_watch_event: 0,
      initial_scan_finished: None,
      refs: 0,
      context_files: Default::default(),
      parent_context: None,
    }
  }
  pub(crate) fn owner(p: &InternedPath) -> Option<InternedPath> {
    p.parent()
      .map(|d| normalize_dir_path(&InternedPath::from(d)))
  }
  pub(crate) fn owner_or_self(p: &InternedPath) -> InternedPath {
    Self::owner(p).unwrap_or_else(|| normalize_dir_path(p))
  }
  pub(crate) fn has_explicit_directory_sub(&self) -> bool {
    let (s, d) = (self.watchers.get(&self.path), WatcherSubKind::Directory);
    s.is_some_and(|v| v.iter().any(|x| x.kind == d))
  }
  pub(crate) fn logical_directories(&self) -> Vec<InternedPath> {
    let d = WatcherSubKind::Directory;
    let it = self.watchers.get(&self.path).into_iter().flatten();
    it.filter(|x| x.kind == d)
      .map(|x| x.logical_path.clone())
      .collect()
  }
  pub(crate) fn is_active_directory(&self) -> bool {
    self.has_explicit_directory_sub() || self.parent_context.is_some()
  }
  pub(crate) fn is_unused(&self) -> bool {
    let a = self.watchers.is_empty() && self.files.is_empty() && self.directories.is_empty();
    let b = self.refs == 0 && self.parent_context.is_none() && self.last_watch_event == 0;
    a && b && self.context_files.is_empty()
  }
  pub(crate) fn add_sub(&mut self, k: &InternedPath, kd: WatcherSubKind, l: &InternedPath, t: u64) {
    let subs = self.watchers.entry(k.clone()).or_default();
    if !subs.iter().any(|s| s.kind == kd && &s.logical_path == l) {
      let (kind, logical_path, start_time) = (kd, l.clone(), t);
      subs.push(WatcherSub {
        kind,
        logical_path,
        start_time,
      });
      self.refs += 1;
    }
  }
  pub(crate) fn remove_sub(&mut self, k: &InternedPath, kind: WatcherSubKind, l: &InternedPath) {
    if let Some(subs) = self.watchers.get_mut(k) {
      let n = subs.len();
      subs.retain(|s| !(s.kind == kind && &s.logical_path == l));
      self.refs = self.refs.saturating_sub(n - subs.len());
      subs.is_empty().then(|| self.watchers.remove(k));
    }
  }
  pub(crate) fn clear_file(&mut self, path: &InternedPath, clear: impl FnOnce(&mut FileRecord)) {
    if let Some(r) = self.files.get_mut(path) {
      clear(r);
      r.is_empty().then(|| self.files.remove(path));
    }
  }
  pub fn set_file_time(&mut self, p: &InternedPath, m: SystemTime, init: bool, eq: bool) -> bool {
    if !init
      && let Some(old) = self.files.get(p)
      && old.time.is_some_and(|t| t.mtime == m)
    {
      let aged = mtime_to_millis(m).saturating_add(mtime_accuracy(m)) < current_time();
      let r = matches!(old.reported, ReportedBaseline::Present(_));
      if aged && (r || self.context_files.contains(p)) {
        return false;
      }
    }
    let ok = self
      .files
      .entry(p.clone())
      .or_default()
      .set_time(m, init, eq);
    ok.then(|| self.removed_times.remove(p));
    ok
  }
  pub fn set_directory(&mut self, dir_path: &InternedPath, birth: SystemTime, init: bool) -> bool {
    let dir = normalize_dir_path(dir_path);
    if dir == self.path || !self.directories.insert(dir.clone()) {
      return false;
    }
    self.context_files.remove(&dir);
    self.clear_file(&dir, FileRecord::clear_present);
    let mut safe = current_time();
    if init {
      safe = safe
        .min(mtime_to_millis(birth))
        .saturating_add(fs_accuracy());
    }
    self.directory_safe_times.insert(dir, safe);
    true
  }
  pub fn set_missing(&mut self, item: &InternedPath, init: bool) -> bool {
    let norm = normalize_dir_path(item);
    let was_dir = self.directories.remove(&norm);
    was_dir.then(|| self.directory_safe_times.remove(&norm));
    let mut hit = false;
    for k in [item, &norm] {
      hit |= self.context_files.remove(k);
      self.clear_file(k, |r| {
        hit |= r.time.is_some();
        r.clear_present();
      });
    }
    (hit && !init).then(|| self.removed_times.insert(norm, current_time()));
    was_dir || hit
  }
  pub fn set_nested_watching(&mut self, flag: bool) -> bool {
    std::mem::replace(&mut self.nested_watching, flag) != flag
  }
  pub fn on_directory_removed(&mut self) -> (Vec<InternedPath>, Vec<InternedPath>) {
    (self.exists_on_disk, self.last_watch_event) = (false, 0);
    let dirs: Vec<_> = self.directories.iter().cloned().collect();
    let files: Vec<_> = self.files.keys().cloned().collect();
    for p in dirs.iter().chain(&files) {
      self.set_missing(p, false);
    }
    self.context_files.clear();
    (dirs, files)
  }
}
