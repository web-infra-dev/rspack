use std::hash::Hash;

use dashmap::DashMap;
use derivative::Derivative;
use once_cell::sync::Lazy;
use rspack_core::{
  rspack_sources::{BoxSource, RawSource, Source, SourceExt},
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
use serde_json::json;

use crate::{
  module_filename_helpers::ModuleFilenameHelpers, ModuleFilenameTemplate, ModuleOrSource,
};

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct EvalDevToolModulePluginOptions {
  pub namespace: Option<String>,
  #[derivative(Debug = "ignore")]
  pub module_filename_template: Option<ModuleFilenameTemplate>,
  pub source_url_comment: Option<String>,
}

static EVAL_MODULE_RENDER_CACHE: Lazy<DashMap<BoxSource, BoxSource>> = Lazy::new(DashMap::default);

const EVAL_DEV_TOOL_MODULE_PLUGIN_NAME: &str = "rspack.EvalDevToolModulePlugin";

#[plugin]
#[derive(Debug)]
pub struct EvalDevToolModulePlugin {
  options: EvalDevToolModulePluginOptions,
}

impl EvalDevToolModulePlugin {
  pub fn new(options: EvalDevToolModulePluginOptions) -> Self {
    Self::new_inner(options)
  }
}

#[plugin_hook(CompilerCompilation for EvalDevToolModulePlugin)]
async fn eval_devtool_plugin_compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  let mut hooks = JsPlugin::get_compilation_hooks_mut(compilation);
  hooks
    .render_module_content
    .tap(eval_devtool_plugin_render_module_content::new(self));
  hooks
    .chunk_hash
    .tap(eval_devtool_plugin_js_chunk_hash::new(self));
  hooks
    .inline_in_runtime_bailout
    .tap(eval_devtool_plugin_inline_in_runtime_bailout::new(self));
  Ok(())
}

#[plugin_hook(JavascriptModulesRenderModuleContent for EvalDevToolModulePlugin)]
fn eval_devtool_plugin_render_module_content(
  &self,
  compilation: &Compilation,
  module: &BoxModule,
  render_source: &mut RenderSource,
  _init_fragments: &mut ChunkInitFragments,
) -> Result<()> {
  let origin_source = render_source.source.clone();
  if let Some(cached) = EVAL_MODULE_RENDER_CACHE.get(&origin_source) {
    render_source.source = cached.value().clone();
    return Ok(());
  } else if module.as_external_module().is_some() {
    return Ok(());
  }
  let output_options = &compilation.options.output;
  let source_url_comment = self
    .options
    .source_url_comment
    .clone()
    .unwrap_or("\n//# sourceURL=[url]".to_string());
  let namespace = self.options.namespace.clone().unwrap_or_default();
  let module_filename_template =
    self
      .options
      .module_filename_template
      .clone()
      .unwrap_or(ModuleFilenameTemplate::String(
        "webpack://[namespace]/[resourcePath]?[loaders]".to_string(),
      ));
  let source_name = match &module_filename_template {
    ModuleFilenameTemplate::String(s) => ModuleFilenameHelpers::create_filename_of_string_template(
      &ModuleOrSource::Module(module.identifier()),
      compilation,
      s,
      output_options,
      namespace.as_str(),
    ),
    ModuleFilenameTemplate::Fn(f) => {
      futures::executor::block_on(ModuleFilenameHelpers::create_filename_of_fn_template(
        &ModuleOrSource::Module(module.identifier()),
        compilation,
        f,
        output_options,
        namespace.as_str(),
      ))
      .expect("todo!")
    }
  };
  let source = {
    let source = &origin_source.source();
    let footer = source_url_comment.replace("[url]", &source_name);
    RawSource::from(format!("eval({});", json!(format!("{source}{footer}")))).boxed()
  };

  EVAL_MODULE_RENDER_CACHE.insert(origin_source, source.clone());
  render_source.source = source;
  Ok(())
}

#[plugin_hook(JavascriptModulesChunkHash for EvalDevToolModulePlugin)]
async fn eval_devtool_plugin_js_chunk_hash(
  &self,
  _compilation: &Compilation,
  _chunk_ukey: &ChunkUkey,
  hasher: &mut RspackHash,
) -> Result<()> {
  EVAL_DEV_TOOL_MODULE_PLUGIN_NAME.hash(hasher);
  Ok(())
}

#[plugin_hook(JavascriptModulesInlineInRuntimeBailout for EvalDevToolModulePlugin)]
fn eval_devtool_plugin_inline_in_runtime_bailout(
  &self,
  _compilation: &Compilation,
) -> Result<Option<String>> {
  Ok(Some("the eval devtool is used.".to_string()))
}

impl Plugin for EvalDevToolModulePlugin {
  fn name(&self) -> &'static str {
    EVAL_DEV_TOOL_MODULE_PLUGIN_NAME
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
      .tap(eval_devtool_plugin_compilation::new(self));
    Ok(())
  }
}
