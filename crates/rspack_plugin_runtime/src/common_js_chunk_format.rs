use std::hash::Hash;

use rspack_core::{
  ChunkUkey, Compilation, CompilationAdditionalChunkRuntimeRequirements,
  CompilationDependentFullHash, CompilationParams, CompilerCompilation, Plugin, RuntimeGlobals,
  RuntimeModule,
  rspack_sources::{ConcatSource, RawStringSource, SourceExt},
};
use rspack_error::Result;
use rspack_hash::RspackHash;
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_javascript::{
  JavascriptModulesChunkHash, JavascriptModulesRenderChunk, JsPlugin, RenderSource,
  runtime::render_chunk_runtime_modules,
};
use rspack_util::json_stringify;

use crate::{
  generate_entry_startup, get_chunk_output_name, get_relative_path, get_runtime_chunk_output_name,
  runtime_chunk_has_hash, update_hash_for_entry_startup,
};

const PLUGIN_NAME: &str = "rspack.CommonJsChunkFormatPlugin";

#[plugin]
#[derive(Debug, Default)]
pub struct CommonJsChunkFormatPlugin;

#[plugin_hook(CompilerCompilation for CommonJsChunkFormatPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  let hooks = JsPlugin::get_compilation_hooks_mut(compilation.id());
  let mut hooks = hooks.write().await;
  hooks.chunk_hash.tap(js_chunk_hash::new(self));
  hooks.render_chunk.tap(render_chunk::new(self));
  Ok(())
}

#[plugin_hook(CompilationAdditionalChunkRuntimeRequirements for CommonJsChunkFormatPlugin)]
async fn additional_chunk_runtime_requirements(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &mut RuntimeGlobals,
  _runtime_modules: &mut Vec<Box<dyn RuntimeModule>>,
) -> Result<()> {
  let chunk = compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .expect_get(chunk_ukey);

  if chunk.has_runtime(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey) {
    return Ok(());
  }

  if compilation
    .build_chunk_graph_artifact
    .chunk_graph
    .get_number_of_entry_modules(chunk_ukey)
    > 0
  {
    runtime_requirements.insert(RuntimeGlobals::REQUIRE);
    runtime_requirements.insert(RuntimeGlobals::STARTUP_ENTRYPOINT);
    runtime_requirements.insert(RuntimeGlobals::EXTERNAL_INSTALL_CHUNK);
  }

  Ok(())
}

#[plugin_hook(JavascriptModulesChunkHash for CommonJsChunkFormatPlugin)]
async fn js_chunk_hash(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  hasher: &mut RspackHash,
) -> Result<()> {
  let chunk = compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .expect_get(chunk_ukey);
  if chunk.has_runtime(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey) {
    return Ok(());
  }

  PLUGIN_NAME.hash(hasher);

  update_hash_for_entry_startup(
    hasher,
    compilation,
    compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .get_chunk_entry_modules_with_chunk_group_iterable(chunk_ukey),
    chunk_ukey,
  );

  Ok(())
}

#[plugin_hook(CompilationDependentFullHash for CommonJsChunkFormatPlugin)]
async fn compilation_dependent_full_hash(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
) -> Result<Option<bool>> {
  let chunk = compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .expect_get(chunk_ukey);
  if chunk.has_entry_module(&compilation.build_chunk_graph_artifact.chunk_graph)
    && runtime_chunk_has_hash(compilation, chunk_ukey).await?
  {
    return Ok(Some(true));
  }
  Ok(None)
}

#[plugin_hook(JavascriptModulesRenderChunk for CommonJsChunkFormatPlugin)]
async fn render_chunk(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  render_source: &mut RenderSource,
) -> Result<()> {
  let hooks = JsPlugin::get_compilation_hooks(compilation.id());
  let chunk = compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .expect_get(chunk_ukey);
  let base_chunk_output_name = get_chunk_output_name(chunk, compilation).await?;
  let mut sources = ConcatSource::default();
  sources.add(RawStringSource::from(format!(
    "exports.ids = [{}];\n",
    json_stringify(chunk.expect_id())
  )));
  sources.add(RawStringSource::from_static("exports.modules = "));
  sources.add(render_source.source.clone());
  sources.add(RawStringSource::from_static(";\n"));
  if compilation
    .build_chunk_graph_artifact
    .chunk_graph
    .has_chunk_runtime_modules(chunk_ukey)
  {
    sources.add(RawStringSource::from_static("exports.runtime = "));
    sources.add(render_chunk_runtime_modules(compilation, chunk_ukey).await?);
    sources.add(RawStringSource::from_static(";\n"));
  }

  if chunk.has_entry_module(&compilation.build_chunk_graph_artifact.chunk_graph) {
    let runtime_chunk_output_name = get_runtime_chunk_output_name(compilation, chunk_ukey).await?;
    sources.add(RawStringSource::from(format!(
      r#"// load runtime
var {} = require({});
"#,
      compilation
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::REQUIRE),
      json_stringify(&get_relative_path(
        base_chunk_output_name
          .trim_start_matches("/")
          .trim_start_matches("\\"),
        &runtime_chunk_output_name
      ))
    )));
    sources.add(RawStringSource::from(format!(
      "{}(exports)\n",
      compilation
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::EXTERNAL_INSTALL_CHUNK),
    )));

    let entries = compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .get_chunk_entry_modules_with_chunk_group_iterable(chunk_ukey);
    let start_up_source = generate_entry_startup(compilation, chunk_ukey, entries, false);
    let last_entry_module = entries
      .keys()
      .next_back()
      .expect("should have last entry module");
    let mut startup_render_source = RenderSource {
      source: start_up_source,
    };
    hooks
      .try_read()
      .expect("should have js plugin drive")
      .render_startup
      .call(
        compilation,
        chunk_ukey,
        last_entry_module,
        &mut startup_render_source,
      )
      .await?;
    sources.add(startup_render_source.source);
    render_source.source = ConcatSource::new([
      RawStringSource::from_static("(function() {\n").boxed(),
      sources.boxed(),
      RawStringSource::from_static("\n})()").boxed(),
    ])
    .boxed();
    return Ok(());
  }
  render_source.source = sources.boxed();
  Ok(())
}

impl Plugin for CommonJsChunkFormatPlugin {
  fn name(&self) -> &'static str {
    PLUGIN_NAME
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.compilation.tap(compilation::new(self));
    ctx
      .compilation_hooks
      .additional_chunk_runtime_requirements
      .tap(additional_chunk_runtime_requirements::new(self));
    ctx
      .compilation_hooks
      .dependent_full_hash
      .tap(compilation_dependent_full_hash::new(self));
    Ok(())
  }
}
