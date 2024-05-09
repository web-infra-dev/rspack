use async_trait::async_trait;
use rspack_core::{
  compile_boolean_matcher, impl_runtime_module,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  ApplyContext, BooleanMatcher, Chunk, ChunkUkey, Compilation,
  CompilationAdditionalTreeRuntimeRequirements, CompilerOptions, Plugin, PluginContext,
  RuntimeGlobals, RuntimeModule, RuntimeModuleStage,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_identifier::Identifier;
use rspack_plugin_runtime::chunk_has_js;
use rspack_util::source_map::SourceMapKind;
#[impl_runtime_module]
#[derive(Debug, Eq)]
pub struct FederationRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
}

impl Default for FederationRuntimeModule {
  fn default() -> Self {
    Self {
      id: Identifier::from("module_federation/runtime"),
      chunk: None,
      source_map_kind: SourceMapKind::empty(),
      custom_source: None,
    }
  }
}

impl RuntimeModule for FederationRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Normal
  }

  fn generate(&self, compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    let chunk = compilation
      .chunk_by_ukey
      .expect_get(&self.chunk.expect("The chunk should be attached."));
    Ok(RawSource::from(federation_runtime_template(chunk, compilation)).boxed())
  }
}

fn federation_runtime_template(chunk: &Chunk, compilation: &Compilation) -> String {
  let federation_global = format!("{}.federation", RuntimeGlobals::REQUIRE);

  let condition_map =
    compilation
      .chunk_graph
      .get_chunk_condition_map(&chunk.ukey, compilation, chunk_has_js);
  let has_js_matcher = compile_boolean_matcher(&condition_map);

  let chunk_matcher = if matches!(has_js_matcher, BooleanMatcher::Condition(false)) {
    String::from("")
  } else {
    format!(
      r#"
chunkMatcher: function(chunkId) {{
    return {has_js_matcher};
}}
"#,
      has_js_matcher = &has_js_matcher.render("chunkId")
    )
  };

  format!(
    r#"
if(!{federation_global}){{
    {federation_global} = {{
        {chunk_matcher}
    }};
}}
"#,
    federation_global = federation_global,
    chunk_matcher = chunk_matcher
  )
}

#[plugin]
#[derive(Debug, Default)]
pub struct ModuleFederationRuntimePlugin;

#[plugin_hook(CompilationAdditionalTreeRuntimeRequirements for ModuleFederationRuntimePlugin)]
fn additional_tree_runtime_requirements(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  _runtime_requirements: &mut RuntimeGlobals,
) -> Result<()> {
  compilation.add_runtime_module(chunk_ukey, Box::<FederationRuntimeModule>::default())?;
  Ok(())
}

#[async_trait]
impl Plugin for ModuleFederationRuntimePlugin {
  fn name(&self) -> &'static str {
    "rspack.container.ModuleFederationRuntimePlugin"
  }

  fn apply(
    &self,
    ctx: PluginContext<&mut ApplyContext>,
    _options: &mut CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .compilation_hooks
      .additional_tree_runtime_requirements
      .tap(additional_tree_runtime_requirements::new(self));
    Ok(())
  }
}
