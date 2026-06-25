use std::fmt::Write as _;

use rspack_paths::{ArcPath, ArcPathSet};

use super::{
  build_dependencies::{BuildDeps, BuildDepsValidationResult},
  occasion::Occasion,
  snapshot::{Snapshot, SnapshotScope},
  storage::BoxStorage,
};
use crate::{CompilationLogger, Logger};

const PATH_LOG_LIMIT: usize = 3;

/// Per-build runtime state shared across all cache operations.
///
/// `load_failed` gates every `load_*` call in a single build: once any
/// load fails it is set to `true` and all subsequent loads are skipped.
/// Call [`CacheContext::reset`] at the end of each build to prepare the
/// context for the next one.
#[derive(Debug)]
pub struct CacheContext {
  /// Set when build dependencies have changed, meaning the cached data is
  /// structurally stale.  Unlike `load_failed`, this flag persists across
  /// builds in readonly mode because the cache cannot be rebuilt there.
  invalid: bool,
  /// Per-build load gate.  Flipped to `true` on the first failed `load_*`
  /// call; all subsequent `load_*` calls become no-ops for this build.
  /// Restored to `false` (or derived from `invalid`) by `reset`.
  load_failed: bool,
  /// When `true`, all `save_*` and scope `reset` calls to storage are skipped.
  ///
  /// This is a user-configured option, distinct from `DB::readonly` in the
  /// storage layer.  Skipping at this level is cheaper: occasion serialisation
  /// and snapshot diffing are never executed, whereas `DB::readonly` only
  /// suppresses the final disk write after all that work has already been done.
  readonly: bool,

  logger: CompilationLogger,
  storage: BoxStorage,
}

impl CacheContext {
  pub fn new(storage: BoxStorage, readonly: bool, logger: CompilationLogger) -> Self {
    Self {
      invalid: false,
      load_failed: false,
      readonly,
      logger,
      storage,
    }
  }

  pub fn logger(&self) -> &CompilationLogger {
    &self.logger
  }

  pub fn cleanup_stale(&self) {
    if self.readonly {
      return;
    }
    self.storage.cleanup_stale();
  }

  /// Validates build dependencies and sets `invalid` + `load_failed` on
  /// failure.  Resets the BUILD scope when invalid and not readonly.
  ///
  /// Normally called only once per compiler instance, guarded by the
  /// `initialized` flag in `PersistentCache::initialize`.
  #[tracing::instrument("Cache::Context::load_build_deps", skip_all)]
  pub async fn load_build_deps(&mut self, build_deps: &mut BuildDeps) {
    let start = self.logger().time("validate build dependencies");
    match build_deps.validate(&*self.storage).await {
      Ok(BuildDepsValidationResult::Valid { tracked_files }) => {
        self.invalid = false;
        self.logger().info(format!(
          "build dependencies are valid ({tracked_files} tracked)"
        ));
      }
      Ok(BuildDepsValidationResult::Invalid {
        modified_files,
        removed_files,
      }) => {
        self.invalid = true;
        self.load_failed = true;
        let reason = format_path_changes(&modified_files, &removed_files);
        self.logger().warn(format!(
          "persistent cache invalidated because build dependencies changed:\n{reason}"
        ));
        if self.readonly {
          self
            .logger()
            .warn("persistent cache is readonly, stale entries will not be rewritten");
        }
      }
      Err(err) => {
        self.load_failed = true;
        self
          .logger()
          .warn(format!("build dependencies validation failed: {err}"));
      }
    }
    self.logger().time_end(start);
    if self.load_failed && !self.readonly {
      build_deps.reset(&mut *self.storage);
    }
  }

  /// Saves build dependency hashes. No-op in readonly mode.
  #[tracing::instrument("Cache::Context::save_build_deps", skip_all)]
  pub async fn save_build_deps(
    &mut self,
    build_deps: &mut BuildDeps,
    added: impl Iterator<Item = ArcPath>,
  ) {
    if self.readonly {
      return;
    }

    let start = self
      .logger()
      .time("write build dependencies to persistent cache");
    let logger = self.logger().clone();
    build_deps.add(&mut *self.storage, added, logger).await;
    self.logger().time_end(start);
  }

  /// Computes modified/removed paths from all snapshot scopes.
  ///
  /// Returns `None` when the cache is invalid or any scope fails to load.
  /// On failure all snapshot scopes are reset (unless readonly) so they
  /// are fully rewritten this build.
  #[tracing::instrument("Cache::Context::load_snapshot", skip_all)]
  pub async fn load_snapshot(
    &mut self,
    snapshot: &Snapshot,
  ) -> Option<(bool, ArcPathSet, ArcPathSet)> {
    if !self.load_failed {
      let start = self.logger().time("read snapshot from persistent cache");
      let mut is_hot_start = false;
      let mut modified_paths = ArcPathSet::default();
      let mut removed_paths = ArcPathSet::default();
      let data = vec![
        snapshot
          .calc_modified_paths(&*self.storage, SnapshotScope::FILE)
          .await,
        snapshot
          .calc_modified_paths(&*self.storage, SnapshotScope::CONTEXT)
          .await,
        snapshot
          .calc_modified_paths(&*self.storage, SnapshotScope::MISSING)
          .await,
      ];
      for item in data {
        match item {
          Ok((a, b, c, _)) => {
            is_hot_start = is_hot_start || a;
            modified_paths.extend(b);
            removed_paths.extend(c);
          }
          Err(err) => {
            self.load_failed = true;
            self
              .logger()
              .warn(format!("snapshot scope load failed: {err}"));
          }
        }
      }
      if !self.load_failed {
        self.logger().time_end(start);
        if is_hot_start {
          if modified_paths.is_empty() && removed_paths.is_empty() {
            self
              .logger()
              .info("snapshot restored with no changed dependencies");
          } else {
            self.logger().info(format!(
              "snapshot restored with detected changed dependencies:\n{}",
              format_path_changes(&modified_paths, &removed_paths)
            ));
          }
        }
        return Some((is_hot_start, modified_paths, removed_paths));
      }
      self.logger().time_end(start);
    }

    // load_failed: reset snapshot scopes so they are fully rewritten this build.
    if !self.readonly {
      snapshot.reset(&mut *self.storage);
    }
    None
  }

  /// Persists snapshot data for all three scopes. No-op in readonly mode.
  #[tracing::instrument("Cache::Context::save_snapshot", skip_all)]
  pub async fn save_snapshot(
    &mut self,
    snapshot: &Snapshot,
    file_deps: (impl Iterator<Item = ArcPath>, impl Iterator<Item = ArcPath>),
    context_deps: (impl Iterator<Item = ArcPath>, impl Iterator<Item = ArcPath>),
    missing_deps: (impl Iterator<Item = ArcPath>, impl Iterator<Item = ArcPath>),
  ) {
    if self.readonly {
      return;
    }

    let start = self.logger().time("write snapshot to persistent cache");
    let (file_added, file_removed) = file_deps;
    let (context_added, context_removed) = context_deps;
    let (missing_added, missing_removed) = missing_deps;
    snapshot.remove(&mut *self.storage, SnapshotScope::FILE, file_removed);
    snapshot.remove(&mut *self.storage, SnapshotScope::CONTEXT, context_removed);
    snapshot.remove(&mut *self.storage, SnapshotScope::MISSING, missing_removed);
    snapshot
      .add(&mut *self.storage, SnapshotScope::FILE, file_added)
      .await;
    snapshot
      .add(&mut *self.storage, SnapshotScope::CONTEXT, context_added)
      .await;
    snapshot
      .add(&mut *self.storage, SnapshotScope::MISSING, missing_added)
      .await;
    self.logger().time_end(start);
  }

  /// Loads an occasion's artifact from storage.
  ///
  /// Returns `None` and resets the occasion's scope when the cache is
  /// invalid or recovery fails.
  #[tracing::instrument("Cache::Context::load_occasion", skip_all)]
  pub async fn load_occasion<O: Occasion>(&mut self, occasion: &O) -> Option<O::Artifact> {
    if !self.load_failed {
      let start = self
        .logger()
        .time(read_occasion_timing_label(occasion.name()));
      match occasion.recovery(&*self.storage).await {
        Ok(artifact) => {
          self.logger().info(format!(
            "{} persistent cache recovery succeeded",
            occasion.name()
          ));
          self.logger().time_end(start);
          return Some(artifact);
        }
        Err(err) => {
          self.load_failed = true;
          self.logger().warn(format!(
            "{} persistent cache recovery failed: {err}",
            occasion.name()
          ));
        }
      }
      self.logger().time_end(start);
    }
    if !self.readonly {
      occasion.reset(&mut *self.storage);
    }
    None
  }

  /// Persists an occasion's artifact. No-op in readonly mode.
  #[tracing::instrument("Cache::Context::save_occasion", skip_all)]
  pub fn save_occasion<O: Occasion>(&mut self, occasion: &O, artifact: &O::Artifact) {
    if self.readonly {
      return;
    }

    let start = self
      .logger()
      .time(write_occasion_timing_label(occasion.name()));
    occasion.save(&mut *self.storage, artifact);
    self.logger().time_end(start);
  }

  /// Enqueues a background persistence flush. No-op in readonly mode.
  ///
  /// The write completes asynchronously; call [`CacheContext::flush_storage`]
  /// to wait for it.
  pub fn save_storage(&mut self) {
    if self.readonly {
      return;
    }

    let start = self.logger().time("stage persistent cache");
    self.storage.save();
    self.logger().time_end(start);
  }

  /// Waits for all background storage writes to complete.
  ///
  /// Must be called before process exit to avoid losing buffered data.
  pub async fn flush_storage(&self) {
    let start = self.logger().time("flush persistent cache to disk");
    self.storage.flush().await;
    self.logger().time_end(start);
  }

  /// Resets per-build state.
  ///
  /// In non-readonly mode both flags are cleared; scope resets done during
  /// this build ensure a clean slate next time.
  ///
  /// In readonly mode `invalid` is preserved (the cache is still stale and
  /// cannot be rebuilt), so `load_failed` is derived from it — stale-cache
  /// loads are skipped on the next build as well.  Transient errors
  /// (`load_failed` without `invalid`) are cleared so the next build retries.
  pub fn reset(&mut self) {
    if !self.readonly {
      self.invalid = false;
      self.load_failed = false
    } else {
      self.load_failed = self.invalid;
    }
  }
}

fn read_occasion_timing_label(name: &'static str) -> &'static str {
  match name {
    "make" => "read make from persistent cache",
    "meta" => "read meta from persistent cache",
    "minimize" => "read minimize from persistent cache",
    _ => "read occasion from persistent cache",
  }
}

fn write_occasion_timing_label(name: &'static str) -> &'static str {
  match name {
    "make" => "write make to persistent cache",
    "meta" => "write meta to persistent cache",
    "minimize" => "write minimize to persistent cache",
    _ => "write occasion to persistent cache",
  }
}

fn format_path_changes(modified_paths: &ArcPathSet, removed_paths: &ArcPathSet) -> String {
  let mut changes = String::new();
  if !modified_paths.is_empty() {
    append_paths_group(&mut changes, "modified paths", modified_paths);
  }
  if !removed_paths.is_empty() {
    if !changes.is_empty() {
      changes.push('\n');
    }
    append_paths_group(&mut changes, "removed paths", removed_paths);
  }
  changes
}

fn append_paths_group(output: &mut String, label: &str, paths: &ArcPathSet) {
  let mut paths = paths
    .iter()
    .map(|path| path.as_ref())
    .collect::<Vec<&std::path::Path>>();
  paths.sort_unstable();

  let _ = write!(output, "{label} ({}):", paths.len());
  let is_truncated = paths.len() > PATH_LOG_LIMIT;
  paths.truncate(PATH_LOG_LIMIT);
  for path in paths {
    let _ = write!(output, "\n  - {}", path.display());
  }
  if is_truncated {
    output.push_str("\n  - ...");
  }
}
