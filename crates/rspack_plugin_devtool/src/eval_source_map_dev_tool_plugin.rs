use std::{borrow::Cow, hash::Hash};

use dashmap::DashMap;
use derivative::Derivative;
use futures::future::join_all;
use once_cell::sync::Lazy;
use rspack_core::{
  rspack_sources::{BoxSource, MapOptions, RawSource, Source, SourceExt},
  ApplyContext, BoxModule, ChunkInitFragments, ChunkUkey, Compilation, CompilationParams,
  CompilerCompilation, CompilerOptions, ModuleIdentifier, Plugin, PluginContext,
};
use rspack_error::Result;
use rspack_hash::RspackHash;
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_javascript::{
  JavascriptModulesChunkHash, JavascriptModulesInlineInRuntimeBailout,
  JavascriptModulesRenderModuleContent, JsPlugin, RenderSource,
};
use rspack_util::identifier::make_paths_absolute;
use serde_json::json;

use crate::{
  module_filename_helpers::ModuleFilenameHelpers, ModuleFilenameTemplate, ModuleOrSource,
  SourceMapDevToolPluginOptions,
};

static MODULE_RENDER_CACHE: Lazy<DashMap<BoxSource, BoxSource>> = Lazy::new(DashMap::default);

const EVAL_SOURCE_MAP_DEV_TOOL_PLUGIN_NAME: &str = "rspack.EvalSourceMapDevToolPlugin";

#[plugin]
#[derive(Derivative)]
#[derivative(Debug)]
pub struct EvalSourceMapDevToolPlugin {
  columns: bool,
  no_sources: bool,
  #[derivative(Debug = "ignore")]
  module_filename_template: ModuleFilenameTemplate,
  namespace: String,
}

impl EvalSourceMapDevToolPlugin {
  pub fn new(options: SourceMapDevToolPluginOptions) -> Self {
    let module_filename_template =
      options
        .module_filename_template
        .unwrap_or(ModuleFilenameTemplate::String(
          "webpack://[namespace]/[resource-path]?[hash]".to_string(),
        ));

    let namespace = options.namespace.unwrap_or("".to_string());

    Self::new_inner(
      options.columns,
      options.no_sources,
      module_filename_template,
      namespace,
    )
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
  let output_options = &compilation.options.output;

  let origin_source = render_source.source.clone();
  if let Some(cached) = MODULE_RENDER_CACHE.get(&origin_source) {
    render_source.source = cached.value().clone();
    return Ok(());
  } else if let Some(mut map) = origin_source.map(&MapOptions::new(self.columns)) {
    let source = {
      let source = &origin_source.source();

      {
        let sources = map.sources_mut();
        let modules = sources.iter().map(|source| {
          if let Some(stripped) = source.strip_prefix("webpack://") {
            let source = make_paths_absolute(compilation.options.context.as_str(), stripped);
            let identifier = ModuleIdentifier::from(source.clone());
            match compilation
              .get_module_graph()
              .module_by_identifier(&identifier)
            {
              Some(module) => ModuleOrSource::Module(module.identifier()),
              None => ModuleOrSource::Source(source),
            }
          } else {
            ModuleOrSource::Source(source.to_string())
          }
        });
        let module_filenames = match &self.module_filename_template {
          ModuleFilenameTemplate::String(s) => modules
            .map(|module_or_source| {
              ModuleFilenameHelpers::create_filename_of_string_template(
                &module_or_source,
                compilation,
                s,
                output_options,
                &self.namespace,
              )
            })
            .collect::<Vec<_>>(),
          ModuleFilenameTemplate::Fn(f) => {
            let modules = modules.collect::<Vec<_>>();
            let features = modules.iter().map(|module_or_source| {
              ModuleFilenameHelpers::create_filename_of_fn_template(
                module_or_source,
                compilation,
                f,
                output_options,
                &self.namespace,
              )
            });
            futures::executor::block_on(join_all(features))
              .into_iter()
              .collect::<Result<Vec<_>>>()?
          }
        };
        let mut module_filenames =
          ModuleFilenameHelpers::replace_duplicates(module_filenames, |mut filename, _, n| {
            filename.extend(std::iter::repeat('*').take(n));
            return filename;
          })
          .into_iter()
          .map(|s| Some(s))
          .collect::<Vec<Option<_>>>();
        for (i, source) in sources.iter_mut().enumerate() {
          if let Some(filename) = module_filenames[i].take() {
            *source = Cow::from(filename);
          }
        }
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
