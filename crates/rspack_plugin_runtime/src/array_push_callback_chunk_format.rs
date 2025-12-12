use std::hash::Hash;

use rspack_core::{
  ChunkGraph, ChunkKind, ChunkUkey, Compilation, CompilationAdditionalChunkRuntimeRequirements,
  CompilationParams, CompilerCompilation, Plugin, RuntimeGlobals, RuntimeVariable,
  rspack_sources::{ConcatSource, RawStringSource, SourceExt},
};
use rspack_error::{Result, ToStringResultToRspackResultExt};
use rspack_hash::RspackHash;
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_javascript::{
  JavascriptModulesChunkHash, JavascriptModulesRenderChunk, JsPlugin, RenderSource,
  runtime::{render_chunk_runtime_modules, render_runtime_modules},
};
use rspack_util::json_stringify;

use super::{generate_entry_startup, update_hash_for_entry_startup};

const PLUGIN_NAME: &str = "rspack.ArrayPushCallbackChunkFormatPlugin";

#[plugin]
#[derive(Debug, Default)]
pub struct ArrayPushCallbackChunkFormatPlugin;

#[plugin_hook(CompilerCompilation for ArrayPushCallbackChunkFormatPlugin)]
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

#[plugin_hook(CompilationAdditionalChunkRuntimeRequirements for ArrayPushCallbackChunkFormatPlugin)]
async fn additional_chunk_runtime_requirements(
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
    // When async federation startup is requested, STARTUP_ENTRYPOINT will be present and
    // the MF wrapper handles startup; avoid adding ON_CHUNKS_LOADED in that case.
    if !runtime_requirements.contains(RuntimeGlobals::ASYNC_FEDERATION_STARTUP) {
      runtime_requirements.insert(RuntimeGlobals::ON_CHUNKS_LOADED);
    }
    runtime_requirements.insert(RuntimeGlobals::EXPORTS);
    runtime_requirements.insert(RuntimeGlobals::REQUIRE);
  }
  runtime_requirements.insert(RuntimeGlobals::CHUNK_CALLBACK);

  Ok(())
}

#[plugin_hook(JavascriptModulesChunkHash for ArrayPushCallbackChunkFormatPlugin)]
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
  let output = &compilation.options.output;
  output.global_object.hash(hasher);
  output.chunk_loading_global.hash(hasher);
  output.hot_update_global.hash(hasher);

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

#[plugin_hook(JavascriptModulesRenderChunk for ArrayPushCallbackChunkFormatPlugin)]
async fn render_chunk(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  render_source: &mut RenderSource,
) -> Result<()> {
  let hooks = JsPlugin::get_compilation_hooks(compilation.id());
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
  let has_runtime_modules = compilation
    .chunk_graph
    .has_chunk_runtime_modules(chunk_ukey);
  let global_object = &compilation.options.output.global_object;
  let hot_update_global = &compilation.options.output.hot_update_global;
  let mut source = ConcatSource::default();

  if matches!(chunk.kind(), ChunkKind::HotUpdate) {
    source.add(RawStringSource::from(format!(
      "{}[{}]({}, ",
      global_object,
      serde_json::to_string(hot_update_global).to_rspack_result()?,
      json_stringify(chunk.expect_id())
    )));
    source.add(render_source.source.clone());
    if has_runtime_modules {
      source.add(RawStringSource::from_static(","));
      source.add(render_chunk_runtime_modules(compilation, chunk_ukey).await?);
    }
    source.add(RawStringSource::from_static(")"));
  } else {
    let chunk_loading_global = &compilation.options.output.chunk_loading_global;

    source.add(RawStringSource::from(format!(
      r#"({}["{}"] = {}["{}"] || []).push([[{}], "#,
      global_object,
      chunk_loading_global,
      global_object,
      chunk_loading_global,
      serde_json::to_string(chunk.expect_id()).expect("json stringify failed"),
    )));
    source.add(render_source.source.clone());
    let has_entry = chunk.has_entry_module(&compilation.chunk_graph);
    if has_entry || has_runtime_modules {
      source.add(RawStringSource::from_static(","));
      source.add(RawStringSource::from(format!(
        "function({}) {{\n",
        compilation
          .runtime_template
          .render_runtime_globals(&RuntimeGlobals::REQUIRE)
      )));
      if has_runtime_modules {
        source.add(render_runtime_modules(compilation, chunk_ukey).await?);
      }
      if has_entry {
        let entries = compilation
          .chunk_graph
          .get_chunk_entry_modules_with_chunk_group_iterable(chunk_ukey);
        let runtime_requirements =
          ChunkGraph::get_tree_runtime_requirements(compilation, chunk_ukey);
        let passive = !runtime_requirements.contains(RuntimeGlobals::STARTUP_ENTRYPOINT);
        let start_up_source = generate_entry_startup(compilation, chunk_ukey, entries, passive);
        let last_entry_module = entries
          .keys()
          .next_back()
          .expect("should have last entry module");
        let mut render_source = RenderSource {
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
            &mut render_source,
          )
          .await?;
        source.add(render_source.source);
        if runtime_requirements.contains(RuntimeGlobals::RETURN_EXPORTS_FROM_RUNTIME) {
          source.add(RawStringSource::from(format!(
            "return {};\n",
            compilation
              .runtime_template
              .render_runtime_variable(&RuntimeVariable::Exports)
          )));
        }
      }
      source.add(RawStringSource::from_static("\n}\n"));
    }
    source.add(RawStringSource::from_static("])"));
  }
  render_source.source = source.boxed();
  Ok(())
}

impl Plugin for ArrayPushCallbackChunkFormatPlugin {
  fn name(&self) -> &'static str {
    PLUGIN_NAME
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.compilation.tap(compilation::new(self));
    ctx
      .compilation_hooks
      .additional_chunk_runtime_requirements
      .tap(additional_chunk_runtime_requirements::new(self));
    Ok(())
  }
}
