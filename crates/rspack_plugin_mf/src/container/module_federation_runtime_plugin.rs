//! # ModuleFederationRuntimePlugin
//!
//! Main orchestration plugin for Module Federation runtime functionality.
//! Coordinates federation plugins, manages runtime dependencies, and adds the base FederationRuntimeModule.

use async_trait::async_trait;
use rspack_core::{
  chunk_has_js, compile_boolean_matcher, get_js_chunk_filename_template, get_undo_path,
  ApplyContext, BooleanMatcher, BoxDependency, Chunk, ChunkUkey, Compilation,
  CompilationAdditionalTreeRuntimeRequirements, CompilerFinishMake, CompilerOptions, EntryOptions,
  Identifier, Plugin, PluginContext, RuntimeGlobals, RuntimeModule, RuntimeModuleStage,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use serde::Deserialize;

use super::{
  embed_federation_runtime_plugin::EmbedFederationRuntimePlugin,
  federation_data_runtime_module::FederationDataRuntimeModule,
  federation_modules_plugin::FederationModulesPlugin,
  federation_runtime_dependency::FederationRuntimeDependency,
  hoist_container_references_plugin::HoistContainerReferencesPlugin,
};

#[derive(Debug, Default, Deserialize, Clone)]
pub struct ModuleFederationRuntimePluginOptions {
  pub entry_runtime: Option<String>,
}

#[derive(Debug)]
pub struct FederationRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
}

impl FederationRuntimeModule {
  pub fn with_default(id: Identifier, chunk: Option<ChunkUkey>) -> Self {
    Self { id, chunk }
  }
}

impl Default for FederationRuntimeModule {
  fn default() -> Self {
    Self::with_default(Identifier::from("module_federation/runtime"), None)
  }
}

#[async_trait]
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

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    let chunk = compilation
      .chunk_by_ukey
      .expect_get(&self.chunk.expect("The chunk should be attached."));
    Ok(federation_runtime_template(chunk, compilation).await)
  }
}

async fn federation_runtime_template(chunk: &Chunk, compilation: &Compilation) -> String {
  let federation_global = format!("{}.federation", RuntimeGlobals::REQUIRE);

  let condition_map =
    compilation
      .chunk_graph
      .get_chunk_condition_map(&chunk.ukey(), compilation, chunk_has_js);
  let has_js_matcher = compile_boolean_matcher(&condition_map);

  let chunk_matcher = if matches!(has_js_matcher, BooleanMatcher::Condition(false)) {
    String::from("")
  } else {
    format!(
      r#"
chunkMatcher: function(chunkId) {{
    return {has_js_matcher};
}},
"#,
      has_js_matcher = &has_js_matcher.render("chunkId")
    )
  };

  // Calculate rootOutputDir similar to webpack
  let root_output_dir = {
    let filename = get_js_chunk_filename_template(
      chunk,
      &compilation.options.output,
      &compilation.chunk_group_by_ukey,
    );
    let output_name = compilation
      .get_path(&filename, Default::default())
      .await
      .expect("failed to get output path");
    get_undo_path(
      &output_name,
      compilation.options.output.path.to_string(),
      false,
    )
  };

  let root_output_dir_str = format!(
    r#"rootOutputDir: "{root_output_dir}",
"#
  );

  format!(
    r#"
if(!{federation_global}){{
    {federation_global} = {{
        {chunk_matcher}{root_output_dir_str}
    }};
}}
"#
  )
}

#[plugin]
#[derive(Debug)]
pub struct ModuleFederationRuntimePlugin {
  options: ModuleFederationRuntimePluginOptions,
}

impl ModuleFederationRuntimePlugin {
  pub fn new(options: ModuleFederationRuntimePluginOptions) -> Self {
    Self::new_inner(options)
  }
}

#[plugin_hook(CompilationAdditionalTreeRuntimeRequirements for ModuleFederationRuntimePlugin)]
async fn additional_tree_runtime_requirements(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  _runtime_requirements: &mut RuntimeGlobals,
) -> Result<()> {
  // Add base FederationRuntimeModule which is responsible for providing bundler data to the runtime.
  compilation.add_runtime_module(chunk_ukey, Box::<FederationDataRuntimeModule>::default())?;

  Ok(())
}

#[plugin_hook(CompilerFinishMake for ModuleFederationRuntimePlugin)]
async fn finish_make(&self, compilation: &mut Compilation) -> Result<()> {
  if let Some(entry_request) = self.options.entry_runtime.clone() {
    let federation_runtime_dep = FederationRuntimeDependency::new(entry_request.clone());

    let hooks = FederationModulesPlugin::get_compilation_hooks(compilation);

    hooks
      .add_federation_runtime_dependency
      .lock()
      .await
      .call(&federation_runtime_dep)
      .await?;

    let boxed_dep: BoxDependency = Box::new(federation_runtime_dep);
    let entry_options = EntryOptions::default();
    let args = vec![(boxed_dep, entry_options)];

    compilation.add_include(args).await?;
  }

  Ok(())
}

#[async_trait]
impl Plugin for ModuleFederationRuntimePlugin {
  fn name(&self) -> &'static str {
    "rspack.container.ModuleFederationRuntimePlugin"
  }

  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, options: &CompilerOptions) -> Result<()> {
    ctx
      .context
      .compilation_hooks
      .additional_tree_runtime_requirements
      .tap(additional_tree_runtime_requirements::new(self));

    ctx
      .context
      .compiler_hooks
      .finish_make
      .tap(finish_make::new(self));

    // Apply supporting plugins
    EmbedFederationRuntimePlugin::default()
      .apply(PluginContext::with_context(ctx.context), options)?;
    HoistContainerReferencesPlugin::default()
      .apply(PluginContext::with_context(ctx.context), options)?;

    Ok(())
  }
}
