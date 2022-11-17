#[derive(Debug, Clone, Default)]
pub struct SnapshotStrategy {
  pub hash: bool,
  pub timestamp: bool,
}

#[derive(Debug, Clone, Default)]
pub struct SnapshotOptions {
  /// Snapshots for resolving of build dependencies when using the persistent cache.
  pub resolve_build_dependencies: SnapshotStrategy,
  /// Snapshots for build dependencies when using the persistent cache.
  pub build_dependencies: SnapshotStrategy,
  /// Snapshots for resolving of requests.
  pub resolve: SnapshotStrategy,
  /// Snapshots for building modules.
  pub module: SnapshotStrategy,
  // An array of paths that are managed by a package manager and contain a version or a hash in their paths.
  // immutable_paths: Vec<String>,
  // An array of paths that are managed by a package manager.
  // managed_paths: Vec<String>,
}
