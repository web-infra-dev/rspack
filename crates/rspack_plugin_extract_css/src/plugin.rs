use std::{
  hash::Hash,
  sync::{Arc, LazyLock},
};

use rspack_cacheable::cacheable;
use rspack_core::{
  AssetInfo, ChunkGraph, ChunkKind, ChunkUkey, Compilation, CompilationContentHash,
  CompilationParams, CompilationRenderManifest, CompilationRuntimeRequirementInTree,
  CompilerCompilation, DependencyType, Filename, ManifestAssetType, ModuleType,
  NormalModuleFactoryParser, ParserAndGenerator, ParserOptions, PathData, Plugin,
  RenderManifestEntry, RuntimeGlobals, RuntimeModule, SourceType,
  rspack_sources::{CachedSource, SourceExt},
};
use rspack_error::{Diagnostic, Result};
use rspack_hash::RspackHash;
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_css::{
  plugin::{
    CssExtractAssetModule, CssExtractAssetRenderOptions, CssExtractOrderConflict,
    get_extract_modules_in_order, render_extract_css_asset,
  },
  runtime::{CssLoadingRuntimeInsert, CssLoadingRuntimeModule, ExtractCssLoadingRuntimeOptions},
};
use rspack_plugin_javascript::{
  BoxJavascriptParserPlugin, parser_and_generator::JavaScriptParserAndGenerator,
};
use rspack_plugin_runtime::GetChunkFilenameRuntimeModule;
use rustc_hash::FxHashMap;
use ustr::Ustr;

use crate::{
  css_module::{CssModule, CssModuleFactory},
  parser_plugin::PluginCssExtractParserPlugin,
};
pub static PLUGIN_NAME: &str = "css-extract-rspack-plugin";

pub static MODULE_TYPE_STR: LazyLock<Ustr> = LazyLock::new(|| Ustr::from("css/mini-extract"));

pub static MODULE_TYPE: LazyLock<ModuleType> =
  LazyLock::new(|| ModuleType::Custom(*MODULE_TYPE_STR));
pub static SOURCE_TYPE: LazyLock<[SourceType; 1]> =
  LazyLock::new(|| [SourceType::Custom(*MODULE_TYPE_STR)]);

pub static BASE_URI: &str = "rspack-css-extract://";
pub static ABSOLUTE_PUBLIC_PATH: &str = "rspack-css-extract:///css-extract-plugin/";
pub static AUTO_PUBLIC_PATH: &str = "__css_extract_public_path_auto__";
pub static SINGLE_DOT_PATH_SEGMENT: &str = "__css_extract_single_dot_path_segment__";

#[plugin]
#[derive(Debug)]
pub struct PluginCssExtract {
  pub(crate) options: Arc<CssExtractOptions>,
}

impl Eq for PluginCssExtractInner {}

impl PartialEq for PluginCssExtractInner {
  fn eq(&self, other: &Self) -> bool {
    Arc::ptr_eq(&self.options, &other.options)
  }
}

#[derive(Debug)]
pub struct CssExtractOptions {
  pub filename: Filename,
  pub chunk_filename: Filename,
  pub ignore_order: bool,
  pub insert: InsertType,
  pub attributes: FxHashMap<String, String>,
  pub link_type: Option<String>,
  pub runtime: bool,
  pub pathinfo: bool,
  pub enforce_relative: bool,
}

// impl PartialEq for CssExtractOptions {
//   fn eq(&self, other: &Self) -> bool {
//     let equal = self.ignore_order == other.ignore_order
//       && self.insert == other.insert
//       && self.attributes == other.attributes
//       && self.link_type == other.link_type
//       && self.runtime == other.runtime
//       && self.pathinfo == other.pathinfo;

//     if !equal {
//       return false;
//     }

//     // TODO: function eq
//     match (self.filename.template(), self.chunk_filename.template()) {
//       (None, None) => return true,
//       (None, Some(_)) => return false,
//       (Some(_), None) => return false,
//       (Some(a), Some(b)) => a == b,
//     }
//   }
// }

#[cacheable]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum InsertType {
  Fn(String),
  Selector(String),
  Default,
}

impl From<InsertType> for CssLoadingRuntimeInsert {
  fn from(value: InsertType) -> Self {
    match value {
      InsertType::Fn(value) => Self::Fn(value),
      InsertType::Selector(value) => Self::Selector(value),
      InsertType::Default => Self::Default,
    }
  }
}

impl PluginCssExtract {
  pub fn new(options: CssExtractOptions) -> Self {
    Self::new_inner(Arc::new(options))
  }

  fn order_conflict_diagnostics(
    conflicts: Vec<CssExtractOrderConflict>,
    filename: &str,
    compilation: &Compilation,
  ) -> Vec<Diagnostic> {
    let module_graph = compilation.get_module_graph();
    conflicts
      .into_iter()
      .map(|conflict| {
        let chunk = compilation
          .build_chunk_graph_artifact
          .chunk_by_ukey
          .expect_get(&conflict.chunk);
        let fallback_module = module_graph
          .module_by_identifier(&conflict.fallback_module)
          .expect("should have module");

        let mut diagnostic = Diagnostic::warn(
          String::new(),
          format!(
            r#"chunk {} [{PLUGIN_NAME}]
Conflicting order. Following module has been added:
 * {}
despite it was not able to fulfill desired ordering with these modules:
{}"#,
            chunk
              .name()
              .or_else(|| chunk.id().map(|id| id.as_str()))
              .unwrap_or_default(),
            fallback_module.readable_identifier(&compilation.options.context),
            conflict
              .reasons
              .iter()
              .map(|(m, failed_reasons, good_reasons)| {
                let m = module_graph
                  .module_by_identifier(m)
                  .expect("should have module");

                format!(
                  " * {}\n  - couldn't fulfill desired order of chunk group(s) {}{}",
                  m.readable_identifier(&compilation.options.context),
                  failed_reasons
                    .as_ref()
                    .map(|s| s.as_str())
                    .unwrap_or_default(),
                  good_reasons
                    .as_ref()
                    .map(|s| format!(
                      "\n  - while fulfilling desired order of chunk group(s) {}",
                      s.as_str()
                    ))
                    .unwrap_or_default(),
                )
              })
              .collect::<Vec<_>>()
              .join("\n")
          ),
        );
        diagnostic.file = Some(filename.to_owned().into());
        diagnostic.chunk = Some(chunk.ukey().as_u32());
        diagnostic
      })
      .collect()
  }
}

#[plugin_hook(CompilerCompilation for PluginCssExtract)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  compilation.set_dependency_factory(DependencyType::ExtractCSS, Arc::new(CssModuleFactory));
  Ok(())
}

#[plugin_hook(CompilationRuntimeRequirementInTree for PluginCssExtract)]
async fn runtime_requirement_in_tree(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  all_runtime_requirements: &RuntimeGlobals,
  runtime_requirements: &RuntimeGlobals,
  runtime_requirements_mut: &mut RuntimeGlobals,
  runtime_modules_to_add: &mut Vec<(ChunkUkey, Box<dyn RuntimeModule>)>,
) -> Result<Option<()>> {
  // different from webpack, Rspack can invoke this multiple times,
  // each time with current runtime_globals, and records every mutation
  // by `runtime_requirements_mut`, but this RuntimeModule depends on
  // 2 runtimeGlobals, if check current runtime_requirements, we might
  // insert CssLoadingRuntimeModule with with_loading: true but with_hmr: false
  // for the first time, and with_loading: false but with_hmr: true for the
  // second time
  // For plugin that depends on 2 runtime_globals, should check all_runtime_requirements
  if !self.options.runtime {
    return Ok(None);
  }

  let has_hot_update = runtime_requirements.contains(RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS);

  if has_hot_update || runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS) {
    let runtime_template = compilation.runtime_template.create_runtime_code_template();
    let filename = self.options.filename.clone();
    let chunk_filename = self.options.chunk_filename.clone();

    runtime_modules_to_add.push((
      *chunk_ukey,
      Box::new(GetChunkFilenameRuntimeModule::new(
        &compilation.runtime_template,
        "css",
        "mini-css",
        SOURCE_TYPE[0],
        format!(
          "{}.miniCssF",
          runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE)
        ),
        move |runtime_requirements| {
          runtime_requirements.contains(RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS)
        },
        move |chunk, compilation| {
          chunk
            .content_hash(&compilation.chunk_hashes_artifact)?
            .contains_key(&SOURCE_TYPE[0])
            .then(|| {
              if chunk.can_be_initial(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey) {
                filename.clone()
              } else {
                chunk_filename.clone()
              }
            })
        },
      )),
    ));

    runtime_modules_to_add.push((
      *chunk_ukey,
      Box::new(CssLoadingRuntimeModule::new_extract(
        &compilation.runtime_template,
        ExtractCssLoadingRuntimeOptions {
          attributes: self.options.attributes.clone(),
          link_type: self.options.link_type.clone(),
          insert: self.options.insert.clone().into(),
          source_type: SOURCE_TYPE[0],
        },
      )),
    ));

    runtime_requirements_mut.extend(CssLoadingRuntimeModule::get_extract_runtime_requirements(
      all_runtime_requirements,
    ));
  }

  Ok(None)
}

#[plugin_hook(CompilationContentHash for PluginCssExtract)]
async fn content_hash(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  hashes: &mut FxHashMap<SourceType, RspackHash>,
) -> Result<()> {
  let module_graph = compilation.get_module_graph();

  let rendered_modules = compilation
    .build_chunk_graph_artifact
    .chunk_graph
    .get_chunk_modules_by_source_type(chunk_ukey, SOURCE_TYPE[0], module_graph);

  if rendered_modules.is_empty() {
    return Ok(());
  }
  let chunk = compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .expect_get(chunk_ukey);

  let (used_modules, diagnostics) = get_extract_modules_in_order(
    chunk,
    &rendered_modules,
    compilation,
    module_graph,
    self.options.ignore_order,
  );

  let hasher = hashes
    .entry(SOURCE_TYPE[0])
    .or_insert_with(|| RspackHash::from(&compilation.options.output));

  used_modules
    .iter()
    .map(|m| ChunkGraph::get_module_hash(compilation, m.identifier(), chunk.runtime()))
    .for_each(|current| current.hash(hasher));

  " ".hash(hasher);
  if let Some(diagnostics) = diagnostics {
    diagnostics.iter().for_each(|curr| {
      curr.fallback_module.hash(hasher);
    });
  }

  Ok(())
}

#[plugin_hook(CompilationRenderManifest for PluginCssExtract)]
async fn render_manifest(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  manifest: &mut Vec<RenderManifestEntry>,
  diagnostics: &mut Vec<Diagnostic>,
) -> Result<()> {
  let module_graph = compilation.get_module_graph();
  let chunk = compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .expect_get(chunk_ukey);

  if matches!(chunk.kind(), ChunkKind::HotUpdate) {
    return Ok(());
  }

  let rendered_modules = compilation
    .build_chunk_graph_artifact
    .chunk_graph
    .get_chunk_modules_by_source_type(chunk_ukey, SOURCE_TYPE[0], module_graph);

  if rendered_modules.is_empty() {
    return Ok(());
  }

  let filename_template =
    if chunk.can_be_initial(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey) {
      &self.options.filename
    } else {
      &self.options.chunk_filename
    };

  let mut asset_info =
    AssetInfo::default().with_asset_type(ManifestAssetType::Custom("extract-css".into()));
  let filename = compilation
    .get_path_with_info(
      filename_template,
      PathData::default()
        .chunk_id_optional(chunk.id().map(|id| id.as_str()))
        .chunk_hash_optional(chunk.rendered_hash(
          &compilation.chunk_hashes_artifact,
          compilation.options.output.hash_digest_length,
        ))
        .chunk_name_optional(chunk.name_for_filename_template())
        .content_hash_optional(chunk.rendered_content_hash_by_source_type(
          &compilation.chunk_hashes_artifact,
          &SOURCE_TYPE[0],
          compilation.options.output.hash_digest_length,
        )),
      &mut asset_info,
    )
    .await?;

  let (source, more_diagnostics) = compilation
    .chunk_render_cache_artifact
    .use_cache(compilation, chunk, &SOURCE_TYPE[0], || async {
      let (used_modules, conflicts) = get_extract_modules_in_order(
        chunk,
        &rendered_modules,
        compilation,
        module_graph,
        self.options.ignore_order,
      );
      let diagnostics = conflicts
        .map(|conflicts| Self::order_conflict_diagnostics(conflicts, &filename, compilation))
        .unwrap_or_default();
      let modules = used_modules
        .into_iter()
        .filter_map(|module| module.downcast_ref::<CssModule>())
        .map(|module| CssExtractAssetModule {
          module,
          content: module.content.as_str(),
          media: module.media.as_deref(),
          supports: module.supports.as_deref(),
          source_map: module.source_map.as_deref(),
          css_layer: module.css_layer.as_deref(),
        })
        .collect::<Vec<_>>();
      let source = render_extract_css_asset(
        &modules,
        &CssExtractAssetRenderOptions {
          chunk,
          filename: &filename,
          compilation,
          pathinfo: self.options.pathinfo,
          enforce_relative: self.options.enforce_relative,
          base_uri: BASE_URI,
          absolute_public_path: ABSOLUTE_PUBLIC_PATH,
          auto_public_path: AUTO_PUBLIC_PATH,
          single_dot_path_segment: SINGLE_DOT_PATH_SEGMENT,
        },
      );
      Ok((CachedSource::new(source).boxed(), diagnostics))
    })
    .await?;

  diagnostics.extend(more_diagnostics);
  manifest.push(RenderManifestEntry {
    source,
    filename,
    has_filename: false,
    info: asset_info,
    auxiliary: false,
  });

  Ok(())
}

#[plugin_hook(NormalModuleFactoryParser for PluginCssExtract)]
async fn nmf_parser(
  &self,
  module_type: &ModuleType,
  parser: &mut Box<dyn ParserAndGenerator>,
  _parser_options: Option<&ParserOptions>,
) -> Result<()> {
  if module_type.is_js_like()
    && let Some(parser) = parser.downcast_mut::<JavaScriptParserAndGenerator>()
  {
    parser.add_parser_plugin(
      Box::<PluginCssExtractParserPlugin>::default() as BoxJavascriptParserPlugin
    );
  }
  Ok(())
}

impl Plugin for PluginCssExtract {
  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.compilation.tap(compilation::new(self));
    ctx
      .compilation_hooks
      .runtime_requirement_in_tree
      .tap(runtime_requirement_in_tree::new(self));
    ctx
      .compilation_hooks
      .content_hash
      .tap(content_hash::new(self));
    ctx
      .compilation_hooks
      .render_manifest
      .tap(render_manifest::new(self));

    ctx
      .normal_module_factory_hooks
      .parser
      .tap(nmf_parser::new(self));

    Ok(())
  }
}
