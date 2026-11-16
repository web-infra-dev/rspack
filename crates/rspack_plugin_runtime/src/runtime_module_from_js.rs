use std::sync::Arc;

use derive_more::Debug;
use futures::future::BoxFuture;
use rspack_cacheable::with::Unsupported;
use rspack_core::{
  Compilation, RuntimeModule, RuntimeModuleGenerateContext, RuntimeModuleStage,
  impl_runtime_module, runtime_mode::RuntimeMode,
};

type GenerateFn = Arc<dyn Fn() -> BoxFuture<'static, rspack_error::Result<String>> + Send + Sync>;

#[impl_runtime_module]
#[derive(Debug)]
pub struct RuntimeModuleFromJs {
  #[debug(skip)]
  #[cacheable(with=Unsupported)]
  pub generator: GenerateFn,
  pub full_hash: bool,
  pub dependent_hash: bool,
  pub isolate: bool,
  pub stage: RuntimeModuleStage,
}

#[async_trait::async_trait]
impl RuntimeModule for RuntimeModuleFromJs {
  fn is_custom(&self) -> bool {
    true
  }

  async fn prepare_custom_source(&mut self, compilation: &Compilation) -> rspack_error::Result<()> {
    if self.common.get_custom_source().source.is_some() {
      return Ok(());
    }
    let graph = &compilation.build_chunk_graph_artifact;
    let chunk = graph.chunk_by_ukey.expect_get(
      &self
        .common
        .chunk()
        .expect("runtime module should be attached"),
    );
    let mode = compilation.runtime_template.render_mode();
    self.common.render_mode = if chunk.has_runtime(&graph.chunk_group_by_ukey) {
      mode.runtime_module_render_mode()
    } else {
      mode.chunk_render_mode()
    };
    let source = (self.generator)().await?;
    self.common.set_generated_source(source)
  }

  fn runtime_module_variables() -> &'static [&'static str] {
    &[]
  }

  async fn generate(&self, _: &RuntimeModuleGenerateContext<'_>) -> rspack_error::Result<String> {
    let res = (self.generator)().await?;
    Ok(res)
  }

  fn full_hash(&self) -> bool {
    self.full_hash
  }

  fn dependent_hash(&self) -> bool {
    self.dependent_hash
  }

  fn should_isolate(&self, _runtime_mode: RuntimeMode) -> bool {
    self.isolate
  }

  fn stage(&self) -> RuntimeModuleStage {
    self.stage.clone()
  }
}
