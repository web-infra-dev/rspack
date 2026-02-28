use rspack_core::{
  Filename, PublicPath, RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext,
  RuntimeTemplate, has_hash_placeholder, impl_runtime_module,
};

#[impl_runtime_module]
#[derive(Debug)]
pub struct PublicPathRuntimeModule {
  public_path: Box<Filename>,
}

impl PublicPathRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate, public_path: Box<Filename>) -> Self {
    Self::with_default(runtime_template, public_path)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for PublicPathRuntimeModule {
  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    let compilation = context.compilation;
    Ok(format!(
      "{} = \"{}\";",
      context
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::PUBLIC_PATH),
      &PublicPath::render_filename(compilation, &self.public_path).await,
    ))
  }

  // be cacheable only when the template does not contain a hash placeholder
  fn full_hash(&self) -> bool {
    if let Some(template) = self.public_path.template() {
      has_hash_placeholder(template)
    } else {
      true
    }
  }
}
