#[derive(Debug)]
pub struct Experiments {
  pub layers: bool,
  pub incremental: Incremental,
  pub top_level_await: bool,
  pub rspack_future: RspackFuture,
}

#[allow(clippy::empty_structs_with_brackets)]
#[derive(Debug)]
pub struct RspackFuture {}

#[derive(Debug)]
pub enum Incremental {
  Disabled,
  Enabled {
    make: bool,
    emit_assets: bool,
    infer_async_modules: bool,
    provided_exports: bool,
    collect_modules_diagnostics: bool,
    module_hashes: bool,
    module_codegen: bool,
    module_runtime_requirements: bool,
  },
}

impl Incremental {
  pub fn enabled(&self) -> bool {
    matches!(self, Incremental::Enabled { .. })
  }

  pub fn make_enabled(&self) -> bool {
    matches!(self, Incremental::Enabled { make, .. } if *make)
  }

  pub fn emit_assets_enabled(&self) -> bool {
    matches!(self, Incremental::Enabled { emit_assets, .. } if *emit_assets)
  }

  pub fn infer_async_modules_enabled(&self) -> bool {
    matches!(self, Incremental::Enabled { infer_async_modules, .. } if *infer_async_modules)
  }

  pub fn provided_exports_enabled(&self) -> bool {
    matches!(self, Incremental::Enabled { provided_exports, .. } if *provided_exports)
  }

  pub fn collect_modules_diagnostics_enabled(&self) -> bool {
    matches!(self, Incremental::Enabled { collect_modules_diagnostics, .. } if *collect_modules_diagnostics)
  }

  pub fn module_hashes_enabled(&self) -> bool {
    matches!(self, Incremental::Enabled { module_hashes, .. } if *module_hashes)
  }

  pub fn module_codegen_enabled(&self) -> bool {
    matches!(self, Incremental::Enabled { module_codegen, .. } if *module_codegen)
  }

  pub fn module_runtime_requirements_enabled(&self) -> bool {
    matches!(self, Incremental::Enabled { module_runtime_requirements, .. } if *module_runtime_requirements)
  }
}
