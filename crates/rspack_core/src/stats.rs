use rspack_error::emitter::emit_batch_diagnostic;

use crate::{Compilation, CompilationAssets};

#[derive(Debug)]
pub struct Stats<'compilation> {
  compilation: &'compilation Compilation,
}

impl<'compilation> Stats<'compilation> {
  pub fn new(compilation: &'compilation Compilation) -> Self {
    Self { compilation }
  }

  pub fn emit_error(&self) {
    emit_batch_diagnostic(&self.compilation.diagnostic);
  }
}

impl<'compilation> Stats<'compilation> {
  // This function is only used for tests compatiable.
  pub fn __should_only_used_in_tests_assets(&self) -> &CompilationAssets {
    &self.compilation.assets
  }

  pub fn to_description(self) -> StatsDescription {
    StatsDescription {
      assets: self
        .compilation
        .assets
        .iter()
        .map(|(filename, _asset)| StatsAsset {
          name: filename.clone(),
        })
        .collect(),
    }
  }
}

pub struct StatsDescription {
  pub assets: Vec<StatsAsset>,
  // pub entrypoints: HashMap<String, StatsEntrypoint>,
}

pub struct StatsAsset {
  pub name: String,
}

pub struct StatsAssetReference {
  pub name: String,
}

pub struct StatsEntrypoint {
  pub name: String,
  pub assets: Vec<StatsAssetReference>,
}
