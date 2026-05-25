use rspack_core::{
  Compilation, RuntimeCodeTemplate, RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext,
  RuntimeTemplate, impl_runtime_module,
};
use rspack_plugin_javascript::impl_plugin_for_js_plugin::chunk_has_js;
use rspack_util::json_stringify_str;

#[impl_runtime_module]
#[derive(Debug)]
pub(crate) struct EsmRegisterModuleRuntimeModule {}

impl EsmRegisterModuleRuntimeModule {
  pub(crate) fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_default(runtime_template)
  }
  pub(crate) fn runtime_id(runtime_template: &RuntimeCodeTemplate) -> String {
    format!(
      "{}.add",
      runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE)
    )
  }
}

#[async_trait::async_trait]
impl RuntimeModule for EsmRegisterModuleRuntimeModule {
  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    Ok(format!(
      "{} = function registerModules(modules) {{ Object.assign({}, modules) }}\n",
      Self::runtime_id(context.runtime_template),
      context
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::MODULE_FACTORIES),
    ))
  }
}

#[impl_runtime_module]
#[derive(Debug)]
pub(crate) struct EsmChunkLoadingRuntimeModule {}

impl EsmChunkLoadingRuntimeModule {
  pub(crate) fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_default(runtime_template)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for EsmChunkLoadingRuntimeModule {
  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    let compilation = context.compilation;
    let chunk_ukey = self.chunk.expect("should have chunk");
    let chunk = compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .expect_get(&chunk_ukey);
    let initial_chunks =
      chunk.get_all_initial_chunks(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey);

    let mut chunk_imports = compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .values()
      .filter(|chunk| !initial_chunks.contains(&chunk.ukey()))
      .filter(|chunk| chunk.id().is_some())
      .filter(|chunk| chunk_has_js(&chunk.ukey(), compilation))
      .map(|chunk| {
        let chunk_id = chunk.expect_id().as_str();
        format!(
          "{}: function() {{ return import(\"__RSPACK_ESM_CHUNK_{chunk_id}\"); }}",
          json_stringify_str(chunk_id)
        )
      })
      .collect::<Vec<_>>();
    chunk_imports.sort_unstable();

    Ok(format!(
      r#"var installedChunks = {{}};
var chunkMap = {{
{chunk_imports}
}};
{ensure_chunk} = function(chunkId) {{
	var installedChunkData = installedChunks[chunkId];
	if(installedChunkData === 0) return Promise.resolve();
	if(installedChunkData) return installedChunkData;
	var loadChunk = chunkMap[chunkId];
	if(!loadChunk) return Promise.resolve();
	var promise = loadChunk().then(function() {{
		installedChunks[chunkId] = 0;
	}}, function(error) {{
		delete installedChunks[chunkId];
		throw error;
	}});
	installedChunks[chunkId] = promise;
	return promise;
}};
"#,
      chunk_imports = chunk_imports.join(",\n"),
      ensure_chunk = context
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::ENSURE_CHUNK)
    ))
  }

  fn additional_runtime_requirements(&self, _compilation: &Compilation) -> RuntimeGlobals {
    RuntimeGlobals::REQUIRE_SCOPE
  }
}
