#[derive(Debug, Clone, Copy)]
pub enum SnapshotScope {
  /// for compilation.file_dependencies
  FILE,
  /// for compilation.context_dependencies
  CONTEXT,
  /// for compilation.missing_dependencies
  MISSING,
  /// for compilation.build_dependencies
  BUILD,
}

impl SnapshotScope {
  pub fn name(&self) -> &'static str {
    match self {
      Self::FILE => "snapshot_file",
      Self::CONTEXT => "snapshot_context",
      Self::MISSING => "snapshot_missing",
      Self::BUILD => "snapshot_build",
    }
  }
}
