#[napi(object, object_to_js = false)]
pub struct RawEsmLibraryPlugin {
  pub preserve_modules: Option<String>,
}
