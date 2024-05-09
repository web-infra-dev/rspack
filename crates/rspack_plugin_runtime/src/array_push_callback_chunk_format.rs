use std::hash::Hash;
use std::sync::Arc;

use async_trait::async_trait;
use rspack_core::rspack_sources::{ConcatSource, RawSource, SourceExt};
use rspack_core::{
  ApplyContext, ChunkKind, ChunkUkey, Compilation, CompilationAdditionalChunkRuntimeRequirements,
  CompilationParams, CompilerCompilation, CompilerOptions, Plugin, PluginContext, RuntimeGlobals,
};
use rspack_error::{error, Result};
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_javascript::runtime::{render_chunk_runtime_modules, render_runtime_modules};
use rspack_plugin_javascript::{
  JavascriptModulesPluginPlugin, JsChunkHashArgs, JsPlugin, PluginJsChunkHashHookOutput,
  PluginRenderJsChunkHookOutput, RenderJsChunkArgs, RenderJsStartupArgs,
};

use super::{generate_entry_startup, update_hash_for_entry_startup};

const PLUGIN_NAME: &str = "rspack.ArrayPushCallbackChunkFormatPlugin";

#[derive(Debug, Default)]
struct ArrayPushCallbackChunkFormatJavascriptModulesPluginPlugin;

#[async_trait]
impl JavascriptModulesPluginPlugin for ArrayPushCallbackChunkFormatJavascriptModulesPluginPlugin {
  fn js_chunk_hash(&self, args: &mut JsChunkHashArgs) -> PluginJsChunkHashHookOutput {
    if args
      .chunk()
      .has_runtime(&args.compilation.chunk_group_by_ukey)
    {
      return Ok(());
    }

    PLUGIN_NAME.hash(&mut args.hasher);
    let output = &args.compilation.options.output;
    output.global_object.hash(&mut args.hasher);
    output.chunk_loading_global.hash(&mut args.hasher);
    output.hot_update_global.hash(&mut args.hasher);

    update_hash_for_entry_startup(
      args.hasher,
      args.compilation,
      args
        .compilation
        .chunk_graph
        .get_chunk_entry_modules_with_chunk_group_iterable(args.chunk_ukey),
      args.chunk_ukey,
    );

    Ok(())
  }

  async fn render_chunk(&self, args: &RenderJsChunkArgs) -> PluginRenderJsChunkHookOutput {
    let drive = JsPlugin::get_compilation_drives(args.compilation);
    let chunk = args.chunk();
    let has_runtime_modules = args
      .compilation
      .chunk_graph
      .has_chunk_runtime_modules(args.chunk_ukey);
    let global_object = &args.compilation.options.output.global_object;
    let hot_update_global = &args.compilation.options.output.hot_update_global;
    let mut source = ConcatSource::default();

    if matches!(chunk.kind, ChunkKind::HotUpdate) {
      source.add(RawSource::Source(format!(
        "{}[{}]('{}', ",
        global_object,
        serde_json::to_string(hot_update_global).map_err(|e| error!(e.to_string()))?,
        chunk.expect_id()
      )));
      source.add(args.module_source.clone());
      if has_runtime_modules {
        source.add(RawSource::Source(",".to_string()));
        source.add(render_chunk_runtime_modules(
          args.compilation,
          args.chunk_ukey,
        )?);
      }
      source.add(RawSource::Source(")".to_string()));
    } else {
      let chunk_loading_global = &args.compilation.options.output.chunk_loading_global;

      source.add(RawSource::from(format!(
        r#"({}['{}'] = {}['{}'] || []).push([["{}"], "#,
        global_object,
        chunk_loading_global,
        global_object,
        chunk_loading_global,
        chunk.expect_id(),
      )));
      source.add(args.module_source.clone());
      let has_entry = chunk.has_entry_module(&args.compilation.chunk_graph);
      if has_entry || has_runtime_modules {
        source.add(RawSource::from(","));
        source.add(RawSource::from(format!(
          "function({}) {{\n",
          RuntimeGlobals::REQUIRE
        )));
        if has_runtime_modules {
          source.add(render_runtime_modules(args.compilation, args.chunk_ukey)?);
        }
        if has_entry {
          let entries = args
            .compilation
            .chunk_graph
            .get_chunk_entry_modules_with_chunk_group_iterable(args.chunk_ukey);
          let start_up_source =
            generate_entry_startup(args.compilation, args.chunk_ukey, entries, true);
          let last_entry_module = entries
            .keys()
            .last()
            .expect("should have last entry module");
          if let Some(s) = drive.render_startup(RenderJsStartupArgs {
            compilation: args.compilation,
            chunk: &chunk.ukey,
            module: *last_entry_module,
            source: start_up_source,
          })? {
            source.add(s);
          }
          let runtime_requirements = args
            .compilation
            .chunk_graph
            .get_tree_runtime_requirements(args.chunk_ukey);
          if runtime_requirements.contains(RuntimeGlobals::RETURN_EXPORTS_FROM_RUNTIME) {
            source.add(RawSource::from("return __webpack_exports__;\n"));
          }
        }
        source.add(RawSource::from("\n}\n"));
      }
      source.add(RawSource::from("])"));
    }

    Ok(Some(source.boxed()))
  }
}

#[plugin]
#[derive(Debug, Default)]
pub struct ArrayPushCallbackChunkFormatPlugin {
  js_plugin: Arc<ArrayPushCallbackChunkFormatJavascriptModulesPluginPlugin>,
}

#[plugin_hook(CompilerCompilation for ArrayPushCallbackChunkFormatPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  let mut drive = JsPlugin::get_compilation_drives_mut(compilation);
  drive.add_plugin(self.js_plugin.clone());
  Ok(())
}

#[plugin_hook(CompilationAdditionalChunkRuntimeRequirements for ArrayPushCallbackChunkFormatPlugin)]
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
    runtime_requirements.insert(RuntimeGlobals::ON_CHUNKS_LOADED);
    runtime_requirements.insert(RuntimeGlobals::REQUIRE);
  }
  runtime_requirements.insert(RuntimeGlobals::CHUNK_CALLBACK);

  Ok(())
}

impl Plugin for ArrayPushCallbackChunkFormatPlugin {
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
