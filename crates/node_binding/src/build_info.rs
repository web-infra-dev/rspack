#[napi]
pub struct Assets {
  pub assets: HashMap<String, CompilationAsset>,
}

#[napi]
pub struct BuildInfo {
  module_reference: WeakReference<Module>,
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
