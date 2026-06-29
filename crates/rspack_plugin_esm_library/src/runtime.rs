use rspack_core::{
  Compilation, RuntimeCodeTemplate, RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext,
  RuntimeModuleStage, RuntimeTemplate, RuntimeVariable, impl_runtime_module, property_access,
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
    if runtime_template.uses_runtime_context() {
      let modules_key = RuntimeGlobals::MODULE_FACTORIES
        .property_name()
        .expect("module factories should have a property name");
      return format!(
        "{}{}.add",
        runtime_template.render_runtime_variable(&RuntimeVariable::Context),
        property_access([modules_key], 0)
      );
    }

    format!(
      "{}.add",
      runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE)
    )
  }
}

#[async_trait::async_trait]
impl RuntimeModule for EsmRegisterModuleRuntimeModule {
  fn runtime_requirements(
    &self,
    _compilation: &Compilation,
  ) -> rspack_core::RuntimeModuleRuntimeRequirements {
    rspack_core::RuntimeModuleRuntimeRequirements {
      dependencies: RuntimeGlobals::MODULE_FACTORIES | RuntimeGlobals::REQUIRE,
      ..Default::default()
    }
  }

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
pub(crate) struct EsmEnsureChunkRuntimeModule {}

impl EsmEnsureChunkRuntimeModule {
  pub(crate) fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_default(runtime_template)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for EsmEnsureChunkRuntimeModule {
  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    Ok(format!(
      r#"{ensure_chunk_handlers} = {{}};
{ensure_chunk} = function(chunkId, fetchPriority) {{
	return Promise.all(Object.keys({ensure_chunk_handlers}).reduce(function(promises, key) {{
		{ensure_chunk_handlers}[key](chunkId, promises, fetchPriority);
		return promises;
	}}, []));
}};
"#,
      ensure_chunk = context
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::ENSURE_CHUNK),
      ensure_chunk_handlers = context
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::ENSURE_CHUNK_HANDLERS)
    ))
  }
  fn runtime_requirements(
    &self,
    _compilation: &Compilation,
  ) -> rspack_core::RuntimeModuleRuntimeRequirements {
    rspack_core::RuntimeModuleRuntimeRequirements {
      dependencies: RuntimeGlobals::REQUIRE_SCOPE | RuntimeGlobals::ENSURE_CHUNK_HANDLERS,
      write: { RuntimeGlobals::ENSURE_CHUNK | RuntimeGlobals::ENSURE_CHUNK_HANDLERS },
      ..Default::default()
    }
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
    let runtime = chunk.runtime().clone();
    let initial_chunks =
      chunk.get_all_initial_chunks(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey);
    let async_chunks =
      chunk.get_all_async_chunks(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey);

    let mut chunk_imports = async_chunks
      .iter()
      .filter(|chunk_ukey| !initial_chunks.contains(*chunk_ukey))
      .map(|chunk_ukey| {
        compilation
          .build_chunk_graph_artifact
          .chunk_by_ukey
          .expect_get(chunk_ukey)
      })
      .filter(|chunk| !chunk.runtime().is_disjoint(&runtime))
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
{ensure_chunk_handlers}.j = function(chunkId, promises) {{
	var installedChunkData = installedChunks[chunkId];
	if(installedChunkData === 0) return;
	if(installedChunkData) {{
		promises.push(installedChunkData);
		return;
	}}
	var loadChunk = chunkMap[chunkId];
	if(!loadChunk) return;
	var promise = loadChunk().then(function() {{
		installedChunks[chunkId] = 0;
	}}, function(error) {{
		delete installedChunks[chunkId];
		throw error;
	}});
	installedChunks[chunkId] = promise;
	promises.push(promise);
}};
"#,
      chunk_imports = chunk_imports.join(",\n"),
      ensure_chunk_handlers = context
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::ENSURE_CHUNK_HANDLERS)
    ))
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Attach
  }

  fn runtime_requirements(
    &self,
    _compilation: &Compilation,
  ) -> rspack_core::RuntimeModuleRuntimeRequirements {
    rspack_core::RuntimeModuleRuntimeRequirements {
      dependencies: RuntimeGlobals::REQUIRE_SCOPE | RuntimeGlobals::ENSURE_CHUNK_HANDLERS,
      write: RuntimeGlobals::ENSURE_CHUNK_HANDLERS,
      ..Default::default()
    }
  }
}
