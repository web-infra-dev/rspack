use std::{fmt::Write as _, time::Duration};

use rspack_paths::{ArcPath, ArcPathSet};

use super::{
  occasion::Occasion,
  snapshot::{Snapshot, SnapshotScope},
  storage::BoxStorage,
  validation::{CacheValidation, CacheValidationResult},
};
use crate::{CompilationLogger, LogType, Logger};

const PATH_LOG_LIMIT: usize = 3;

/// Per-build runtime state shared across all cache operations.
///
/// `load_failed` gates every `load_*` call in a single build: once any
/// load fails it is set to `true` and all subsequent loads are skipped.
/// Call [`CacheContext::reset`] at the end of each build to prepare the
/// context for the next one.
#[derive(Debug)]
pub struct CacheContext {
  /// Set when cache compatibility validation fails, meaning the cached data
  /// is structurally stale. Unlike `load_failed`, this flag persists across
  /// builds in readonly mode because the cache cannot be rebuilt there.
  invalid: bool,
  /// Per-build load gate.  Flipped to `true` on the first failed `load_*`
  /// call; all subsequent `load_*` calls become no-ops for this build.
  /// Restored to `false` (or derived from `invalid`) by `reset`.
  load_failed: bool,
  /// When `true`, all `save_*` and storage reset calls are skipped.
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

  /// Validates all compatibility inputs before any compilation cache item is
  /// read.
  ///
  /// Like webpack's pack validation, both build dependency changes and a
  /// cache version mismatch enter the same invalidation path. The whole
  /// compiler cache is reset so a scope that is not used by the invalidating
  /// build cannot survive and be reused later.
  #[tracing::instrument("Cache::Context::validate", skip_all)]
  pub async fn validate(&mut self, validation: &mut CacheValidation) {
    let report = validation.validate(&*self.storage).await;
    match report.result {
      CacheValidationResult::Valid { tracked_files } => {
        self.logger().info(format!(
          "build dependencies are valid ({tracked_files} tracked)"
        ));
        self.log_duration(
          "validate build dependencies",
          report
            .build_dependencies_duration
            .expect("build dependencies should have been validated"),
        );

        self
          .logger()
          .info("meta persistent cache recovery succeeded");
        self.log_duration(read_occasion_timing_label("meta"), report.version_duration);
      }
      CacheValidationResult::InvalidVersion { message } => {
        self
          .logger()
          .warn(format!("meta persistent cache recovery failed: {message}"));
        self.log_duration(read_occasion_timing_label("meta"), report.version_duration);
        self.invalidate();
      }
      CacheValidationResult::InvalidBuildDependencies {
        modified_files,
        removed_files,
      } => {
        let reason = format_path_changes(&modified_files, &removed_files);
        self.logger().warn(format!(
          "persistent cache invalidated because build dependencies changed:\n{reason}"
        ));
        self.log_duration(
          "validate build dependencies",
          report
            .build_dependencies_duration
            .expect("build dependencies should have been validated"),
        );
        self.invalidate();
      }
      CacheValidationResult::VersionError(error) => {
        self
          .logger()
          .warn(format!("meta persistent cache recovery failed: {error}"));
        self.log_duration(read_occasion_timing_label("meta"), report.version_duration);
        self.invalidate();
      }
      CacheValidationResult::BuildDependenciesError(error) => {
        self
          .logger()
          .warn(format!("build dependencies validation failed: {error}"));
        self.log_duration(
          "validate build dependencies",
          report
            .build_dependencies_duration
            .expect("build dependencies should have been validated"),
        );
        self.invalidate();
      }
    }
  }

  fn log_duration(&self, label: &'static str, duration: Duration) {
    self.logger().raw(LogType::Time {
      label,
      secs: duration.as_secs(),
      subsec_nanos: duration.subsec_nanos(),
    });
  }

  fn invalidate(&mut self) {
    self.invalid = true;
    self.load_failed = true;
    if self.readonly {
      self
        .logger()
        .warn("persistent cache is readonly, stale entries will not be rewritten");
    } else {
      self.storage.reset_all();
    }
  }

  /// Saves build dependency hashes. No-op in readonly mode.
  #[tracing::instrument("Cache::Context::save_build_deps", skip_all)]
  pub async fn save_build_deps(
    &mut self,
    validation: &mut CacheValidation,
    added: impl Iterator<Item = ArcPath>,
  ) {
    if self.readonly {
      return;
    }

    let start = self
      .logger()
      .time("write build dependencies to persistent cache");
    let logger = self.logger().clone();
    validation
      .add_build_dependencies(&mut *self.storage, added, logger)
      .await;
    self.logger().time_end(start);
  }

  /// Persists compatibility metadata. No-op in readonly mode.
  pub fn save_validation(&mut self, validation: &CacheValidation) {
    if self.readonly {
      return;
    }

    let start = self.logger().time("write meta to persistent cache");
    validation.save(&mut *self.storage);
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

  /// Loads an occasion's cache item from storage.
  ///
  /// Returns `None` and resets the occasion's scope when the cache is
  /// invalid or recovery fails.
  #[tracing::instrument("Cache::Context::load_occasion", skip_all)]
  pub async fn load_occasion<O: Occasion>(&mut self, occasion: &O) -> Option<O::CacheItem> {
    if !self.load_failed {
      let start = self
        .logger()
        .time(read_occasion_timing_label(occasion.name()));
      match occasion.recovery(&*self.storage).await {
        Ok(cache_item) => {
          self.logger().info(format!(
            "{} persistent cache recovery succeeded",
            occasion.name()
          ));
          self.logger().time_end(start);
          return Some(cache_item);
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

  /// Persists an occasion's cache item. No-op in readonly mode.
  #[tracing::instrument("Cache::Context::save_occasion", skip_all)]
  pub fn save_occasion<O: Occasion>(&mut self, occasion: &O, cache_item: &O::CacheItem) {
    if self.readonly {
      return;
    }

    let start = self
      .logger()
      .time(write_occasion_timing_label(occasion.name()));
    occasion.save(&mut *self.storage, cache_item);
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
  /// In non-readonly mode both flags are cleared; storage resets done during
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
    "source map" => "read source map from persistent cache",
    _ => "read occasion from persistent cache",
  }
}

fn write_occasion_timing_label(name: &'static str) -> &'static str {
  match name {
    "make" => "write make to persistent cache",
    "meta" => "write meta to persistent cache",
    "minimize" => "write minimize to persistent cache",
    "source map" => "write source map to persistent cache",
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
