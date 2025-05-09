use rspack_core::WeakBindingCell;

#[napi]
pub struct Assets {
  assets: WeakBindingCell<HashMap<String, CompilationAsset>>,
}

#[napi]
pub struct BuildInfo {
  module_reference: WeakBindingCell<rspack_core::BuildInfo>,
}

impl BuildInfo {
  pub fn new(module_reference: WeakReference<Module>) -> Self {
    Self { module_reference }
  }
}

#[napi]
impl BuildInfo {
  #[napi(getter)]
  pub fn assets(&self, env: &Env) -> napi::Result<Assets> {}
}
