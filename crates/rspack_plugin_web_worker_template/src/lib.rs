use rspack_core::{CompilerOptions, Plugin, PluginContext};
use rspack_error::Result;

#[derive(Debug, Clone, Default)]
pub struct WebWorkerTemplatePluginOptions {}

#[derive(Debug)]
pub struct WebWorkerTemplatePlugin {
  options: WebWorkerTemplatePluginOptions,
}

impl WebWorkerTemplatePlugin {
  pub fn new(options: WebWorkerTemplatePluginOptions) -> Self {
    Self { options }
  }
}

#[async_trait::async_trait]
impl Plugin for WebWorkerTemplatePlugin {
  fn name(&self) -> &'static str {
    "WebWorkerTemplatePlugin"
  }

  fn apply(
    &self,
    ctx: PluginContext<&mut rspack_core::ApplyContext>,
    options: &mut CompilerOptions,
  ) -> Result<()> {
    Ok(())
  }
}
