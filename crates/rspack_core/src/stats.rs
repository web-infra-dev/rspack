use crate::{Compilation, CompilationAssets, PATH_START_BYTE_POS_MAP};
use rspack_error::{
  emitter::{DiagnosticDisplay, StdioDiagnosticDisplay, StringDiagnosticDisplay},
  Result,
};

#[derive(Debug)]
pub struct Stats<'compilation> {
  pub compilation: &'compilation Compilation,
}

impl<'compilation> Stats<'compilation> {
  pub fn new(compilation: &'compilation Compilation) -> Self {
    Self { compilation }
  }

  pub fn emit_error(&self) -> Result<()> {
    StdioDiagnosticDisplay::default().emit_batch_diagnostic(
      &self.compilation.diagnostic,
      PATH_START_BYTE_POS_MAP.clone(),
    )
  }

  pub fn emit_error_string(&self) -> Result<String> {
    StringDiagnosticDisplay::default().emit_batch_diagnostic(
      &self.compilation.diagnostic,
      PATH_START_BYTE_POS_MAP.clone(),
    )
  }
}

impl<'compilation> Stats<'compilation> {
  // This function is only used for tests compatible.
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
