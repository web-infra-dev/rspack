use rspack_collections::Identifier;
use rspack_core::{ChunkUkey, Compilation, RuntimeGlobals, RuntimeModule, impl_runtime_module};

use crate::get_chunk_runtime_requirements;

#[impl_runtime_module]
#[derive(Debug)]
pub struct MakeDeferredNamespaceObjectRuntimeModule {
  id: Identifier,
  chunk_ukey: ChunkUkey,
}

impl MakeDeferredNamespaceObjectRuntimeModule {
  pub fn new(chunk_ukey: ChunkUkey) -> Self {
    Self::with_default(
      Identifier::from("webpack/runtime/make_deferred_namespace_object"),
      chunk_ukey,
    )
  }
}

#[async_trait::async_trait]
impl RuntimeModule for MakeDeferredNamespaceObjectRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id.to_string(),
      include_str!("runtime/make_deferred_namespace_object.ejs").to_string(),
    )]
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    let has_async = get_chunk_runtime_requirements(compilation, &self.chunk_ukey)
      .contains(RuntimeGlobals::ASYNC_MODULE);
    let get_async_module_export_str = if has_async {
      format!(
        "if ({es} in ns) ns = ns[{es}];",
        es = RuntimeGlobals::ASYNC_MODULE_EXPORT_SYMBOL,
      )
    } else {
      String::new()
    };
    let cached_get_async_module_export_str = if has_async {
      format!(
        "if ({es} in exports) exports = exports[{es}];",
        es = RuntimeGlobals::ASYNC_MODULE_EXPORT_SYMBOL,
      )
    } else {
      String::new()
    };
    let source = compilation.runtime_template.render(
      &self.id,
      Some(serde_json::json!({
        "get_async_module_export": get_async_module_export_str,
        "cached_get_async_module_export": cached_get_async_module_export_str,
      })),
    )?;

    Ok(source)
  }
}
