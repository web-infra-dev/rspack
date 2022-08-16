use rspack_error::Diagnostic;

use crate::{Compilation, CompilationAssets};

#[derive(Debug)]
pub struct Stats<'compilation> {
  compilation: &'compilation Compilation,
  // TODO: Remove this suppresion
  #[allow(unused)]
  pub diagnostics: Vec<Diagnostic>,
}

impl<'compilation> Stats<'compilation> {
  pub fn new(compilation: &'compilation Compilation, diagnostics: Vec<Diagnostic>) -> Self {
    Self {
      compilation,
      diagnostics,
    }
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
