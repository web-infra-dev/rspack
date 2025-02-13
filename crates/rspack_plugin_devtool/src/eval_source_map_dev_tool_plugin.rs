use std::hash::Hash;

use dashmap::DashMap;
use derive_more::Debug;
use futures::future::join_all;
use rspack_core::{
  rspack_sources::{BoxSource, MapOptions, RawStringSource, Source, SourceExt},
  ApplyContext, BoxModule, ChunkGraph, ChunkInitFragments, ChunkUkey, Compilation,
  CompilationAdditionalModuleRuntimeRequirements, CompilationParams, CompilerCompilation,
  CompilerOptions, ModuleIdentifier, Plugin, PluginContext, RuntimeGlobals,
};
use rspack_error::Result;
use rspack_hash::RspackHash;
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_javascript::{
  JavascriptModulesChunkHash, JavascriptModulesInlineInRuntimeBailout,
  JavascriptModulesRenderModuleContent, JsPlugin, RenderSource,
};
use rspack_util::identifier::make_paths_absolute;

use crate::{
  module_filename_helpers::ModuleFilenameHelpers, ModuleFilenameTemplate, ModuleOrSource,
  SourceMapDevToolPluginOptions,
};

const EVAL_SOURCE_MAP_DEV_TOOL_PLUGIN_NAME: &str = "rspack.EvalSourceMapDevToolPlugin";

#[plugin]
#[derive(Debug)]
pub struct EvalSourceMapDevToolPlugin {
  columns: bool,
  no_sources: bool,
  #[debug(skip)]
  module_filename_template: ModuleFilenameTemplate,
  namespace: String,
  source_root: Option<String>,
  cache: DashMap<BoxSource, BoxSource>,
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
      options.source_root,
      Default::default(),
    )
  }
}

#[plugin_hook(CompilerCompilation for EvalSourceMapDevToolPlugin)]
async fn eval_source_map_devtool_plugin_compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  let mut hooks = JsPlugin::get_compilation_hooks_mut(compilation.id());
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
  module: &BoxModule,
  render_source: &mut RenderSource,
  _init_fragments: &mut ChunkInitFragments,
) -> Result<()> {
  let output_options = &compilation.options.output;

  let origin_source = render_source.source.clone();
  if let Some(cached_source) = self.cache.get(&origin_source) {
    render_source.source = cached_source.value().clone();
    return Ok(());
  } else if let Some(mut map) = origin_source.map(&MapOptions::new(self.columns)) {
    let source = {
      let source = &origin_source.source();

      {
        let modules = map.sources().iter().map(|source| {
          if let Some(stripped) = source.strip_prefix("webpack://") {
            let source = make_paths_absolute(compilation.options.context.as_str(), stripped);
            let identifier = ModuleIdentifier::from(source.as_str());
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
        let module_filenames =
          ModuleFilenameHelpers::replace_duplicates(module_filenames, |mut filename, _, n| {
            filename.extend(std::iter::repeat('*').take(n));
            filename
          });
        map.set_sources(module_filenames);
      }

      if self.no_sources {
        map.set_sources_content([]);
      }

      map.set_source_root(self.source_root.clone());
      map.set_file(Some(module.identifier().to_string()));

      let mut map_buffer = Vec::new();
      let module_ids = &compilation.module_ids_artifact;
      // align with https://github.com/webpack/webpack/blob/3919c844eca394d73ca930e4fc5506fb86e2b094/lib/EvalSourceMapDevToolPlugin.js#L171
      let module_id =
        if let Some(module_id) = ChunkGraph::get_module_id(module_ids, module.identifier()) {
          module_id.as_str()
        } else {
          "unknown"
        };
      map
        .to_writer(&mut map_buffer)
        .unwrap_or_else(|e| panic!("{}", e.to_string()));
      let base64 = rspack_base64::encode_to_string(&map_buffer);
      let footer =
        format!("\n//# sourceMappingURL=data:application/json;charset=utf-8;base64,{base64}\n//# sourceURL=webpack-internal:///{module_id}\n");
      let module_content =
        simd_json::to_string(&format!("{source}{footer}")).expect("should convert to string");
      RawStringSource::from(format!(
        "eval({});",
        if compilation.options.output.trusted_types.is_some() {
          format!("{}({})", RuntimeGlobals::CREATE_SCRIPT, module_content)
        } else {
          module_content
        }
      ))
      .boxed()
    };
    self.cache.insert(origin_source, source.clone());
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

#[plugin_hook(CompilationAdditionalModuleRuntimeRequirements for EvalSourceMapDevToolPlugin)]
fn eval_source_map_devtool_plugin_additional_module_runtime_requirements(
  &self,
  compilation: &Compilation,
  _module: &ModuleIdentifier,
  runtime_requirements: &mut RuntimeGlobals,
) -> Result<()> {
  if compilation.options.output.trusted_types.is_some() {
    runtime_requirements.insert(RuntimeGlobals::CREATE_SCRIPT);
  }

  Ok(())
}

#[async_trait::async_trait]
impl Plugin for EvalSourceMapDevToolPlugin {
  fn name(&self) -> &'static str {
    EVAL_SOURCE_MAP_DEV_TOOL_PLUGIN_NAME
  }

  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, _options: &CompilerOptions) -> Result<()> {
    ctx
      .context
      .compiler_hooks
      .compilation
      .tap(eval_source_map_devtool_plugin_compilation::new(self));
    ctx
      .context
      .compilation_hooks
      .additional_module_runtime_requirements
      .tap(eval_source_map_devtool_plugin_additional_module_runtime_requirements::new(self));
    Ok(())
  }
}
