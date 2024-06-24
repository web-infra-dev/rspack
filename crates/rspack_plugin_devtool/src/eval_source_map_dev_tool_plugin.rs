use std::{borrow::Cow, hash::Hash};

use dashmap::DashMap;
use once_cell::sync::Lazy;
use rspack_core::{
  contextify,
  rspack_sources::{BoxSource, MapOptions, RawSource, Source, SourceExt},
  ApplyContext, BoxModule, ChunkInitFragments, ChunkUkey, Compilation, CompilationParams,
  CompilerCompilation, CompilerOptions, Plugin, PluginContext,
};
use rspack_error::Result;
use rspack_hash::RspackHash;
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_javascript::{
  JavascriptModulesChunkHash, JavascriptModulesInlineInRuntimeBailout,
  JavascriptModulesRenderModuleContent, JsPlugin, RenderSource,
};
use rspack_util::swc::normalize_custom_filename;
use serde_json::json;

use crate::SourceMapDevToolPluginOptions;

static MODULE_RENDER_CACHE: Lazy<DashMap<BoxSource, BoxSource>> = Lazy::new(DashMap::default);

const EVAL_SOURCE_MAP_DEV_TOOL_PLUGIN_NAME: &str = "rspack.EvalSourceMapDevToolPlugin";

#[plugin]
#[derive(Debug)]
pub struct EvalSourceMapDevToolPlugin {
  columns: bool,
  no_sources: bool,
}

impl EvalSourceMapDevToolPlugin {
  pub fn new(options: SourceMapDevToolPluginOptions) -> Self {
    Self::new_inner(options.columns, options.no_sources)
  }
}

#[plugin_hook(CompilerCompilation for EvalSourceMapDevToolPlugin)]
async fn eval_source_map_devtool_plugin_compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  let mut hooks = JsPlugin::get_compilation_hooks_mut(compilation);
  hooks
    .render_module_content
    .tap(eval_source_map_devtool_plugin_render_module_content::new(
      self,
    ));
  hooks
    .chunk_hash
    .tap(eval_source_map_devtool_plugin_js_chunk_hash::new(self));
  hooks
    .inline_in_runtime_bailout
    .tap(eval_source_map_devtool_plugin_inline_in_runtime_bailout::new(self));
  Ok(())
}

#[plugin_hook(JavascriptModulesRenderModuleContent for EvalSourceMapDevToolPlugin)]
fn eval_source_map_devtool_plugin_render_module_content(
  &self,
  compilation: &Compilation,
  _module: &BoxModule,
  render_source: &mut RenderSource,
  _init_fragments: &mut ChunkInitFragments,
) -> Result<()> {
  let origin_source = render_source.source.clone();
  if let Some(cached) = MODULE_RENDER_CACHE.get(&origin_source) {
    render_source.source = cached.value().clone();
    return Ok(());
  } else if let Some(mut map) = origin_source.map(&MapOptions::new(self.columns)) {
    let source = {
      let source = &origin_source.source();
      for source in map.sources_mut() {
        let resource_path = normalize_custom_filename(source);
        let resource_path = contextify(&compilation.options.context, resource_path);
        *source = Cow::from(resource_path);
      }
      if self.no_sources {
        for content in map.sources_content_mut() {
          *content = Cow::from(String::default());
        }
      }
      let mut map_buffer = Vec::new();
      map
        .to_writer(&mut map_buffer)
        .unwrap_or_else(|e| panic!("{}", e.to_string()));
      let base64 = rspack_base64::encode_to_string(&map_buffer);
      let footer =
        format!("\n//# sourceMappingURL=data:application/json;charset=utf-8;base64,{base64}");
      RawSource::from(format!("eval({});", json!(format!("{source}{footer}")))).boxed()
    };
    MODULE_RENDER_CACHE.insert(origin_source, source.clone());
    render_source.source = source;
    return Ok(());
  }
  Ok(())
}

#[plugin_hook(JavascriptModulesChunkHash for EvalSourceMapDevToolPlugin)]
async fn eval_source_map_devtool_plugin_js_chunk_hash(
  &self,
  _compilation: &Compilation,
  _chunk_ukey: &ChunkUkey,
  hasher: &mut RspackHash,
) -> Result<()> {
  EVAL_SOURCE_MAP_DEV_TOOL_PLUGIN_NAME.hash(hasher);
  Ok(())
}

#[plugin_hook(JavascriptModulesInlineInRuntimeBailout for EvalSourceMapDevToolPlugin)]
fn eval_source_map_devtool_plugin_inline_in_runtime_bailout(
  &self,
  _compilation: &Compilation,
) -> Result<Option<String>> {
  Ok(Some("the eval-source-map devtool is used.".to_string()))
}

#[async_trait::async_trait]
impl Plugin for EvalSourceMapDevToolPlugin {
  fn name(&self) -> &'static str {
    EVAL_SOURCE_MAP_DEV_TOOL_PLUGIN_NAME
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
      .tap(eval_source_map_devtool_plugin_compilation::new(self));
    Ok(())
  }
}
