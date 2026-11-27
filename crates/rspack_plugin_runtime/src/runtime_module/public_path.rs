use rspack_core::{
  Compilation, Filename, PublicPath, RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext,
  RuntimeTemplate, impl_runtime_module,
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
  fn runtime_module_variables() -> &'static [&'static str] {
    &[]
  }

  fn runtime_requirements(
    &self,
    _compilation: &Compilation,
  ) -> rspack_core::RuntimeModuleRuntimeRequirements {
    rspack_core::RuntimeModuleRuntimeRequirements {
      define: { RuntimeGlobals::PUBLIC_PATH },
      force_context: RuntimeGlobals::PUBLIC_PATH,
      ..Default::default()
    }
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id().to_string(),
      include_str!("runtime/public_path.ejs").to_string(),
    )]
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    let public_path = PublicPath::render_filename(context.compilation, &self.public_path).await;
    context.runtime_template.render_preserving_line_endings(
      self.id(),
      Some(serde_json::json!({ "_public_path": public_path })),
    )
  }

  // be cacheable only when the template does not contain a hash placeholder
  fn full_hash(&self) -> bool {
    self.public_path.has_hash_placeholder()
  }
}
