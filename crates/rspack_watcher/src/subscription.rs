use std::{
  ffi::{OsStr, OsString},
  path::{Component, Path, PathBuf},
  time::SystemTime,
};

use rspack_paths::{InternedPath, InternedPathMap, InternedPathSet};
use rspack_util::fx_hash::FxHashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SubId(u32);

#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LogicalKind {
  File = 0,
  Directory = 1,
  Missing = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReportedBaseline {
  Unreported,
  InitialMissingAbsent,
  ReportedRemoved,
  Present(SystemTime),
}

impl ReportedBaseline {
  pub fn is_absent(self) -> bool {
    matches!(self, Self::InitialMissingAbsent | Self::ReportedRemoved)
  }
}

#[cfg(feature = "testing")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ObservedEntry {
  File { mtime: SystemTime },
  Directory,
}

pub(crate) enum SubscriptionRole {
  TargetEntry {
    dir: InternedPath,
    name: OsString,
    notify_path: InternedPath,
  },
  WaitingPrefix {
    dir: InternedPath,
    wait_name: OsString,
    remainder: PathBuf,
    notify_path: InternedPath,
  },
  ContextDir {
    dir: InternedPath,
    logical_dir: InternedPath,
    is_root: bool,
  },
}

pub(crate) struct Subscription {
  pub owner_kind: LogicalKind,
  pub owner_path: InternedPath,
  pub epoch: u64,
  pub role: SubscriptionRole,
}

pub(crate) struct LogicalRegistration {
  pub target_path: InternedPath,
  pub epoch: u64,
  pub sub_ids: Vec<SubId>,
}

#[derive(Default)]
pub(crate) struct ChildSlot {
  #[cfg(feature = "testing")]
  pub observed: Option<ObservedEntry>,
  pub entry_subs: Vec<SubId>,
  pub waiting_subs: Vec<SubId>,
  pub has_child_node: bool,
}

impl ChildSlot {
  pub fn is_unreferenced(&self) -> bool {
    self.entry_subs.is_empty() && self.waiting_subs.is_empty() && !self.has_child_node
  }

  fn has_observed(&self) -> bool {
    #[cfg(feature = "testing")]
    {
      self.observed.is_some()
    }
    #[cfg(not(feature = "testing"))]
    {
      false
    }
  }
}

#[derive(Default)]
pub(crate) struct DirectoryNode {
  pub children: FxHashMap<OsString, ChildSlot>,
  #[cfg(feature = "testing")]
  pub has_complete_listing: bool,
  pub context_subs: Vec<SubId>,
}

impl DirectoryNode {
  pub fn is_empty(&self) -> bool {
    self.context_subs.is_empty() && self.children.is_empty()
  }
}

/// Matched subscription candidate for an incoming event on `disk_path`.
pub(crate) struct MatchedTarget {
  pub owner_path: InternedPath,
  pub target_path: InternedPath,
  pub notify_path: InternedPath,
}

/// Matched context binding covering an entry or directory.
pub(crate) struct MatchedContext {
  pub epoch: u64,
  pub root_context: InternedPath,
  pub root_target: InternedPath,
  pub logical_entry_path: InternedPath,
  pub is_root_self: bool,
}

/// Waiting subscription that may advance now that `disk_path` exists.
pub(crate) struct PendingAdvance {
  pub sub_id: SubId,
  pub owner_kind: LogicalKind,
  pub owner_path: InternedPath,
  pub epoch: u64,
  pub current_dir: InternedPath,
  pub wait_name: OsString,
  pub remainder: PathBuf,
  pub notify_path: InternedPath,
}

/// Planned disk operations for one active scan pass (`scan_once`).
#[cfg(feature = "testing")]
pub(crate) struct ActiveScanPlan {
  pub context_dirs: Vec<ActiveContextScan>,
  pub stat_entries: Vec<ActiveEntryScan>,
  pub waiting_entries: Vec<PendingAdvance>,
}

#[cfg(feature = "testing")]
pub(crate) struct ActiveContextScan {
  pub dir: InternedPath,
  pub bindings: Vec<(SubId, u64, InternedPath, InternedPath, bool)>,
  pub prev_entries: FxHashMap<OsString, ObservedEntry>,
  pub had_complete_listing: bool,
}

#[cfg(feature = "testing")]
pub(crate) struct ActiveEntryScan {
  pub dir: InternedPath,
  pub name: OsString,
  pub full_path: InternedPath,
  pub targets: Vec<(SubId, u64, LogicalKind, InternedPath, InternedPath)>,
}

#[derive(Default)]
pub(crate) struct SubscriptionGraph {
  pub directories: InternedPathMap<DirectoryNode>,
  subscriptions: Vec<Option<Subscription>>,
  free_sub_ids: Vec<u32>,
  pub registrations: [InternedPathMap<LogicalRegistration>; 3],
  pub baselines: InternedPathMap<ReportedBaseline>,
  pub watch_refs: [InternedPathMap<u32>; 3],
  #[cfg(feature = "testing")]
  pub custom_targets: InternedPathMap<InternedPath>,
}

impl SubscriptionGraph {
  #[cfg(feature = "testing")]
  pub fn set_custom_target(&mut self, logical_path: InternedPath, target_path: InternedPath) {
    self.custom_targets.insert(logical_path, target_path);
  }

  pub fn resolve_target(&self, logical_path: &InternedPath) -> InternedPath {
    #[cfg(feature = "testing")]
    if let Some(target) = self.custom_targets.get(logical_path) {
      return target.clone();
    }
    logical_path.clone()
  }

  pub fn target_of(&self, kind: LogicalKind, logical_path: &InternedPath) -> Option<InternedPath> {
    self.registrations[kind as usize]
      .get(logical_path)
      .map(|reg| reg.target_path.clone())
  }

  pub fn retain_watch_ref(&mut self, kind: LogicalKind, path: InternedPath) -> bool {
    let count = self.watch_refs[kind as usize].entry(path).or_default();
    *count += 1;
    *count == 1
  }

  pub fn release_watch_ref(&mut self, kind: LogicalKind, path: &InternedPath) -> bool {
    let map = &mut self.watch_refs[kind as usize];
    let Some(count) = map.get_mut(path) else {
      return false;
    };
    *count -= 1;
    if *count == 0 {
      map.remove(path);
      return true;
    }
    false
  }

  #[inline(never)]
  pub fn register(&mut self, kind: LogicalKind, logical_path: InternedPath) -> InternedPath {
    let target_path = self.resolve_target(&logical_path);
    let next_epoch = self.registrations[kind as usize]
      .get(&logical_path)
      .map_or(1, |prev| prev.epoch.wrapping_add(1));

    self.detach_registration_subs(kind, &logical_path);

    let sub_ids = self.attach_initial_subs(kind, &logical_path, next_epoch, &target_path);
    self.registrations[kind as usize].insert(
      logical_path.clone(),
      LogicalRegistration {
        target_path: target_path.clone(),
        epoch: next_epoch,
        sub_ids,
      },
    );
    if matches!(kind, LogicalKind::File | LogicalKind::Missing) {
      self
        .baselines
        .entry(logical_path)
        .or_insert(ReportedBaseline::Unreported);
    }
    target_path
  }

  #[inline(never)]
  pub fn unregister(
    &mut self,
    kind: LogicalKind,
    logical_path: &InternedPath,
  ) -> Option<InternedPath> {
    self.detach_registration_subs(kind, logical_path);
    let removed = self.registrations[kind as usize]
      .remove(logical_path)
      .map(|reg| reg.target_path);

    let still_file_or_missing = self.registrations[LogicalKind::File as usize]
      .contains_key(logical_path)
      || self.registrations[LogicalKind::Missing as usize].contains_key(logical_path);
    if !still_file_or_missing {
      self.baselines.remove(logical_path);
      #[cfg(feature = "testing")]
      if !self.registrations[LogicalKind::Directory as usize].contains_key(logical_path) {
        self.custom_targets.remove(logical_path);
      }
    }
    removed
  }

  pub fn attach_child_context(
    &mut self,
    root_context: &InternedPath,
    expected_epoch: u64,
    child_dir: InternedPath,
    logical_subdir: InternedPath,
  ) -> bool {
    let Some(reg) = self.registrations[LogicalKind::Directory as usize].get(root_context) else {
      return false;
    };
    if reg.epoch != expected_epoch {
      return false;
    }
    if let Some(node) = self.directories.get(&child_dir) {
      let already_bound = node.context_subs.iter().any(|&sub_id| {
        self.get_sub(sub_id).is_some_and(|sub| {
          sub.owner_kind == LogicalKind::Directory
            && &sub.owner_path == root_context
            && matches!(
              &sub.role,
              SubscriptionRole::ContextDir { logical_dir, .. } if logical_dir == &logical_subdir
            )
        })
      });
      if already_bound {
        return true;
      }
    }
    let sub_id = self.alloc_sub(Subscription {
      owner_kind: LogicalKind::Directory,
      owner_path: root_context.clone(),
      epoch: expected_epoch,
      role: SubscriptionRole::ContextDir {
        dir: child_dir.clone(),
        logical_dir: logical_subdir,
        is_root: false,
      },
    });
    self.ensure_dir_node(&child_dir).context_subs.push(sub_id);
    if let Some(reg_mut) = self.registrations[LogicalKind::Directory as usize].get_mut(root_context)
    {
      reg_mut.sub_ids.push(sub_id);
    }
    true
  }

  pub fn is_registration_current(
    &self,
    kind: LogicalKind,
    path: &InternedPath,
    epoch: u64,
  ) -> bool {
    self.registrations[kind as usize]
      .get(path)
      .is_some_and(|reg| reg.epoch == epoch)
  }

  pub fn registration_epoch(&self, kind: LogicalKind, path: &InternedPath) -> Option<u64> {
    self.registrations[kind as usize]
      .get(path)
      .map(|reg| reg.epoch)
  }

  #[cfg(feature = "testing")]
  pub fn record_observed_entry(&mut self, path: &InternedPath, observed: ObservedEntry) {
    let (Some(parent), Some(name)) = (path.parent(), path.file_name()) else {
      return;
    };
    let parent_ip = InternedPath::from(parent);
    if let Some(node) = self.directories.get_mut(&parent_ip) {
      if !node.context_subs.is_empty() {
        node
          .children
          .entry(name.to_os_string())
          .or_default()
          .observed = Some(observed);
      } else if let Some(slot) = node.children.get_mut(name) {
        slot.observed = Some(observed);
      }
    }
  }

  #[cfg(feature = "testing")]
  pub fn remove_observed_entry(&mut self, path: &InternedPath) {
    let (Some(parent), Some(name)) = (path.parent(), path.file_name()) else {
      return;
    };
    let parent_ip = InternedPath::from(parent);
    if let Some(node) = self.directories.get_mut(&parent_ip) {
      let remove_slot = if let Some(slot) = node.children.get_mut(name) {
        slot.observed = None;
        node.context_subs.is_empty() && slot.is_unreferenced()
      } else {
        false
      };
      if remove_slot {
        node.children.remove(name);
      }
    }
  }

  #[cfg(feature = "testing")]
  pub fn set_dir_snapshot(
    &mut self,
    dir: &InternedPath,
    snapshot: FxHashMap<OsString, ObservedEntry>,
    complete: bool,
  ) {
    let Some(node) = self.directories.get_mut(dir) else {
      return;
    };
    if node.context_subs.is_empty() {
      return;
    }
    for slot in node.children.values_mut() {
      slot.observed = None;
    }
    for (name, obs) in snapshot {
      node.children.entry(name).or_default().observed = Some(obs);
    }
    node
      .children
      .retain(|_, slot| slot.observed.is_some() || !slot.is_unreferenced());
    node.has_complete_listing = complete;
  }

  /// Resolves incoming OS / injected / backfill event on `disk_path` against active subscriptions.
  /// Supports both exact entry events (`disk_path = parent/name`) and parent-directory-only events
  /// (e.g. macOS FSEvents coalesced directory clues where `disk_path` is a watched directory).
  #[inline(never)]
  pub fn lookup_disk_event(
    &self,
    disk_path: &InternedPath,
  ) -> (Vec<MatchedTarget>, Vec<MatchedContext>, Vec<PendingAdvance>) {
    let mut targets = Vec::new();
    let mut contexts = Vec::new();
    let mut waiting = Vec::new();

    if let (Some(parent), Some(name)) = (disk_path.parent(), disk_path.file_name()) {
      let parent_ip = InternedPath::from(parent);
      if let Some(parent_node) = self.directories.get(&parent_ip)
        && let Some(slot) = parent_node.children.get(name)
      {
        for &sub_id in &slot.entry_subs {
          if let Some(sub) = self.get_sub(sub_id)
            && let SubscriptionRole::TargetEntry { notify_path, .. } = &sub.role
          {
            match sub.owner_kind {
              LogicalKind::File | LogicalKind::Missing => {
                if let Some(reg) = self.registrations[sub.owner_kind as usize].get(&sub.owner_path)
                {
                  targets.push(MatchedTarget {
                    owner_path: sub.owner_path.clone(),
                    target_path: reg.target_path.clone(),
                    notify_path: notify_path.clone(),
                  });
                }
              }
              LogicalKind::Directory => {
                if let Some(reg) =
                  self.registrations[LogicalKind::Directory as usize].get(&sub.owner_path)
                {
                  contexts.push(MatchedContext {
                    epoch: sub.epoch,
                    root_context: sub.owner_path.clone(),
                    root_target: reg.target_path.clone(),
                    logical_entry_path: notify_path.clone(),
                    is_root_self: true,
                  });
                }
              }
            }
          }
        }

        for &sub_id in &slot.waiting_subs {
          if let Some(sub) = self.get_sub(sub_id)
            && let SubscriptionRole::WaitingPrefix {
              dir,
              wait_name,
              remainder,
              notify_path,
            } = &sub.role
          {
            waiting.push(PendingAdvance {
              sub_id,
              owner_kind: sub.owner_kind,
              owner_path: sub.owner_path.clone(),
              epoch: sub.epoch,
              current_dir: dir.clone(),
              wait_name: wait_name.clone(),
              remainder: remainder.clone(),
              notify_path: notify_path.clone(),
            });
          }
        }
      }
    }

    // When `disk_path` itself has a `DirectoryNode` (e.g. directory-only FSEvents clue or
    // direct event on a context root), include its direct `waiting_subs` and `context_subs`.
    if let Some(dir_node) = self.directories.get(disk_path) {
      for slot in dir_node.children.values() {
        for &sub_id in &slot.waiting_subs {
          if waiting.iter().any(|w| w.sub_id == sub_id) {
            continue;
          }
          if let Some(sub) = self.get_sub(sub_id)
            && let SubscriptionRole::WaitingPrefix {
              dir,
              wait_name,
              remainder,
              notify_path,
            } = &sub.role
          {
            waiting.push(PendingAdvance {
              sub_id,
              owner_kind: sub.owner_kind,
              owner_path: sub.owner_path.clone(),
              epoch: sub.epoch,
              current_dir: dir.clone(),
              wait_name: wait_name.clone(),
              remainder: remainder.clone(),
              notify_path: notify_path.clone(),
            });
          }
        }
      }

      for &sub_id in &dir_node.context_subs {
        if let Some(sub) = self.get_sub(sub_id)
          && let SubscriptionRole::ContextDir {
            logical_dir,
            is_root,
            ..
          } = &sub.role
          && *is_root
          && !contexts
            .iter()
            .any(|c| c.is_root_self && c.root_context == sub.owner_path)
          && let Some(reg) =
            self.registrations[LogicalKind::Directory as usize].get(&sub.owner_path)
        {
          contexts.push(MatchedContext {
            epoch: sub.epoch,
            root_context: sub.owner_path.clone(),
            root_target: reg.target_path.clone(),
            logical_entry_path: logical_dir.clone(),
            is_root_self: true,
          });
        }
      }
    }

    // Also match waiting prefixes when an event arrives directly for a deeper path inside a
    // newly created directory tree before the intermediate directory's own event was seen.
    for ancestor in disk_path.ancestors().skip(1) {
      let (Some(ancestor_parent), Some(ancestor_name)) = (ancestor.parent(), ancestor.file_name())
      else {
        continue;
      };
      let ancestor_parent_path = InternedPath::from(ancestor_parent);
      if let Some(node) = self.directories.get(&ancestor_parent_path)
        && let Some(slot) = node.children.get(ancestor_name)
      {
        for &sub_id in &slot.waiting_subs {
          if waiting.iter().any(|w| w.sub_id == sub_id) {
            continue;
          }
          if let Some(sub) = self.get_sub(sub_id)
            && let SubscriptionRole::WaitingPrefix {
              dir,
              wait_name,
              remainder,
              notify_path,
            } = &sub.role
          {
            waiting.push(PendingAdvance {
              sub_id,
              owner_kind: sub.owner_kind,
              owner_path: sub.owner_path.clone(),
              epoch: sub.epoch,
              current_dir: dir.clone(),
              wait_name: wait_name.clone(),
              remainder: remainder.clone(),
              notify_path: notify_path.clone(),
            });
          }
        }
      }
    }

    // Match context subscriptions on `disk_path.parent()` or nearest covering context ancestor.
    if let Some(parent) = disk_path.parent() {
      for ancestor in parent.ancestors() {
        let ancestor_path = InternedPath::from(ancestor);
        let Some(node) = self.directories.get(&ancestor_path) else {
          continue;
        };
        let Ok(rel_to_ancestor) = disk_path.as_path().strip_prefix(ancestor) else {
          continue;
        };
        for &sub_id in &node.context_subs {
          if let Some(sub) = self.get_sub(sub_id)
            && let SubscriptionRole::ContextDir { logical_dir, .. } = &sub.role
            && let Some(reg) =
              self.registrations[LogicalKind::Directory as usize].get(&sub.owner_path)
          {
            let logical_entry = InternedPath::from(logical_dir.as_path().join(rel_to_ancestor));
            let already_seen = contexts
              .iter()
              .any(|c| c.root_context == sub.owner_path && c.logical_entry_path == logical_entry);
            if !already_seen {
              contexts.push(MatchedContext {
                epoch: sub.epoch,
                root_context: sub.owner_path.clone(),
                root_target: reg.target_path.clone(),
                logical_entry_path: logical_entry,
                is_root_self: false,
              });
            }
          }
        }
      }
    }

    (targets, contexts, waiting)
  }

  /// When `removed_dir` is removed on disk, collects affected logical paths below it and demotes
  /// subscriptions inside `removed_dir`'s subtree back to `WaitingPrefix` on the nearest
  /// surviving ancestor directory so later directory recreation recovers automatically.
  #[inline(never)]
  pub fn handle_directory_removed(
    &mut self,
    removed_dir: &InternedPath,
  ) -> (InternedPathSet, InternedPathSet) {
    let mut affected_files = InternedPathSet::default();
    let mut affected_contexts = InternedPathSet::default();

    if !self.directories.contains_key(removed_dir) {
      return (affected_files, affected_contexts);
    }

    let mut subtree_dirs = Vec::new();
    let mut stack = vec![removed_dir.clone()];
    while let Some(dir) = stack.pop() {
      if let Some(node) = self.directories.get(&dir) {
        for (child_name, slot) in &node.children {
          if slot.has_child_node {
            stack.push(InternedPath::from(dir.as_path().join(child_name)));
          }
        }
      }
      subtree_dirs.push(dir);
    }

    let mut subs_to_demote: Vec<SubId> = Vec::new();
    let mut child_ctx_subs_to_drop: Vec<(InternedPath, SubId)> = Vec::new();
    for dir in &subtree_dirs {
      if let Some(node) = self.directories.get_mut(dir) {
        #[cfg(feature = "testing")]
        {
          node.has_complete_listing = false;
        }
        for slot in node.children.values_mut() {
          #[cfg(feature = "testing")]
          {
            slot.observed = None;
          }
          subs_to_demote.extend(slot.entry_subs.iter().copied());
          subs_to_demote.extend(slot.waiting_subs.iter().copied());
        }
        node.children.retain(|_, slot| !slot.is_unreferenced());
        for &sub_id in &node.context_subs {
          if let Some(sub) = self
            .subscriptions
            .get(sub_id.0 as usize)
            .and_then(Option::as_ref)
            && let SubscriptionRole::ContextDir { is_root, .. } = &sub.role
          {
            if *is_root {
              affected_contexts.insert(sub.owner_path.clone());
              subs_to_demote.push(sub_id);
            } else {
              child_ctx_subs_to_drop.push((sub.owner_path.clone(), sub_id));
            }
          }
        }
      }
    }

    for (owner_path, sub_id) in child_ctx_subs_to_drop {
      self.remove_sub(sub_id);
      if let Some(reg) = self.registrations[LogicalKind::Directory as usize].get_mut(&owner_path) {
        reg.sub_ids.retain(|&id| id != sub_id);
      }
    }

    let mut registrations_to_rebind: [InternedPathSet; 3] = Default::default();
    for sub_id in subs_to_demote {
      if let Some(sub) = self.get_sub(sub_id) {
        match sub.owner_kind {
          LogicalKind::File | LogicalKind::Missing => {
            affected_files.insert(sub.owner_path.clone());
          }
          LogicalKind::Directory => {
            affected_contexts.insert(sub.owner_path.clone());
          }
        }
        registrations_to_rebind[sub.owner_kind as usize].insert(sub.owner_path.clone());
      }
    }

    for kind in [
      LogicalKind::File,
      LogicalKind::Directory,
      LogicalKind::Missing,
    ] {
      for path in &registrations_to_rebind[kind as usize] {
        let Some((target_path, epoch)) = self.registrations[kind as usize]
          .get(path)
          .map(|r| (r.target_path.clone(), r.epoch))
        else {
          continue;
        };
        self.detach_registration_subs(kind, path);
        let new_subs = self.attach_initial_subs(kind, path, epoch, &target_path);
        if let Some(reg) = self.registrations[kind as usize].get_mut(path) {
          reg.sub_ids = new_subs;
        }
      }
    }

    (affected_files, affected_contexts)
  }

  /// Replaces a `WaitingPrefix` subscription either with a deeper `WaitingPrefix` (when
  /// intermediate directories appeared) or with final `TargetEntry` / `ContextDir` subscriptions
  /// (when the full suffix has appeared). Returns `true` if the subscription was still valid.
  #[inline(never)]
  pub fn commit_waiting_advance(
    &mut self,
    sub_id: SubId,
    expected_epoch: u64,
    new_dir: InternedPath,
    next_step: WaitingStepResult,
  ) -> bool {
    let Some(sub) = self.get_sub(sub_id) else {
      return false;
    };
    if sub.epoch != expected_epoch {
      return false;
    }
    let owner_kind = sub.owner_kind;
    let owner_path = sub.owner_path.clone();
    let notify_path = match &sub.role {
      SubscriptionRole::WaitingPrefix { notify_path, .. } => notify_path.clone(),
      _ => return false,
    };

    self.remove_sub(sub_id);

    let new_ids = match next_step {
      WaitingStepResult::StillWaiting {
        wait_name,
        remainder,
      } => vec![self.push_waiting_sub(
        owner_kind,
        &owner_path,
        expected_epoch,
        (new_dir, wait_name, remainder),
        notify_path,
      )],
      WaitingStepResult::ReachedEntry { entry_name } => match owner_kind {
        LogicalKind::File | LogicalKind::Missing => vec![self.push_target_entry_sub(
          owner_kind,
          &owner_path,
          expected_epoch,
          new_dir,
          entry_name,
          notify_path,
        )],
        LogicalKind::Directory => {
          let full_dir = InternedPath::from(new_dir.as_path().join(&entry_name));
          self.push_directory_subs(&owner_path, expected_epoch, &full_dir, &notify_path)
        }
      },
    };

    if let Some(reg) = self.registrations[owner_kind as usize].get_mut(&owner_path) {
      reg.sub_ids.retain(|&id| id != sub_id);
      reg.sub_ids.extend(new_ids);
    }
    true
  }

  /// Builds a minimal disk check plan for an active scan (`scan_once`) without holding I/O in lock.
  #[cfg(feature = "testing")]
  pub fn plan_active_scan(&self) -> ActiveScanPlan {
    let mut context_dirs = Vec::new();
    let mut stat_entries = Vec::new();
    let mut waiting_entries = Vec::new();

    for (dir_path, node) in &self.directories {
      if !node.context_subs.is_empty() {
        let mut bindings = Vec::new();
        for &sub_id in &node.context_subs {
          if let Some(sub) = self.get_sub(sub_id)
            && let SubscriptionRole::ContextDir {
              logical_dir,
              is_root,
              ..
            } = &sub.role
          {
            bindings.push((
              sub_id,
              sub.epoch,
              sub.owner_path.clone(),
              logical_dir.clone(),
              *is_root,
            ));
          }
        }
        if !bindings.is_empty() {
          let mut prev_entries = FxHashMap::default();
          for (k, slot) in &node.children {
            if let Some(obs) = slot.observed {
              prev_entries.insert(k.clone(), obs);
            }
          }
          context_dirs.push(ActiveContextScan {
            dir: dir_path.clone(),
            bindings,
            prev_entries,
            had_complete_listing: node.has_complete_listing,
          });
        }
      }

      for (entry_name, slot) in &node.children {
        if !slot.entry_subs.is_empty() {
          let mut targets = Vec::new();
          for &sub_id in &slot.entry_subs {
            if let Some(sub) = self.get_sub(sub_id)
              && let SubscriptionRole::TargetEntry { notify_path, .. } = &sub.role
              && matches!(sub.owner_kind, LogicalKind::File | LogicalKind::Missing)
            {
              targets.push((
                sub_id,
                sub.epoch,
                sub.owner_kind,
                sub.owner_path.clone(),
                notify_path.clone(),
              ));
            }
          }
          if !targets.is_empty() {
            stat_entries.push(ActiveEntryScan {
              dir: dir_path.clone(),
              name: entry_name.clone(),
              full_path: InternedPath::from(dir_path.as_path().join(entry_name)),
              targets,
            });
          }
        }

        for &sub_id in &slot.waiting_subs {
          if let Some(sub) = self.get_sub(sub_id)
            && let SubscriptionRole::WaitingPrefix {
              dir,
              wait_name,
              remainder,
              notify_path,
            } = &sub.role
          {
            waiting_entries.push(PendingAdvance {
              sub_id,
              owner_kind: sub.owner_kind,
              owner_path: sub.owner_path.clone(),
              epoch: sub.epoch,
              current_dir: dir.clone(),
              wait_name: wait_name.clone(),
              remainder: remainder.clone(),
              notify_path: notify_path.clone(),
            });
          }
        }
      }
    }

    ActiveScanPlan {
      context_dirs,
      stat_entries,
      waiting_entries,
    }
  }

  /// Active directory count for `index_sizes()` reclamation checks.
  pub fn directory_count(&self) -> usize {
    self.directories.len()
  }

  /// Collects already-bound `TargetEntry` subscriptions (`File` / `Missing`) inside `root_dir`'s
  /// active subtree in `SubscriptionGraph` without scanning unsubscribed disk directories.
  #[inline(never)]
  pub fn collect_subtree_targets(&self, root_dir: &InternedPath) -> Vec<MatchedTarget> {
    let mut results = Vec::new();
    if !self.directories.contains_key(root_dir) {
      return results;
    }
    let mut stack = vec![root_dir.clone()];
    while let Some(dir) = stack.pop() {
      let Some(node) = self.directories.get(&dir) else {
        continue;
      };
      for (child_name, slot) in &node.children {
        if slot.has_child_node {
          stack.push(InternedPath::from(dir.as_path().join(child_name)));
        }
        for &sub_id in &slot.entry_subs {
          if let Some(sub) = self.get_sub(sub_id)
            && matches!(sub.owner_kind, LogicalKind::File | LogicalKind::Missing)
            && let SubscriptionRole::TargetEntry { notify_path, .. } = &sub.role
            && let Some(reg) = self.registrations[sub.owner_kind as usize].get(&sub.owner_path)
          {
            results.push(MatchedTarget {
              owner_path: sub.owner_path.clone(),
              target_path: reg.target_path.clone(),
              notify_path: notify_path.clone(),
            });
          }
        }
      }
    }
    results
  }

  fn get_sub(&self, id: SubId) -> Option<&Subscription> {
    self
      .subscriptions
      .get(id.0 as usize)
      .and_then(Option::as_ref)
  }

  fn alloc_sub(&mut self, sub: Subscription) -> SubId {
    if let Some(idx) = self.free_sub_ids.pop() {
      self.subscriptions[idx as usize] = Some(sub);
      SubId(idx)
    } else {
      let idx = self.subscriptions.len() as u32;
      self.subscriptions.push(Some(sub));
      SubId(idx)
    }
  }

  fn push_target_entry_sub(
    &mut self,
    kind: LogicalKind,
    owner_path: &InternedPath,
    epoch: u64,
    dir: InternedPath,
    name: OsString,
    notify_path: InternedPath,
  ) -> SubId {
    let sub_id = self.alloc_sub(Subscription {
      owner_kind: kind,
      owner_path: owner_path.clone(),
      epoch,
      role: SubscriptionRole::TargetEntry {
        dir: dir.clone(),
        name: name.clone(),
        notify_path,
      },
    });
    self
      .ensure_dir_node(&dir)
      .children
      .entry(name)
      .or_default()
      .entry_subs
      .push(sub_id);
    sub_id
  }

  fn push_waiting_sub(
    &mut self,
    kind: LogicalKind,
    owner_path: &InternedPath,
    epoch: u64,
    (dir, wait_name, remainder): (InternedPath, OsString, PathBuf),
    notify_path: InternedPath,
  ) -> SubId {
    let sub_id = self.alloc_sub(Subscription {
      owner_kind: kind,
      owner_path: owner_path.clone(),
      epoch,
      role: SubscriptionRole::WaitingPrefix {
        dir: dir.clone(),
        wait_name: wait_name.clone(),
        remainder,
        notify_path,
      },
    });
    self
      .ensure_dir_node(&dir)
      .children
      .entry(wait_name)
      .or_default()
      .waiting_subs
      .push(sub_id);
    sub_id
  }

  fn push_directory_subs(
    &mut self,
    owner_path: &InternedPath,
    epoch: u64,
    target_path: &InternedPath,
    notify_path: &InternedPath,
  ) -> Vec<SubId> {
    let mut ids = Vec::with_capacity(2);
    let ctx_id = self.alloc_sub(Subscription {
      owner_kind: LogicalKind::Directory,
      owner_path: owner_path.clone(),
      epoch,
      role: SubscriptionRole::ContextDir {
        dir: target_path.clone(),
        logical_dir: notify_path.clone(),
        is_root: true,
      },
    });
    self.ensure_dir_node(target_path).context_subs.push(ctx_id);
    ids.push(ctx_id);

    if let (Some(parent), Some(name)) = (target_path.parent(), target_path.file_name()) {
      ids.push(self.push_target_entry_sub(
        LogicalKind::Directory,
        owner_path,
        epoch,
        InternedPath::from(parent),
        name.to_os_string(),
        notify_path.clone(),
      ));
    }
    ids
  }

  fn attach_initial_subs(
    &mut self,
    kind: LogicalKind,
    owner_path: &InternedPath,
    epoch: u64,
    target_path: &InternedPath,
  ) -> Vec<SubId> {
    match kind {
      LogicalKind::File | LogicalKind::Missing => {
        if let (Some(parent), Some(name)) = (target_path.parent(), target_path.file_name())
          && parent.is_dir()
        {
          return vec![self.push_target_entry_sub(
            kind,
            owner_path,
            epoch,
            InternedPath::from(parent),
            name.to_os_string(),
            owner_path.clone(),
          )];
        }
      }
      LogicalKind::Directory => {
        if target_path.is_dir() {
          return self.push_directory_subs(owner_path, epoch, target_path, owner_path);
        }
      }
    }

    if let Some(split) = find_waiting_split(target_path.as_path()) {
      return vec![self.push_waiting_sub(kind, owner_path, epoch, split, owner_path.clone())];
    }
    Vec::new()
  }

  fn detach_registration_subs(&mut self, kind: LogicalKind, owner_path: &InternedPath) {
    let Some(sub_ids) = self.registrations[kind as usize]
      .get_mut(owner_path)
      .map(|reg| std::mem::take(&mut reg.sub_ids))
    else {
      return;
    };
    for sub_id in sub_ids {
      self.remove_sub(sub_id);
    }
  }

  fn remove_sub(&mut self, sub_id: SubId) {
    let Some(slot_opt) = self.subscriptions.get_mut(sub_id.0 as usize) else {
      return;
    };
    let Some(sub) = slot_opt.take() else {
      return;
    };
    self.free_sub_ids.push(sub_id.0);

    let dir = match &sub.role {
      SubscriptionRole::TargetEntry { dir, name, .. } => {
        if let Some(node) = self.directories.get_mut(dir) {
          let drop_child = if let Some(slot) = node.children.get_mut(name) {
            slot.entry_subs.retain(|&id| id != sub_id);
            node.context_subs.is_empty() && slot.is_unreferenced()
          } else {
            false
          };
          if drop_child {
            node.children.remove(name);
          }
        }
        dir.clone()
      }
      SubscriptionRole::WaitingPrefix { dir, wait_name, .. } => {
        if let Some(node) = self.directories.get_mut(dir) {
          let drop_child = if let Some(slot) = node.children.get_mut(wait_name) {
            slot.waiting_subs.retain(|&id| id != sub_id);
            (node.context_subs.is_empty() || !slot.has_observed()) && slot.is_unreferenced()
          } else {
            false
          };
          if drop_child {
            node.children.remove(wait_name);
          }
        }
        dir.clone()
      }
      SubscriptionRole::ContextDir { dir, .. } => {
        if let Some(node) = self.directories.get_mut(dir) {
          node.context_subs.retain(|&id| id != sub_id);
          if node.context_subs.is_empty() {
            #[cfg(feature = "testing")]
            {
              node.has_complete_listing = false;
            }
            node.children.retain(|_, slot| !slot.is_unreferenced());
          }
        }
        dir.clone()
      }
    };
    self.prune_empty_dir_chain(&dir);
  }

  fn ensure_dir_node(&mut self, dir: &InternedPath) -> &mut DirectoryNode {
    if !self.directories.contains_key(dir) {
      self
        .directories
        .insert(dir.clone(), DirectoryNode::default());
      let mut child = dir.clone();
      while let Some(parent) = child.parent().map(InternedPath::from) {
        let Some(name) = child.file_name().map(OsStr::to_os_string) else {
          break;
        };
        let existed = self.directories.contains_key(&parent);
        self
          .directories
          .entry(parent.clone())
          .or_default()
          .children
          .entry(name)
          .or_default()
          .has_child_node = true;
        if existed {
          break;
        }
        child = parent;
      }
    }
    self
      .directories
      .get_mut(dir)
      .expect("directory node just inserted")
  }

  fn prune_empty_dir_chain(&mut self, start_dir: &InternedPath) {
    let mut current = start_dir.clone();
    while self
      .directories
      .get(&current)
      .is_some_and(DirectoryNode::is_empty)
    {
      self.directories.remove(&current);
      let (Some(parent), Some(name)) = (current.parent(), current.file_name()) else {
        break;
      };
      let parent_ip = InternedPath::from(parent);
      if let Some(parent_node) = self.directories.get_mut(&parent_ip) {
        let drop_child = if let Some(slot) = parent_node.children.get_mut(name) {
          slot.has_child_node = false;
          (parent_node.context_subs.is_empty() || !slot.has_observed()) && slot.is_unreferenced()
        } else {
          false
        };
        if drop_child {
          parent_node.children.remove(name);
        }
      }
      current = parent_ip;
    }
  }
}

pub(crate) enum WaitingStepResult {
  StillWaiting {
    wait_name: OsString,
    remainder: PathBuf,
  },
  ReachedEntry {
    entry_name: OsString,
  },
}

/// Walks down `current_dir / wait_name / remainder` on disk without holding any lock.
/// Returns `None` if `current_dir / wait_name` is still absent on disk.
pub(crate) fn probe_waiting_advance(
  owner_kind: LogicalKind,
  current_dir: &Path,
  wait_name: &OsStr,
  remainder: &Path,
) -> Option<(InternedPath, WaitingStepResult)> {
  let first_step = current_dir.join(wait_name);
  if remainder.as_os_str().is_empty() {
    let present = match owner_kind {
      LogicalKind::Directory => first_step.is_dir(),
      LogicalKind::File | LogicalKind::Missing => std::fs::symlink_metadata(&first_step).is_ok(),
    };
    if !present {
      return None;
    }
    return Some((
      InternedPath::from(current_dir),
      WaitingStepResult::ReachedEntry {
        entry_name: wait_name.to_os_string(),
      },
    ));
  }

  if !first_step.is_dir() {
    return None;
  }

  let components: Vec<OsString> = remainder
    .components()
    .filter_map(|c| match c {
      Component::Normal(s) => Some(s.to_os_string()),
      _ => None,
    })
    .collect();
  if components.is_empty() {
    return Some((
      InternedPath::from(current_dir),
      WaitingStepResult::ReachedEntry {
        entry_name: wait_name.to_os_string(),
      },
    ));
  }

  let mut dir = first_step;
  for (idx, comp) in components.iter().enumerate() {
    let is_last = idx + 1 == components.len();
    if is_last {
      let leaf = dir.join(comp);
      let present = match owner_kind {
        LogicalKind::Directory => leaf.is_dir(),
        LogicalKind::File | LogicalKind::Missing => std::fs::symlink_metadata(&leaf).is_ok(),
      };
      if present {
        return Some((
          InternedPath::from(dir),
          WaitingStepResult::ReachedEntry {
            entry_name: comp.clone(),
          },
        ));
      }
      return Some((
        InternedPath::from(dir),
        WaitingStepResult::StillWaiting {
          wait_name: comp.clone(),
          remainder: PathBuf::new(),
        },
      ));
    }

    let next_dir = dir.join(comp);
    if next_dir.is_dir() {
      dir = next_dir;
    } else {
      let mut rem = PathBuf::new();
      for rest in &components[idx + 1..] {
        rem.push(rest);
      }
      return Some((
        InternedPath::from(dir),
        WaitingStepResult::StillWaiting {
          wait_name: comp.clone(),
          remainder: rem,
        },
      ));
    }
  }
  None
}

fn find_waiting_split(target_path: &Path) -> Option<(InternedPath, OsString, PathBuf)> {
  for ancestor in target_path.ancestors().skip(1) {
    if ancestor.is_dir() {
      let rel = target_path.strip_prefix(ancestor).ok()?;
      let mut comps = rel.components().filter_map(|c| match c {
        Component::Normal(s) => Some(s.to_os_string()),
        _ => None,
      });
      let wait_name = comps.next()?;
      let mut remainder = PathBuf::new();
      for rest in comps {
        remainder.push(rest);
      }
      return Some((InternedPath::from(ancestor), wait_name, remainder));
    }
  }
  None
}
