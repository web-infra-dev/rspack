use std::{
  sync::{Arc, Mutex, Weak},
  time::SystemTime,
};

use rspack_paths::{InternedPath, InternedPathDashSet, InternedPathMap, InternedPathSet};
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};

use super::{FsEvent, FsEventKind};
use crate::{
  EventBatch, ResolvedEvents,
  analyzer::{Analyzer, RecommendedAnalyzer},
  disk_watcher::DiskWatcher,
  paths::{
    ContextEntryOutcome, LiveEntryFlags, PathManager, disk_mtime,
    links::{DepKind, EntrySubscriber, LinkState, push_unique},
  },
  refresh_disk_watcher,
};

#[derive(Clone, Copy)]
enum LiveSource {
  Disk {
    renamed: bool,
    directory_probe: bool,
  },
  Injected,
}

#[derive(Clone, Copy)]
struct LiveEventFlags {
  source: LiveSource,
  appeared: bool,
  directory_replaced: bool,
}

pub struct DependencyFinder<'a> {
  pub files: &'a InternedPathDashSet,
  pub directories: &'a InternedPathDashSet,
  pub missing: &'a InternedPathDashSet,
}

impl<'a> DependencyFinder<'a> {
  pub fn find_associated_event(
    &self,
    path: &InternedPath,
    kind: FsEventKind,
  ) -> Vec<(InternedPath, FsEventKind)> {
    let mut paths = Vec::new();

    if path.exists() {
      if path.is_dir() && self.contains_directory(path) {
        paths.push((path.clone(), kind));
      }

      if path.is_file() && self.contains_file(path) {
        paths.push((path.clone(), kind));
      }
    } else if self.contains_path(path) {
      paths.push((path.clone(), kind));
    }

    self.recurse_parent_directories(path, &mut paths);

    paths
  }

  fn contains_file(&self, path: &InternedPath) -> bool {
    self.files.contains(path) || self.missing.contains(path)
  }

  fn contains_directory(&self, path: &InternedPath) -> bool {
    self.directories.contains(path) || self.missing.contains(path)
  }

  fn contains_path(&self, path: &InternedPath) -> bool {
    self.files.contains(path) || self.directories.contains(path) || self.missing.contains(path)
  }

  fn recurse_parent_directories(
    &self,
    path: &InternedPath,
    paths: &mut Vec<(InternedPath, FsEventKind)>,
  ) {
    if let Some(parent) = path.parent() {
      let parent_interned = InternedPath::from(parent);
      if self.contains_directory(&parent_interned) {
        paths.push((parent_interned.clone(), FsEventKind::Change));
      }
      self.recurse_parent_directories(&parent_interned, paths);
    }
  }
}

struct EventProcessor {
  path_manager: Arc<PathManager>,
  tx: UnboundedSender<ResolvedEvents>,
  analyzer: Arc<RecommendedAnalyzer>,
  disk_watcher: Arc<Mutex<Weak<Mutex<DiskWatcher>>>>,
}

pub struct Trigger {
  pending: UnboundedSender<(InternedPath, FsEventKind, Origin)>,
  pub(crate) analyzer: Arc<RecommendedAnalyzer>,
  disk_watcher: Arc<Mutex<Weak<Mutex<DiskWatcher>>>>,
}

#[derive(Clone)]
enum Origin {
  Disk {
    appeared: bool,
    renamed: bool,
    directory_probe: bool,
  },
  Injected,
  Scan,
  ContextScan {
    raw_path: InternedPath,
    root_context: InternedPath,
  },
}

impl Trigger {
  pub fn new(path_manager: Arc<PathManager>, tx: UnboundedSender<ResolvedEvents>) -> Self {
    let (pending, mut incoming) = unbounded_channel::<(InternedPath, FsEventKind, Origin)>();
    let analyzer = Arc::default();
    let disk_watcher = Arc::new(Mutex::new(Weak::new()));
    let processor = EventProcessor {
      path_manager,
      tx,
      analyzer: Arc::clone(&analyzer),
      disk_watcher: Arc::clone(&disk_watcher),
    };

    std::thread::Builder::new()
      .name("rspack-fs-event-filter".to_string())
      .spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
          .enable_all()
          .build()
          .expect("create watcher event filter runtime");

        runtime.block_on(async move {
          while let Some((path, kind, origin)) = incoming.recv().await {
            processor.process(&path, kind, origin).await;
          }
        });
      })
      .expect("spawn watcher event filter thread");

    Self {
      pending,
      analyzer,
      disk_watcher,
    }
  }

  pub(crate) fn set_disk_watcher(&self, disk_watcher: Weak<Mutex<DiskWatcher>>) {
    *self
      .disk_watcher
      .lock()
      .expect("trigger disk_watcher lock poisoned") = disk_watcher;
  }

  pub fn on_event(&self, path: &InternedPath, kind: FsEventKind) {
    self.analyzer.invalidate_path(path);
    let _ = self.pending.send((path.clone(), kind, Origin::Injected));
  }

  pub(crate) fn on_disk_event(
    &self,
    path: &InternedPath,
    kind: FsEventKind,
    appeared: bool,
    renamed: bool,
    directory_probe: bool,
  ) {
    let _ = self.pending.send((
      path.clone(),
      kind,
      Origin::Disk {
        appeared,
        renamed,
        directory_probe,
      },
    ));
  }

  #[cfg(feature = "testing")]
  pub(crate) fn on_scanned_change(&self, path: &InternedPath) {
    let _ = self
      .pending
      .send((path.clone(), FsEventKind::Change, Origin::Scan));
  }

  pub(crate) fn on_scanned_context_change(
    &self,
    raw_path: InternedPath,
    root_context: InternedPath,
  ) {
    let _ = self.pending.send((
      raw_path.clone(),
      FsEventKind::Change,
      Origin::ContextScan {
        raw_path,
        root_context,
      },
    ));
  }

  pub(crate) fn backfill_sender(&self) -> UnboundedSender<EventBatch> {
    let (tx, mut batches) = unbounded_channel::<EventBatch>();
    let pending = self.pending.clone();
    std::thread::Builder::new()
      .name("rspack-fs-scan-backfill".to_string())
      .spawn(move || {
        while let Some(batch) = batches.blocking_recv() {
          for event in batch {
            let _ = pending.send((event.path, event.kind, Origin::Scan));
          }
        }
      })
      .expect("spawn watcher scan backfill thread");
    tx
  }
}

impl EventProcessor {
  fn upgrade_disk_watcher(&self) -> Option<Arc<Mutex<DiskWatcher>>> {
    self
      .disk_watcher
      .lock()
      .expect("trigger disk_watcher lock poisoned")
      .upgrade()
  }

  fn sync_disk_watcher(&self) {
    if let Some(dw) = self.upgrade_disk_watcher() {
      let _ = refresh_disk_watcher(&dw, &self.analyzer, &self.path_manager);
    }
  }

  fn sweep_removed_path(&self, path: &InternedPath) {
    let may_contain = self.path_manager.may_contain_records(path);
    self.path_manager.remove_file_time(path);
    if may_contain {
      self.path_manager.remove_file_times_under(path);
    }
  }

  fn sweep_removed_context(&self, phys: Option<&InternedPath>, logical_dir: &InternedPath) {
    if phys != Some(logical_dir) && std::fs::symlink_metadata(logical_dir).is_ok() {
      self.path_manager.mark_context_target_absent(logical_dir);
    } else {
      self.sweep_removed_path(logical_dir);
    }
  }

  fn rebind_disk_watcher_prefix(&self, prefix: &std::path::Path) {
    if let Some(dw) = self.upgrade_disk_watcher() {
      dw.lock()
        .expect("disk_watcher lock poisoned")
        .rebind_prefix(prefix);
    }
  }

  async fn process(&self, path: &InternedPath, kind: FsEventKind, origin: Origin) {
    match origin {
      Origin::ContextScan {
        raw_path,
        root_context,
      } => {
        if !self
          .path_manager
          .is_registered(DepKind::Directory, &root_context)
        {
          return;
        }
        self
          .path_manager
          .set_last_watch_events_for_context(&raw_path, &root_context);
        self.analyzer.invalidate_path(&raw_path);
        self.analyzer.invalidate_path(&root_context);
        let _ = self.tx.send(ResolvedEvents {
          dependencies: vec![FsEvent {
            path: root_context,
            kind: FsEventKind::Change,
          }],
          raw: vec![FsEvent {
            path: raw_path,
            kind: FsEventKind::Change,
          }],
          recovered: Vec::new(),
          context_removal: Vec::new(),
        });
      }
      Origin::Scan => {
        self.process_scanned_event(path, kind).await;
      }
      Origin::Disk {
        appeared,
        renamed,
        directory_probe,
      } => {
        self
          .process_live_event(
            path,
            kind,
            LiveEventFlags {
              source: LiveSource::Disk {
                renamed,
                directory_probe,
              },
              appeared,
              directory_replaced: false,
            },
          )
          .await;
      }
      Origin::Injected => {
        let appeared = kind == FsEventKind::Create;
        self
          .process_live_event(
            path,
            kind,
            LiveEventFlags {
              source: LiveSource::Injected,
              appeared,
              directory_replaced: false,
            },
          )
          .await;
      }
    }
  }

  async fn process_scanned_event(&self, path: &InternedPath, kind: FsEventKind) {
    if !self.path_manager.is_any_registered_or_below_context(path) {
      return;
    }

    let exists = if self.path_manager.is_registered(DepKind::File, path) {
      self
        .path_manager
        .is_dep_present_on_disk(DepKind::File, path)
    } else if self.path_manager.is_registered(DepKind::Directory, path) {
      self
        .path_manager
        .is_dep_present_on_disk(DepKind::Directory, path)
    } else {
      std::fs::symlink_metadata(path).is_ok()
    };

    let initial_missing = kind == FsEventKind::Remove;
    let kind = match kind {
      FsEventKind::Change | FsEventKind::Create if !exists => FsEventKind::Remove,
      FsEventKind::Remove if exists => FsEventKind::Change,
      kind => kind,
    };

    let accessor = self.path_manager.access();
    let is_watched_path = accessor.files().0.contains(path) || accessor.missing().0.contains(path);

    if initial_missing || kind == FsEventKind::Remove {
      if self.path_manager.is_registered(DepKind::Directory, path) {
        self.sweep_removed_context(None, path);
      } else {
        self.sweep_removed_path(path);
      }
      if kind == FsEventKind::Remove && is_watched_path {
        self.path_manager.set_reported_file_removal(path);
      }
    }

    if kind != FsEventKind::Remove && !self.path_manager.set_context_entry(path, true).await {
      return;
    }
    if !self.path_manager.is_any_registered_or_below_context(path) {
      return;
    }
    self.path_manager.set_last_watch_events(path);

    let finder = self.finder();
    let associated_event = if exists
      && finder.contains_path(path)
      && (self.path_manager.bound_dep(DepKind::File, path).is_some()
        || self
          .path_manager
          .bound_dep(DepKind::Missing, path)
          .is_some()
        || self
          .path_manager
          .bound_dep(DepKind::Directory, path)
          .is_some())
    {
      let mut v = vec![(path.clone(), kind)];
      finder.recurse_parent_directories(path, &mut v);
      v
    } else {
      finder.find_associated_event(path, kind)
    };

    let mut filtered_associated = Vec::with_capacity(associated_event.len());
    for (associated, associated_kind) in associated_event {
      if &associated != path {
        if self
          .path_manager
          .bound_dep(DepKind::Directory, &associated)
          .is_some_and(|b| !b.target_active)
        {
          continue;
        }
        let filter_root = self.path_manager.context_filter_root(&associated);
        let filter_entry = self.path_manager.filter_path_for_context(&associated, path);
        if self
          .path_manager
          .is_ignored_context_chain(filter_root.as_path(), filter_entry.as_path())
          .await
        {
          continue;
        }
      }
      filtered_associated.push((associated, associated_kind));
    }
    let mut associated_event = filtered_associated;

    for (associated, associated_kind) in &mut associated_event {
      if associated != path
        && *associated_kind == FsEventKind::Change
        && std::fs::symlink_metadata(&*associated).is_err()
      {
        *associated_kind = FsEventKind::Remove;
        self.analyzer.invalidate_path(associated);
        self.sweep_removed_path(associated);
      }
    }

    let context_removal: Vec<InternedPath> = (kind == FsEventKind::Remove
      && !initial_missing
      && associated_event
        .iter()
        .any(|(associated, k)| associated != path && *k == FsEventKind::Change))
    .then(|| path.clone())
    .into_iter()
    .collect();
    let context_only_removal = !context_removal.is_empty() && !finder.contains_file(path);
    if !context_removal.is_empty() {
      associated_event.retain_mut(|(associated, associated_kind)| {
        if *associated_kind == FsEventKind::Remove && !finder.contains_file(associated) {
          if associated == path {
            return false;
          }
          *associated_kind = FsEventKind::Change;
        }
        true
      });
    }
    let has_parent_context = associated_event
      .iter()
      .any(|(associated, _)| associated != path);
    let is_root_context_only_change = kind != FsEventKind::Remove
      && finder.directories.contains(path)
      && !finder.contains_file(path)
      && !has_parent_context;
    let raw: EventBatch =
      (!associated_event.is_empty() && !context_only_removal && !is_root_context_only_change)
        .then(|| FsEvent {
          path: path.clone(),
          kind,
        })
        .into_iter()
        .collect();

    self.analyzer.invalidate_path(path);
    for (associated, _) in &associated_event {
      self.analyzer.invalidate_path(associated);
    }
    if !associated_event.is_empty() {
      self.trigger_events(associated_event, raw, Vec::new(), context_removal);
    }
  }

  #[inline(never)]
  async fn process_live_event(
    &self,
    disk_path: &InternedPath,
    raw_kind: FsEventKind,
    flags: LiveEventFlags,
  ) {
    let is_disk = matches!(flags.source, LiveSource::Disk { .. });
    let appeared = flags.appeared;
    let mut dependencies: Vec<(InternedPath, FsEventKind)> = Vec::new();
    let mut raw_events: EventBatch = Vec::new();
    let mut recovered_events: EventBatch = Vec::new();
    let mut context_removals: Vec<InternedPath> = Vec::new();

    // 1. Check if a tracked physical directory was replaced in-place.
    let directory_replaced = self
      .path_manager
      .check_physical_directory_replaced(disk_path);
    if directory_replaced {
      self.rebind_disk_watcher_prefix(disk_path.as_path());
      self
        .reconcile_replaced_directory(
          disk_path,
          &mut dependencies,
          &mut raw_events,
          &mut recovered_events,
          &mut context_removals,
        )
        .await;
      self.sync_disk_watcher();
    }

    // 2. Check `pending_missing_heads` for newly appeared missing symlink/directory ancestors.
    let pending_recovered = self
      .path_manager
      .take_pending_missing_for_path(disk_path, appeared && disk_path.is_dir());
    if !pending_recovered.is_empty() {
      for (dep_kind, logical_path) in pending_recovered {
        if let Some(bound) = self
          .path_manager
          .rebind_single_dep(dep_kind, &logical_path)
          .await
        {
          self.analyzer.invalidate_path(&logical_path);
          if bound.target_active {
            match dep_kind {
              DepKind::File | DepKind::Missing => {
                if let Some(mtime) = disk_mtime(&bound.target) {
                  self
                    .path_manager
                    .seed_physical_baseline_if_absent(&bound.target);
                  self
                    .path_manager
                    .set_file_time(&logical_path, mtime, false, false);
                  emit_dep_and_raw(
                    &mut dependencies,
                    &mut raw_events,
                    logical_path,
                    FsEventKind::Create,
                  );
                }
              }
              DepKind::Directory => {
                if bound.target.is_dir() {
                  let _ = self
                    .path_manager
                    .scan_contexts_with_plan(
                      SystemTime::UNIX_EPOCH,
                      &[(logical_path.clone(), true)],
                      false,
                    )
                    .await;
                  self.path_manager.set_last_watch_events(&logical_path);
                  emit_dep_and_raw(
                    &mut dependencies,
                    &mut raw_events,
                    logical_path,
                    FsEventKind::Change,
                  );
                }
              }
            }
          }
        }
      }
      self.sync_disk_watcher();
    }

    // 3. Process `disk_path` first (awaiting JS `ignored` BEFORE stating `disk_path` so synchronous
    // rename pairs on Windows coalesce identically to Watchpack), then reconcile whole-directory
    // removals / absent context recoveries.
    let root_flags = LiveEventFlags {
      directory_replaced,
      ..flags
    };
    let root_meta_opt = self
      .process_single_physical_path(
        disk_path,
        raw_kind,
        root_flags,
        &mut dependencies,
        &mut raw_events,
        &mut context_removals,
      )
      .await;

    let phys_meta = root_meta_opt.unwrap_or_else(|| std::fs::symlink_metadata(disk_path).ok());
    let exists_on_disk = phys_meta.is_some();
    let is_real_dir = phys_meta.as_ref().is_some_and(|m| m.file_type().is_dir());
    let is_any_dir = is_real_dir
      || (phys_meta
        .as_ref()
        .is_some_and(|m| m.file_type().is_symlink())
        && disk_path.is_dir());
    let removed_explicit = raw_kind == FsEventKind::Remove && !is_disk;

    if exists_on_disk
      && !removed_explicit
      && is_any_dir
      && self
        .path_manager
        .recover_absent_contexts_under(disk_path)
        .await
    {
      self.sync_disk_watcher();
    }

    if removed_explicit || !exists_on_disk {
      if self.path_manager.may_contain_records(disk_path) {
        self.path_manager.remove_file_time(disk_path);
        self.path_manager.remove_file_times_under(disk_path);
      }
      let mut extra_physical_paths = Vec::new();
      for desc in self.path_manager.collect_physical_descendants(disk_path) {
        if desc != *disk_path && std::fs::symlink_metadata(&desc).is_err() {
          push_unique(&mut extra_physical_paths, desc);
        }
      }
      for phys in &extra_physical_paths {
        let desc_flags = LiveEventFlags {
          directory_replaced: false,
          ..flags
        };
        let _ = self
          .process_single_physical_path(
            phys,
            raw_kind,
            desc_flags,
            &mut dependencies,
            &mut raw_events,
            &mut context_removals,
          )
          .await;
      }
    }

    // 4. Appearing directory recovery: walk newly moved-in / created non-symlink directories
    // to recover registered files/missing dependencies and context aliases inside them.
    if appeared && is_real_dir {
      if !directory_replaced {
        self.rebind_disk_watcher_prefix(disk_path.as_path());
      }
      self
        .recover_appeared_directory(disk_path, &mut dependencies, &mut recovered_events)
        .await;
    }

    // Verify any parent context in `dependencies` still exists on disk.
    for (associated, associated_kind) in &mut dependencies {
      if associated != disk_path
        && *associated_kind == FsEventKind::Change
        && self
          .path_manager
          .is_registered(DepKind::Directory, associated)
      {
        let dir_exists = match self.path_manager.bound_dep(DepKind::Directory, associated) {
          Some(b) if b.target_active => {
            (exists_on_disk && b.links.is_empty() && disk_path.starts_with(b.target.as_path()))
              || (b.target.is_dir() && b.links.iter().all(|l| l.symlink_metadata().is_ok()))
          }
          None if exists_on_disk && disk_path.starts_with(associated.as_path()) => true,
          _ => std::fs::symlink_metadata(&*associated).is_ok(),
        };
        if !dir_exists {
          *associated_kind = FsEventKind::Remove;
          self.analyzer.invalidate_path(associated);
          self.sweep_removed_context(None, associated);
        }
      }
    }

    if !dependencies.iter().any(|(_, k)| *k == FsEventKind::Change) {
      context_removals.clear();
    } else if !context_removals.is_empty() {
      let finder = self.finder();
      dependencies.retain_mut(|(associated, associated_kind)| {
        if *associated_kind == FsEventKind::Remove && !finder.contains_file(associated) {
          if context_removals.contains(associated) {
            return false;
          }
          *associated_kind = FsEventKind::Change;
        }
        true
      });
      raw_events.retain(|ev| {
        !(ev.kind == FsEventKind::Remove
          && context_removals.contains(&ev.path)
          && !finder.contains_file(&ev.path))
      });
    }

    if !dependencies.is_empty() {
      self.trigger_events(dependencies, raw_events, recovered_events, context_removals);
    }
  }

  async fn process_single_physical_path(
    &self,
    phys: &InternedPath,
    raw_kind: FsEventKind,
    flags: LiveEventFlags,
    dependencies: &mut Vec<(InternedPath, FsEventKind)>,
    raw_events: &mut EventBatch,
    context_removals: &mut Vec<InternedPath>,
  ) -> Option<Option<std::fs::Metadata>> {
    let (is_disk, renamed, directory_probe) = match flags.source {
      LiveSource::Disk {
        renamed,
        directory_probe,
      } => (true, renamed, directory_probe),
      LiveSource::Injected => (false, false, false),
    };
    let directory_replaced = flags.directory_replaced;
    let entry_subs = self.path_manager.resolve_entry_subscribers(phys);
    let context_subs = self
      .path_manager
      .resolve_context_subscribers(phys, !is_disk);
    if entry_subs.is_empty() && context_subs.is_empty() {
      if !is_disk {
        let _ = self.path_manager.is_ignored_path(phys.as_path()).await;
      }
      return None;
    }

    let removed_explicit = raw_kind == FsEventKind::Remove && !is_disk;
    let mut phys_meta_opt: Option<Option<std::fs::Metadata>> = None;
    let mut judged_logical_entries = InternedPathSet::default();
    let mut unchanged_logical_entries = InternedPathSet::default();
    let mut phys_baseline_result: Option<Option<SystemTime>> = None;

    // A. Process EntrySubscribers (`Registered` and `ContextFile`)
    for sub in entry_subs {
      match sub {
        EntrySubscriber::Registered {
          kind: dep_kind,
          logical_path,
          is_link_hop,
        } => {
          if self
            .path_manager
            .is_ignored_registered_subscriber(phys, &logical_path, is_link_hop)
            .await
          {
            continue;
          }
          if !self.path_manager.is_registered(dep_kind, &logical_path) {
            continue;
          }

          if is_link_hop {
            if !self.path_manager.check_and_update_link_state(phys, is_disk) {
              continue;
            }
            self.path_manager.mark_dep_dirty(dep_kind, &logical_path);
            self.analyzer.invalidate_path(&logical_path);

            let link_still_there = LinkState::read(phys.as_path()).mtime.is_some();
            let logical_still_there = std::fs::symlink_metadata(&logical_path).is_ok();
            let ev_kind = match dep_kind {
              DepKind::File | DepKind::Directory => {
                if link_still_there && logical_still_there {
                  FsEventKind::Change
                } else {
                  FsEventKind::Remove
                }
              }
              DepKind::Missing => {
                if link_still_there && logical_still_there {
                  FsEventKind::Change
                } else if self.path_manager.has_file_time(&logical_path) {
                  FsEventKind::Remove
                } else {
                  continue;
                }
              }
            };

            if ev_kind == FsEventKind::Remove {
              self.sweep_removed_path(&logical_path);
              if matches!(dep_kind, DepKind::File | DepKind::Missing) {
                self.path_manager.set_reported_file_removal(&logical_path);
              }
            } else if matches!(dep_kind, DepKind::File | DepKind::Missing) {
              let mtime = disk_mtime(&logical_path)
                .or_else(|| {
                  std::fs::symlink_metadata(&logical_path)
                    .and_then(|m| m.modified().or_else(|_| m.created()))
                    .ok()
                })
                .unwrap_or_else(SystemTime::now);
              self
                .path_manager
                .set_file_time(&logical_path, mtime, false, false);
              judged_logical_entries.insert(logical_path.clone());
            } else {
              self.path_manager.set_last_watch_events(&logical_path);
            }

            emit_dep_and_raw(dependencies, raw_events, logical_path, ev_kind);
          } else {
            let phys_meta = phys_meta_opt
              .get_or_insert_with(|| std::fs::symlink_metadata(phys).ok())
              .as_ref();
            let phys_exists = phys_meta.is_some();
            let exists = match dep_kind {
              DepKind::Directory => phys.is_dir(),
              DepKind::File | DepKind::Missing => {
                phys_exists
                  && (logical_path == *phys || std::fs::symlink_metadata(&logical_path).is_ok())
              }
            };
            let ev_kind = match raw_kind {
              FsEventKind::Change | FsEventKind::Create if !exists => FsEventKind::Remove,
              FsEventKind::Remove if exists => FsEventKind::Change,
              k => k,
            };

            if dep_kind == DepKind::Missing
              && ev_kind == FsEventKind::Remove
              && (!self.path_manager.has_file_time(&logical_path)
                || self.path_manager.is_known_absent(&logical_path))
            {
              continue;
            }

            if dep_kind == DepKind::Directory {
              if ev_kind == FsEventKind::Remove {
                self.sweep_removed_context(Some(phys), &logical_path);
                emit_dep_and_raw(dependencies, raw_events, logical_path, FsEventKind::Remove);
              } else {
                if directory_probe && !directory_replaced {
                  continue;
                }
                let _ = self
                  .path_manager
                  .observe_live_context_entry(
                    phys,
                    phys_meta,
                    &logical_path,
                    &[],
                    LiveEntryFlags {
                      judged: false,
                      is_disk,
                      renamed,
                    },
                  )
                  .await;
                self.path_manager.set_last_watch_events(&logical_path);
                emit_dep_and_raw(dependencies, raw_events, logical_path, ev_kind);
              }
            } else {
              if removed_explicit || ev_kind == FsEventKind::Remove {
                self.path_manager.remove_file_time(&logical_path);
                if ev_kind == FsEventKind::Remove {
                  self.path_manager.set_reported_file_removal(&logical_path);
                  emit_dep_and_raw(dependencies, raw_events, logical_path, FsEventKind::Remove);
                  continue;
                }
              }

              let advanced = *phys_baseline_result
                .get_or_insert_with(|| self.path_manager.advance_physical_baseline(phys));
              if advanced.is_none()
                && !directory_replaced
                && (disk_mtime(phys).is_some()
                  || phys_meta.is_some_and(|m| m.file_type().is_symlink()))
              {
                unchanged_logical_entries.insert(logical_path);
                continue;
              }
              let logical_mtime_opt = if logical_path == *phys {
                advanced.or_else(|| disk_mtime(&logical_path))
              } else {
                disk_mtime(&logical_path).or(advanced)
              };
              if let Some(logical_mtime) = logical_mtime_opt {
                self
                  .path_manager
                  .set_file_time(&logical_path, logical_mtime, false, false);
              }
              judged_logical_entries.insert(logical_path.clone());
              emit_dep_and_raw(dependencies, raw_events, logical_path, ev_kind);
            }
          }
        }

        EntrySubscriber::ContextFile {
          root_context,
          logical_file,
        } => {
          if self.path_manager.is_ignored_path(phys.as_path()).await
            || self
              .path_manager
              .is_ignored_path(logical_file.as_path())
              .await
          {
            continue;
          }
          if !self
            .path_manager
            .is_registered(DepKind::Directory, &root_context)
            || std::fs::symlink_metadata(&logical_file).is_err()
          {
            continue;
          }
          let phys_exists = phys_meta_opt
            .get_or_insert_with(|| std::fs::symlink_metadata(phys).ok())
            .is_some();
          if !phys_exists {
            self.path_manager.clear_physical_baseline(phys);
            continue;
          }
          let advanced = *phys_baseline_result
            .get_or_insert_with(|| self.path_manager.advance_physical_baseline(phys));
          let Some(phys_mtime) = advanced else {
            continue;
          };
          self
            .path_manager
            .set_file_time(&logical_file, phys_mtime, false, false);
          judged_logical_entries.insert(logical_file.clone());
          self
            .path_manager
            .set_last_watch_events_for_context(&logical_file, &root_context);
          push_dep(dependencies, root_context, FsEventKind::Change);
          push_unique(
            raw_events,
            FsEvent {
              path: logical_file,
              kind: FsEventKind::Change,
            },
          );
        }
      }
    }

    // B. Process ContextSubscribers `(root_context, logical_entry, filter_entry)`
    let mut grouped_contexts: InternedPathMap<Vec<(InternedPath, InternedPath)>> =
      InternedPathMap::default();
    for (root_ctx, logical_entry, filter_entry) in context_subs {
      let filter_root = self.path_manager.context_filter_root(&root_ctx);
      if self
        .path_manager
        .is_ignored_context_chain(filter_root.as_path(), filter_entry.as_path())
        .await
      {
        continue;
      }
      if !self
        .path_manager
        .is_registered(DepKind::Directory, &root_ctx)
      {
        continue;
      }
      grouped_contexts
        .entry(logical_entry)
        .or_default()
        .push((root_ctx, filter_entry));
    }

    for (logical_entry, pairs) in grouped_contexts {
      let mut child_pairs = Vec::new();
      for (root_ctx, filter_entry) in pairs {
        if logical_entry == root_ctx {
          if !phys.is_dir() {
            if phys != &root_ctx && root_ctx.is_dir() {
              self.path_manager.clear_context_for_rebind(&root_ctx);
              if self
                .path_manager
                .rebind_single_dep(DepKind::Directory, &root_ctx)
                .await
                .is_some()
              {
                self.sync_disk_watcher();
                let out = self
                  .path_manager
                  .scan_contexts_with_plan(
                    SystemTime::UNIX_EPOCH,
                    &[(root_ctx.clone(), true)],
                    false,
                  )
                  .await;
                if out.added_watched_paths {
                  self.sync_disk_watcher();
                }
              }
              self.path_manager.set_last_watch_events(&root_ctx);
              push_dep(dependencies, root_ctx, FsEventKind::Change);
            } else {
              self.sweep_removed_context(Some(phys), &root_ctx);
              emit_dep_and_raw(dependencies, raw_events, root_ctx, FsEventKind::Remove);
            }
          } else if !directory_probe || directory_replaced {
            let phys_meta = phys_meta_opt
              .get_or_insert_with(|| std::fs::symlink_metadata(phys).ok())
              .as_ref();
            let (outcome, added_watched) = self
              .path_manager
              .observe_live_context_entry(
                phys,
                phys_meta,
                &root_ctx,
                &[],
                LiveEntryFlags {
                  judged: false,
                  is_disk,
                  renamed,
                },
              )
              .await;
            if added_watched {
              self.sync_disk_watcher();
            }
            if matches!(outcome, ContextEntryOutcome::Changed) || directory_replaced {
              self.path_manager.set_last_watch_events(&root_ctx);
              push_dep(dependencies, root_ctx.clone(), FsEventKind::Change);
            }
          }
        } else {
          child_pairs.push((root_ctx, filter_entry));
        }
      }

      if child_pairs.is_empty() {
        continue;
      }

      let phys_meta = phys_meta_opt
        .get_or_insert_with(|| std::fs::symlink_metadata(phys).ok())
        .as_ref();
      let phys_exists = phys_meta.is_some();
      let ev_kind = match raw_kind {
        FsEventKind::Change | FsEventKind::Create if !phys_exists => FsEventKind::Remove,
        FsEventKind::Remove if phys_exists => FsEventKind::Change,
        k => k,
      };

      let owners: Vec<InternedPath> = child_pairs.iter().map(|(r, _)| r.clone()).collect();
      if removed_explicit || ev_kind == FsEventKind::Remove {
        self.sweep_removed_path(&logical_entry);
        if ev_kind == FsEventKind::Remove {
          for (root_ctx, filter_entry) in &child_pairs {
            self
              .path_manager
              .set_last_watch_events_for_context(&logical_entry, root_ctx);
            push_dep(dependencies, root_ctx.clone(), FsEventKind::Change);
            push_unique(context_removals, filter_entry.clone());
          }
          continue;
        }
      }

      if unchanged_logical_entries.contains(&logical_entry) && !directory_replaced {
        continue;
      }
      let judged = judged_logical_entries.contains(&logical_entry);
      let (outcome, added_watched) = self
        .path_manager
        .observe_live_context_entry(
          phys,
          phys_meta,
          &logical_entry,
          &owners,
          LiveEntryFlags {
            judged,
            is_disk,
            renamed,
          },
        )
        .await;
      if added_watched {
        self.sync_disk_watcher();
      }
      match outcome {
        ContextEntryOutcome::Unchanged if !directory_replaced => continue,
        ContextEntryOutcome::ExistingDirectory if !directory_replaced => continue,
        _ => {}
      }
      if !added_watched
        && cfg!(any(
          feature = "testing",
          not(any(target_os = "macos", target_os = "windows"))
        ))
        && phys_meta
          .is_some_and(|m| m.file_type().is_dir() || (m.file_type().is_symlink() && phys.is_dir()))
      {
        self.sync_disk_watcher();
      }
      for (root_ctx, filter_entry) in child_pairs {
        self
          .path_manager
          .set_last_watch_events_for_context(&logical_entry, &root_ctx);
        push_dep(dependencies, root_ctx, FsEventKind::Change);
        push_unique(
          raw_events,
          FsEvent {
            path: filter_entry,
            kind: ev_kind,
          },
        );
      }
    }
    phys_meta_opt
  }

  async fn reconcile_replaced_directory(
    &self,
    phys_dir: &InternedPath,
    dependencies: &mut Vec<(InternedPath, FsEventKind)>,
    raw_events: &mut EventBatch,
    recovered_events: &mut EventBatch,
    context_removals: &mut Vec<InternedPath>,
  ) {
    let mut context_targets: Vec<(InternedPath, InternedPath)> = self
      .path_manager
      .resolve_context_subscribers(phys_dir, false)
      .into_iter()
      .map(|(r, l, _)| (r, l))
      .collect();
    for dir in self.path_manager.access().directories().0.iter() {
      if *dir == *phys_dir || dir.starts_with(phys_dir.as_path()) {
        push_unique(&mut context_targets, (dir.clone(), dir.clone()));
      }
    }
    for (root_ctx, logical_dir) in context_targets {
      let prev_files = self.collect_recorded_files_under(&logical_dir);
      if logical_dir == root_ctx {
        self.path_manager.mark_context_target_absent(&root_ctx);
      } else {
        self.path_manager.remove_file_times_under(&logical_dir);
      }
      let _ = self
        .path_manager
        .scan_contexts_with_plan(SystemTime::UNIX_EPOCH, &[(root_ctx.clone(), true)], false)
        .await;
      if self
        .path_manager
        .is_registered(DepKind::Directory, &root_ctx)
      {
        self
          .path_manager
          .set_last_watch_events_for_context(&logical_dir, &root_ctx);
        push_dep(dependencies, root_ctx.clone(), FsEventKind::Change);
      }
      for old_file in prev_files {
        if !old_file.exists() {
          if self.path_manager.is_registered(DepKind::File, &old_file) {
            self.path_manager.remove_file_time(&old_file);
            self.path_manager.set_reported_file_removal(&old_file);
            push_dep(dependencies, old_file.clone(), FsEventKind::Remove);
            push_unique(
              raw_events,
              FsEvent {
                path: old_file.clone(),
                kind: FsEventKind::Remove,
              },
            );
          }
          push_unique(context_removals, old_file);
        }
      }
    }

    let accessor = self.path_manager.access();
    let mut file_candidates: Vec<(InternedPath, DepKind, InternedPath)> = Vec::new();
    for candidate in self.path_manager.collect_physical_descendants(phys_dir) {
      for sub in self.path_manager.resolve_entry_subscribers(&candidate) {
        if let EntrySubscriber::Registered {
          kind: dep_kind @ (DepKind::File | DepKind::Missing),
          logical_path,
          is_link_hop: false,
        } = sub
        {
          push_unique(
            &mut file_candidates,
            (candidate.clone(), dep_kind, logical_path),
          );
        }
      }
    }
    let direct_files: Vec<InternedPath> = accessor
      .files()
      .0
      .iter()
      .filter(|f| f.starts_with(phys_dir.as_path()))
      .map(|f| f.clone())
      .collect();
    for f in direct_files {
      if self.path_manager.bound_dep(DepKind::File, &f).is_none() {
        push_unique(&mut file_candidates, (f.clone(), DepKind::File, f));
      }
    }
    let direct_missing: Vec<InternedPath> = accessor
      .missing()
      .0
      .iter()
      .filter(|m| m.starts_with(phys_dir.as_path()))
      .map(|m| m.clone())
      .collect();
    for m in direct_missing {
      if self.path_manager.bound_dep(DepKind::Missing, &m).is_none() {
        push_unique(&mut file_candidates, (m.clone(), DepKind::Missing, m));
      }
    }
    for (candidate, dep_kind, logical_path) in file_candidates {
      if self
        .path_manager
        .is_ignored_registered_subscriber(&candidate, &logical_path, false)
        .await
        || !self.path_manager.is_registered(dep_kind, &logical_path)
      {
        continue;
      }
      if std::fs::symlink_metadata(&candidate).is_ok() {
        if let Some(mtime) = disk_mtime(&candidate) {
          self.path_manager.clear_physical_baseline(&candidate);
          let _ = self
            .path_manager
            .has_physical_mtime_changed(&candidate, &logical_path);
          self
            .path_manager
            .set_file_time(&logical_path, mtime, false, false);
          let k = if dep_kind == DepKind::Missing {
            FsEventKind::Create
          } else {
            FsEventKind::Change
          };
          push_dep(dependencies, logical_path.clone(), k);
          recovered_events.push(FsEvent {
            path: logical_path,
            kind: k,
          });
        }
      } else if dep_kind == DepKind::File {
        self.path_manager.remove_file_time(&logical_path);
        self.path_manager.set_reported_file_removal(&logical_path);
        push_dep(dependencies, logical_path.clone(), FsEventKind::Remove);
        push_unique(
          raw_events,
          FsEvent {
            path: logical_path,
            kind: FsEventKind::Remove,
          },
        );
      }
    }
  }

  fn collect_recorded_files_under(&self, logical_dir: &InternedPath) -> Vec<InternedPath> {
    let accessor = self.path_manager.access();
    accessor
      .files()
      .0
      .iter()
      .filter(|p| p.starts_with(logical_dir.as_path()))
      .map(|p| p.clone())
      .collect()
  }

  async fn recover_appeared_directory(
    &self,
    root_dir: &InternedPath,
    dependencies: &mut Vec<(InternedPath, FsEventKind)>,
    recovered: &mut EventBatch,
  ) {
    let mut directories = vec![root_dir.clone()];
    while let Some(directory) = directories.pop() {
      let Ok(entries) = std::fs::read_dir(&directory) else {
        continue;
      };
      for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else {
          continue;
        };
        let entry_path = InternedPath::from(entry.path());
        let subs = self.path_manager.resolve_entry_subscribers(&entry_path);
        let context_subs = self
          .path_manager
          .resolve_context_subscribers(&entry_path, false);
        if !file_type.is_dir() && subs.is_empty() && context_subs.is_empty() {
          continue;
        }
        if file_type.is_dir()
          && subs.is_empty()
          && context_subs.is_empty()
          && self
            .path_manager
            .is_ignored_path(entry_path.as_path())
            .await
        {
          continue;
        }

        let mut judged_logical = InternedPathSet::default();
        for sub in subs {
          if let EntrySubscriber::Registered {
            kind: dep_kind,
            logical_path,
            is_link_hop,
          } = sub
          {
            if self
              .path_manager
              .is_ignored_registered_subscriber(&entry_path, &logical_path, is_link_hop)
              .await
              || !self.path_manager.is_registered(dep_kind, &logical_path)
            {
              continue;
            }
            if is_link_hop {
              if !self
                .path_manager
                .check_and_update_link_state(&entry_path, true)
              {
                continue;
              }
              self.path_manager.mark_dep_dirty(dep_kind, &logical_path);
              if let Some(bound) = self
                .path_manager
                .rebind_single_dep(dep_kind, &logical_path)
                .await
              {
                self.sync_disk_watcher();
                if bound.target_active && std::fs::symlink_metadata(&bound.target).is_ok() {
                  if let Some(mtime) = disk_mtime(&bound.target) {
                    self
                      .path_manager
                      .set_file_time(&logical_path, mtime, false, false);
                  }
                  judged_logical.insert(logical_path.clone());
                  recovered.push(FsEvent {
                    path: logical_path.clone(),
                    kind: FsEventKind::Create,
                  });
                  push_dep(dependencies, logical_path, FsEventKind::Create);
                }
              }
            } else {
              if dep_kind == DepKind::File
                && !self
                  .path_manager
                  .has_physical_mtime_changed(&entry_path, &logical_path)
              {
                continue;
              }
              if matches!(dep_kind, DepKind::File | DepKind::Missing)
                && let Some(mtime) = disk_mtime(&entry_path)
              {
                self
                  .path_manager
                  .set_file_time(&logical_path, mtime, false, false);
              }
              judged_logical.insert(logical_path.clone());
              recovered.push(FsEvent {
                path: logical_path.clone(),
                kind: FsEventKind::Create,
              });
              push_dep(dependencies, logical_path.clone(), FsEventKind::Create);
              for (root_ctx, _, filter_entry) in &context_subs {
                let filter_root = self.path_manager.context_filter_root(root_ctx);
                if !self
                  .path_manager
                  .is_ignored_context_chain(filter_root.as_path(), filter_entry.as_path())
                  .await
                  && self
                    .path_manager
                    .is_registered(DepKind::Directory, root_ctx)
                {
                  push_dep(dependencies, root_ctx.clone(), FsEventKind::Change);
                }
              }
            }
          }
        }

        if !context_subs.is_empty() {
          let mut grouped: InternedPathMap<Vec<(InternedPath, InternedPath)>> =
            InternedPathMap::default();
          for (root_ctx, logical_entry, filter_entry) in context_subs {
            let filter_root = self.path_manager.context_filter_root(&root_ctx);
            if self
              .path_manager
              .is_ignored_context_chain(filter_root.as_path(), filter_entry.as_path())
              .await
              || !self
                .path_manager
                .is_registered(DepKind::Directory, &root_ctx)
            {
              continue;
            }
            grouped
              .entry(logical_entry)
              .or_default()
              .push((root_ctx, filter_entry));
          }
          for (logical_entry, pairs) in grouped {
            if judged_logical.contains(&logical_entry) {
              continue;
            }
            let mut child_pairs = Vec::new();
            for (root_ctx, filter_entry) in pairs {
              if logical_entry == root_ctx {
                let (_, added_watched) = self
                  .path_manager
                  .observe_live_context_entry(
                    &entry_path,
                    None,
                    &root_ctx,
                    &[],
                    LiveEntryFlags {
                      judged: false,
                      is_disk: true,
                      renamed: false,
                    },
                  )
                  .await;
                if added_watched || file_type.is_dir() {
                  self.sync_disk_watcher();
                }
                self.path_manager.set_last_watch_events(&root_ctx);
                push_dep(dependencies, root_ctx, FsEventKind::Change);
              } else {
                child_pairs.push((root_ctx, filter_entry));
              }
            }
            if !child_pairs.is_empty() {
              let owners: Vec<InternedPath> = child_pairs.iter().map(|(r, _)| r.clone()).collect();
              let (outcome, added_watched) = self
                .path_manager
                .observe_live_context_entry(
                  &entry_path,
                  None,
                  &logical_entry,
                  &owners,
                  LiveEntryFlags {
                    judged: false,
                    is_disk: true,
                    renamed: false,
                  },
                )
                .await;
              if added_watched || file_type.is_dir() {
                self.sync_disk_watcher();
              }
              if matches!(outcome, ContextEntryOutcome::Changed) {
                for (root_ctx, filter_entry) in child_pairs {
                  self
                    .path_manager
                    .set_last_watch_events_for_context(&logical_entry, &root_ctx);
                  push_dep(dependencies, root_ctx, FsEventKind::Change);
                  if !file_type.is_dir() {
                    push_unique(
                      recovered,
                      FsEvent {
                        path: filter_entry,
                        kind: FsEventKind::Create,
                      },
                    );
                  }
                }
              }
            }
          }
        }

        if file_type.is_dir() {
          directories.push(entry_path);
        }
      }
    }
  }

  fn finder(&self) -> DependencyFinder<'_> {
    let accessor = self.path_manager.access();
    DependencyFinder {
      files: accessor.files().0,
      directories: accessor.directories().0,
      missing: accessor.missing().0,
    }
  }

  fn trigger_events(
    &self,
    events: Vec<(InternedPath, FsEventKind)>,
    raw: EventBatch,
    recovered: EventBatch,
    context_removal: Vec<InternedPath>,
  ) -> bool {
    self
      .tx
      .send(ResolvedEvents {
        dependencies: events
          .into_iter()
          .map(|(path, kind)| FsEvent { path, kind })
          .collect(),
        raw,
        recovered,
        context_removal,
      })
      .is_ok()
  }
}

fn emit_dep_and_raw(
  deps: &mut Vec<(InternedPath, FsEventKind)>,
  raw_events: &mut EventBatch,
  path: InternedPath,
  kind: FsEventKind,
) {
  push_dep(deps, path.clone(), kind);
  push_unique(raw_events, FsEvent { path, kind });
}

fn push_dep(deps: &mut Vec<(InternedPath, FsEventKind)>, path: InternedPath, kind: FsEventKind) {
  if let Some((_, existing_kind)) = deps.iter_mut().find(|(p, _)| *p == path) {
    *existing_kind = kind;
  } else {
    deps.push((path, kind));
  }
}

#[cfg(test)]
mod tests {

  use std::path::Path;

  use rspack_paths::InternedPath;

  use super::*;

  #[test]
  fn test_find_dependency_for_file() {
    let files = InternedPathDashSet::default();
    let directories = InternedPathDashSet::default();
    let missing = InternedPathDashSet::default();

    let file_0 = InternedPath::from(Path::new("/path/a/b/c/index.js"));
    let dir_0 = InternedPath::from(Path::new("/path/a/b"));
    files.insert(file_0.clone());

    directories.insert(dir_0.clone());
    let finder = DependencyFinder {
      files: &files,
      directories: &directories,
      missing: &missing,
    };

    let associated_events = finder.find_associated_event(&file_0, FsEventKind::Remove);

    assert_eq!(associated_events.len(), 2);

    assert!(associated_events.contains(&(file_0, FsEventKind::Remove)));
    assert!(associated_events.contains(&(dir_0, FsEventKind::Change)));
  }

  #[test]
  fn test_find_dependency_for_directory() {
    let files = InternedPathDashSet::default();
    let directories = InternedPathDashSet::default();
    let missing = InternedPathDashSet::default();

    let dir_0 = InternedPath::from(Path::new("/path/a/b/c"));
    let dir_1 = InternedPath::from(Path::new("/path/a/b"));

    directories.insert(dir_0.clone());
    directories.insert(dir_1.clone());

    let finder = DependencyFinder {
      files: &files,
      directories: &directories,
      missing: &missing,
    };

    let associated_events = finder.find_associated_event(
      &InternedPath::from(Path::new("/path/a/b/c/index.js")),
      FsEventKind::Create,
    );
    assert_eq!(associated_events.len(), 2);
    assert!(associated_events.contains(&(dir_0, FsEventKind::Change)));
    assert!(associated_events.contains(&(dir_1, FsEventKind::Change)));
  }
}
