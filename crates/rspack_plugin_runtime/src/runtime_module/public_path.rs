use rspack_core::{
  has_hash_placeholder, impl_runtime_module,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, Filename, PublicPath, RuntimeModule,
};
use rspack_identifier::Identifier;

#[impl_runtime_module]
#[derive(Debug)]
pub struct PublicPathRuntimeModule {
  id: Identifier,
  public_path: Box<Filename>,
}

impl PublicPathRuntimeModule {
  pub fn new(public_path: Box<Filename>) -> Self {
    Self::with_default(Identifier::from("webpack/runtime/public_path"), public_path)
  }
}

impl RuntimeModule for PublicPathRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    Ok(
      RawSource::from(include_str!("runtime/public_path.js").replace(
        "__PUBLIC_PATH_PLACEHOLDER__",
        &PublicPath::render_filename(compilation, &self.public_path),
      ))
      .boxed(),
    )
  }

  // be cacheable only when the template does not contain a hash placeholder
  fn cacheable(&self) -> bool {
    if let Some(template) = self.public_path.template() {
      !has_hash_placeholder(template)
    } else {
      false
    }
  }
}
