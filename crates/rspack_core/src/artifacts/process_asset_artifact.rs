use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{CompilationAssets, compiler::CompilationRecords};

#[derive(Debug, Default)]
pub struct ProcessAssetArtifact {
  pub assets: CompilationAssets,
  pub assets_related_in: HashMap<String, HashSet<String>>,
  pub diagnostics: Vec<rspack_error::Diagnostic>,
  pub records: Option<CompilationRecords>,
  pub file_dependencies: rspack_paths::ArcPathIndexSet,
  pub context_dependencies: rspack_paths::ArcPathIndexSet,
  pub code_generated_modules: rspack_collections::IdentifierSet,
}
