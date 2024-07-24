use std::hash::Hash;

use rspack_core::rspack_sources::{ConcatSource, RawSource, SourceExt};
use rspack_core::{
  ApplyContext, ChunkUkey, Compilation, CompilationAdditionalChunkRuntimeRequirements,
  CompilationParams, CompilerCompilation, CompilerOptions, Plugin, PluginContext, RuntimeGlobals,
};
use rspack_error::Result;
use rspack_hash::RspackHash;
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_javascript::runtime::render_chunk_runtime_modules;
use rspack_plugin_javascript::{
  JavascriptModulesChunkHash, JavascriptModulesRenderChunk, JsPlugin, RenderSource,
};
use rspack_util::json_stringify;

use crate::{
  generate_entry_startup, get_chunk_output_name, get_relative_path, get_runtime_chunk_output_name,
  update_hash_for_entry_startup,
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
  let mut hooks = JsPlugin::get_compilation_hooks_mut(compilation);
  hooks.chunk_hash.tap(js_chunk_hash::new(self));
  hooks.render_chunk.tap(render_chunk::new(self));
  Ok(())
}

#[plugin_hook(CompilationAdditionalChunkRuntimeRequirements for CommonJsChunkFormatPlugin)]
fn additional_chunk_runtime_requirements(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &mut RuntimeGlobals,
) -> Result<()> {
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);

  if chunk.has_runtime(&compilation.chunk_group_by_ukey) {
    return Ok(());
  }

  if compilation
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
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
  if chunk.has_runtime(&compilation.chunk_group_by_ukey) {
    return Ok(());
  }

  PLUGIN_NAME.hash(hasher);

  update_hash_for_entry_startup(
    hasher,
    compilation,
    compilation
      .chunk_graph
      .get_chunk_entry_modules_with_chunk_group_iterable(chunk_ukey),
    chunk_ukey,
  );

  Ok(())
}

#[plugin_hook(JavascriptModulesRenderChunk for CommonJsChunkFormatPlugin)]
fn render_chunk(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  render_source: &mut RenderSource,
) -> Result<()> {
  let hooks = JsPlugin::get_compilation_hooks(compilation);
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
  let base_chunk_output_name = get_chunk_output_name(chunk, compilation)?;
  let mut sources = ConcatSource::default();
  sources.add(RawSource::from(format!(
    "exports.ids = ['{}'];\n",
    &chunk.expect_id().to_string()
  )));
  sources.add(RawSource::from("exports.modules = "));
  sources.add(render_source.source.clone());
  sources.add(RawSource::from(";\n"));
  if compilation
    .chunk_graph
    .has_chunk_runtime_modules(chunk_ukey)
  {
    sources.add(RawSource::from("exports.runtime = "));
    sources.add(render_chunk_runtime_modules(compilation, chunk_ukey)?);
    sources.add(RawSource::from(";\n"));
  }

  if chunk.has_entry_module(&compilation.chunk_graph) {
    let runtime_chunk_output_name = get_runtime_chunk_output_name(compilation, chunk_ukey)?;
    sources.add(RawSource::from(format!(
      "// load runtime\nvar {} = require({});\n",
      RuntimeGlobals::REQUIRE,
      json_stringify(&get_relative_path(
        &base_chunk_output_name,
        &runtime_chunk_output_name
      ))
    )));
    sources.add(RawSource::from(format!(
      "{}(exports)\n",
      RuntimeGlobals::EXTERNAL_INSTALL_CHUNK,
    )));

    let entries = compilation
      .chunk_graph
      .get_chunk_entry_modules_with_chunk_group_iterable(chunk_ukey);
    let start_up_source = generate_entry_startup(compilation, chunk_ukey, entries, false);
    let last_entry_module = entries
      .keys()
      .last()
      .expect("should have last entry module");
    let mut startup_render_source = RenderSource {
      source: start_up_source,
    };
    hooks.render_startup.call(
      compilation,
      chunk_ukey,
      last_entry_module,
      &mut startup_render_source,
    )?;
    sources.add(startup_render_source.source);
    render_source.source = ConcatSource::new([
      RawSource::from("(function() {\n").boxed(),
      sources.boxed(),
      RawSource::from("\n})()").boxed(),
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

  fn apply(
    &self,
    ctx: PluginContext<&mut ApplyContext>,
    _options: &mut CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .compiler_hooks
      .compilation
      .tap(compilation::new(self));
    ctx
      .context
      .compilation_hooks
      .additional_chunk_runtime_requirements
      .tap(additional_chunk_runtime_requirements::new(self));
    Ok(())
  }
}
