use rspack_error::Diagnostic;
use rspack_paths::ArcPathIndexSet;

use crate::CompilationAssets;

#[derive(Debug, Default)]
pub struct ProcessAssetsArtifact {
  pub assets: CompilationAssets,
  pub file_dependencies: ArcPathIndexSet,
  pub context_dependencies: ArcPathIndexSet,
  pub diagnostics: Vec<Diagnostic>,
}

impl ProcessAssetsArtifact {
  pub fn new(
    assets: CompilationAssets,
    file_dependencies: ArcPathIndexSet,
    context_dependencies: ArcPathIndexSet,
  ) -> Self {
    Self {
      assets,
      file_dependencies,
      context_dependencies,
      diagnostics: vec![],
    }
  }
}
