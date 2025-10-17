use std::iter;

use itertools::Itertools;
use rspack_collections::Identifier;
use rspack_core::{
  ChunkUkey, Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleStage, impl_runtime_module,
};

#[impl_runtime_module]
#[derive(Debug)]
pub struct StartupChunkDependenciesRuntimeModule {
  id: Identifier,
  async_chunk_loading: bool,
  chunk: Option<ChunkUkey>,
}

impl StartupChunkDependenciesRuntimeModule {
  pub fn new(async_chunk_loading: bool) -> Self {
    Self::with_default(
      Identifier::from("webpack/runtime/startup_chunk_dependencies"),
      async_chunk_loading,
      None,
    )
  }
}

#[async_trait::async_trait]
impl RuntimeModule for StartupChunkDependenciesRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Trigger
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id.to_string(),
      include_str!("runtime/startup_chunk_dependencies.ejs").to_string(),
    )]
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    if let Some(chunk_ukey) = self.chunk {
      let mut chunk_ids = compilation
        .chunk_graph
        .get_chunk_entry_dependent_chunks_iterable(
          &chunk_ukey,
          &compilation.chunk_by_ukey,
          &compilation.chunk_group_by_ukey,
        )
        .map(|chunk_ukey| {
          compilation
            .chunk_by_ukey
            .expect_get(&chunk_ukey)
            .expect_id(&compilation.chunk_ids_artifact)
            .to_string()
        })
        .collect::<Vec<_>>();

      let body = if self.async_chunk_loading {
        if let Some(chunk) = compilation.chunk_by_ukey.get(&chunk_ukey) {
          let chunk_id = chunk
            .id(&compilation.chunk_ids_artifact)
            .expect("should have chunkId for async startup")
            .to_string();
          if !chunk_ids.iter().any(|id| id == &chunk_id) {
            chunk_ids.push(chunk_id);
          }
        }

        let chunk_ids_literal =
          serde_json::to_string(&chunk_ids).expect("Invalid chunk ids serialization");
        format!(
          r#"
var chunkIds = {chunk_ids_literal};
var promises = [];
var installFederationRuntime = __webpack_require__.federation && __webpack_require__.federation.installRuntime;
if (typeof installFederationRuntime === "function") {{
	var installResult = installFederationRuntime();
	if (installResult && typeof installResult.then === "function") {{
		promises.push(installResult);
	}}
}}
var __federation_handlers__ = __webpack_require__.f;
if (__federation_handlers__) {{
  var __federation_handler_list__ = [
    __federation_handlers__.remotes,
    __federation_handlers__.consumes
  ];
  for (var i = 0; i < __federation_handler_list__.length; i++) {{
    var handler = __federation_handler_list__[i];
    if (!handler) continue;
    for (var j = 0; j < chunkIds.length; j++) {{
      handler(chunkIds[j], promises);
    }}
  }}
}}
for (var k = 0; k < chunkIds.length; k++) {{
  var chunkId = chunkIds[k];
  var promise = {ensure_chunk}(chunkId);
  if (promise !== undefined) {{
    promises.push(promise);
  }}
}}
return Promise.all(promises).then(function() {{
	if (typeof next === "function") {{
		return next();
	}}
	return next;
}});
"#,
          ensure_chunk = RuntimeGlobals::ENSURE_CHUNK
        )
      } else {
        chunk_ids
          .iter()
          .map(|cid| format!(r#"{}("{}");"#, RuntimeGlobals::ENSURE_CHUNK, cid))
          .chain(iter::once("return next();".to_string()))
          .join("\n")
      };

      let source = compilation.runtime_template.render(
        &self.id,
        Some(serde_json::json!({
          "_body": body,
        })),
      )?;

      Ok(source)
    } else {
      unreachable!("should have chunk for StartupChunkDependenciesRuntimeModule")
    }
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }
}
