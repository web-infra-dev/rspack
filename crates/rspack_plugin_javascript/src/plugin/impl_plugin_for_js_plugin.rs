use std::{hash::Hash, sync::Arc};

use rspack_core::{
  AssetInfo, CachedConstDependencyTemplate, ChunkGraph, ChunkKind, ChunkUkey, Compilation,
  CompilationAdditionalTreeRuntimeRequirements, CompilationChunkHash, CompilationContentHash,
  CompilationId, CompilationParams, CompilationRenderManifest, CompilerCompilation,
  ConstDependencyTemplate, DependencyType, IgnoreErrorModuleFactory, ManifestAssetType,
  ModuleGraph, ModuleType, ParserAndGenerator, PathData, Plugin, RenderManifestEntry,
  RuntimeGlobals, RuntimeRequirementsDependencyTemplate, SelfModuleFactory, SourceType,
  get_js_chunk_filename_template,
  rspack_sources::{BoxSource, CachedSource, SourceExt},
};
use rspack_error::{Diagnostic, Result};
use rspack_hash::RspackHash;
use rspack_hook::plugin_hook;
use rustc_hash::FxHashMap;

use crate::{
  JsPlugin, JsPluginInner,
  dependency::{
    AMDRequireContextDependencyTemplate, CommonJsExportRequireDependencyTemplate,
    CommonJsExportsDependencyTemplate, CommonJsFullRequireDependencyTemplate,
    CommonJsRequireContextDependencyTemplate, CommonJsRequireDependencyTemplate,
    CommonJsSelfReferenceDependencyTemplate, CreateScriptUrlDependencyTemplate,
    ESMAcceptDependencyTemplate, ESMCompatibilityDependencyTemplate,
    ESMExportExpressionDependencyTemplate, ESMExportHeaderDependencyTemplate,
    ESMExportImportedSpecifierDependencyTemplate, ESMExportSpecifierDependencyTemplate,
    ESMImportSideEffectDependencyTemplate, ESMImportSpecifierDependencyTemplate,
    ExportInfoDependencyTemplate, ExternalModuleDependencyTemplate,
    ImportContextDependencyTemplate, ImportDependencyTemplate, ImportEagerDependencyTemplate,
    ImportMetaContextDependencyTemplate, ImportMetaHotAcceptDependencyTemplate,
    ImportMetaHotDeclineDependencyTemplate, IsIncludedDependencyTemplate,
    ModuleArgumentDependencyTemplate, ModuleDecoratorDependencyTemplate,
    ModuleHotAcceptDependencyTemplate, ModuleHotDeclineDependencyTemplate,
    ProvideDependencyTemplate, PureExpressionDependencyTemplate, RequireContextDependencyTemplate,
    RequireEnsureDependencyTemplate, RequireHeaderDependencyTemplate,
    RequireResolveContextDependencyTemplate, RequireResolveDependencyTemplate,
    RequireResolveHeaderDependencyTemplate, URLDependencyTemplate, WorkerDependencyTemplate,
    amd_define_dependency::AMDDefineDependencyTemplate,
    amd_require_array_dependency::AMDRequireArrayDependencyTemplate,
    amd_require_dependency::AMDRequireDependencyTemplate,
    amd_require_item_dependency::AMDRequireItemDependencyTemplate,
    local_module_dependency::LocalModuleDependencyTemplate,
    unsupported_dependency::UnsupportedDependencyTemplate,
  },
  parser_and_generator::JavaScriptParserAndGenerator,
};

#[plugin_hook(CompilerCompilation for JsPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  params: &mut CompilationParams,
) -> Result<()> {
  // ESMModulesPlugin
  compilation.set_dependency_factory(
    DependencyType::EsmImport,
    params.normal_module_factory.clone(),
  );
  compilation.set_dependency_factory(
    DependencyType::EsmImportSpecifier,
    params.normal_module_factory.clone(),
  );
  compilation.set_dependency_factory(
    DependencyType::EsmExportImport,
    params.normal_module_factory.clone(),
  );
  compilation.set_dependency_factory(
    DependencyType::EsmExportImportedSpecifier,
    params.normal_module_factory.clone(),
  );
  compilation.set_dependency_factory(
    DependencyType::EsmExportSpecifier,
    params.normal_module_factory.clone(),
  );
  // CommonJsPlugin
  compilation.set_dependency_factory(
    DependencyType::CjsRequire,
    params.normal_module_factory.clone(),
  );
  compilation.set_dependency_factory(
    DependencyType::CjsFullRequire,
    params.normal_module_factory.clone(),
  );
  compilation.set_dependency_factory(
    DependencyType::CjsExports,
    params.normal_module_factory.clone(),
  );
  compilation.set_dependency_factory(
    DependencyType::CjsExportRequire,
    params.normal_module_factory.clone(),
  );
  compilation.set_dependency_factory(
    DependencyType::CommonJSRequireContext,
    params.context_module_factory.clone(),
  );
  compilation.set_dependency_factory(
    DependencyType::RequireResolve,
    params.normal_module_factory.clone(),
  );
  compilation.set_dependency_factory(
    DependencyType::RequireResolveContext,
    params.context_module_factory.clone(),
  );
  // AMDPlugin
  compilation.set_dependency_factory(
    DependencyType::AmdRequireItem,
    params.normal_module_factory.clone(),
  );
  compilation.set_dependency_factory(
    DependencyType::AmdRequireContext,
    params.context_module_factory.clone(),
  );
  // RequireContextPlugin
  compilation.set_dependency_factory(
    DependencyType::RequireContext,
    params.context_module_factory.clone(),
  );
  // RequireEnsurePlugin
  compilation.set_dependency_factory(
    DependencyType::RequireEnsureItem,
    params.normal_module_factory.clone(),
  );
  compilation.set_dependency_factory(
    DependencyType::ContextElement(rspack_core::ContextTypePrefix::Import),
    params.normal_module_factory.clone(),
  );
  compilation.set_dependency_factory(
    DependencyType::ContextElement(rspack_core::ContextTypePrefix::Normal),
    params.normal_module_factory.clone(),
  );
  // ImportMetaContextPlugin
  compilation.set_dependency_factory(
    DependencyType::ImportMetaContext,
    params.context_module_factory.clone(),
  );
  // ImportPlugin
  compilation.set_dependency_factory(
    DependencyType::DynamicImport,
    params.normal_module_factory.clone(),
  );

  compilation.set_dependency_factory(
    DependencyType::DynamicImportEager,
    params.normal_module_factory.clone(),
  );
  compilation.set_dependency_factory(
    DependencyType::ImportContext,
    params.context_module_factory.clone(),
  );
  // URLPlugin
  compilation.set_dependency_factory(DependencyType::NewUrl, params.normal_module_factory.clone());
  // ProvidePlugin
  compilation.set_dependency_factory(
    DependencyType::Provided,
    params.normal_module_factory.clone(),
  );
  // ImportModule
  compilation.set_dependency_factory(
    DependencyType::LoaderImport,
    params.normal_module_factory.clone(),
  );
  // other
  compilation.set_dependency_factory(
    DependencyType::IsIncluded,
    Arc::new(IgnoreErrorModuleFactory {
      normal_module_factory: params.normal_module_factory.clone(),
    }),
  );

  let self_factory = Arc::new(SelfModuleFactory {});
  compilation.set_dependency_factory(DependencyType::CjsSelfReference, self_factory.clone());
  compilation.set_dependency_factory(DependencyType::ModuleDecorator, self_factory);

  // esm dependency templates
  compilation.set_dependency_template(
    ESMCompatibilityDependencyTemplate::template_type(),
    Arc::new(ESMCompatibilityDependencyTemplate::default()),
  );
  compilation.set_dependency_template(
    ImportDependencyTemplate::template_type(),
    Arc::new(ImportDependencyTemplate::default()),
  );
  compilation.set_dependency_template(
    ESMExportExpressionDependencyTemplate::template_type(),
    Arc::new(ESMExportExpressionDependencyTemplate::default()),
  );
  compilation.set_dependency_template(
    ESMExportHeaderDependencyTemplate::template_type(),
    Arc::new(ESMExportHeaderDependencyTemplate::default()),
  );
  compilation.set_dependency_template(
    ESMExportImportedSpecifierDependencyTemplate::template_type(),
    Arc::new(ESMExportImportedSpecifierDependencyTemplate::default()),
  );
  compilation.set_dependency_template(
    ESMExportSpecifierDependencyTemplate::template_type(),
    Arc::new(ESMExportSpecifierDependencyTemplate::default()),
  );
  compilation.set_dependency_template(
    ESMImportSideEffectDependencyTemplate::template_type(),
    Arc::new(ESMImportSideEffectDependencyTemplate::default()),
  );
  compilation.set_dependency_template(
    ESMImportSpecifierDependencyTemplate::template_type(),
    Arc::new(ESMImportSpecifierDependencyTemplate::default()),
  );
  compilation.set_dependency_template(
    ExternalModuleDependencyTemplate::template_type(),
    Arc::new(ExternalModuleDependencyTemplate::default()),
  );
  compilation.set_dependency_template(
    ImportEagerDependencyTemplate::template_type(),
    Arc::new(ImportEagerDependencyTemplate::default()),
  );
  compilation.set_dependency_template(
    ProvideDependencyTemplate::template_type(),
    Arc::new(ProvideDependencyTemplate::default()),
  );

  // amd dependency templates
  compilation.set_dependency_template(
    AMDDefineDependencyTemplate::template_type(),
    Arc::new(AMDDefineDependencyTemplate::default()),
  );
  compilation.set_dependency_template(
    AMDRequireArrayDependencyTemplate::template_type(),
    Arc::new(AMDRequireArrayDependencyTemplate::default()),
  );
  compilation.set_dependency_template(
    AMDRequireDependencyTemplate::template_type(),
    Arc::new(AMDRequireDependencyTemplate::default()),
  );
  compilation.set_dependency_template(
    AMDRequireItemDependencyTemplate::template_type(),
    Arc::new(AMDRequireItemDependencyTemplate::default()),
  );
  compilation.set_dependency_template(
    LocalModuleDependencyTemplate::template_type(),
    Arc::new(LocalModuleDependencyTemplate::default()),
  );
  compilation.set_dependency_template(
    UnsupportedDependencyTemplate::template_type(),
    Arc::new(UnsupportedDependencyTemplate::default()),
  );
  // commonjs dependency templates
  compilation.set_dependency_template(
    CommonJsExportRequireDependencyTemplate::template_type(),
    Arc::new(CommonJsExportRequireDependencyTemplate::default()),
  );
  compilation.set_dependency_template(
    CommonJsExportsDependencyTemplate::template_type(),
    Arc::new(CommonJsExportsDependencyTemplate::default()),
  );
  compilation.set_dependency_template(
    CommonJsFullRequireDependencyTemplate::template_type(),
    Arc::new(CommonJsFullRequireDependencyTemplate::default()),
  );
  compilation.set_dependency_template(
    CommonJsRequireDependencyTemplate::template_type(),
    Arc::new(CommonJsRequireDependencyTemplate::default()),
  );
  compilation.set_dependency_template(
    CommonJsSelfReferenceDependencyTemplate::template_type(),
    Arc::new(CommonJsSelfReferenceDependencyTemplate::default()),
  );
  compilation.set_dependency_template(
    ModuleDecoratorDependencyTemplate::template_type(),
    Arc::new(ModuleDecoratorDependencyTemplate::default()),
  );
  compilation.set_dependency_template(
    RequireEnsureDependencyTemplate::template_type(),
    Arc::new(RequireEnsureDependencyTemplate::default()),
  );
  compilation.set_dependency_template(
    RequireHeaderDependencyTemplate::template_type(),
    Arc::new(RequireHeaderDependencyTemplate::default()),
  );
  compilation.set_dependency_template(
    RequireResolveDependencyTemplate::template_type(),
    Arc::new(RequireResolveDependencyTemplate::default()),
  );
  compilation.set_dependency_template(
    RequireResolveHeaderDependencyTemplate::template_type(),
    Arc::new(RequireResolveHeaderDependencyTemplate::default()),
  );

  // commonjs context dependency templates
  compilation.set_dependency_template(
    AMDRequireContextDependencyTemplate::template_type(),
    Arc::new(AMDRequireContextDependencyTemplate::default()),
  );
  compilation.set_dependency_template(
    CommonJsRequireContextDependencyTemplate::template_type(),
    Arc::new(CommonJsRequireContextDependencyTemplate::default()),
  );
  compilation.set_dependency_template(
    ImportContextDependencyTemplate::template_type(),
    Arc::new(ImportContextDependencyTemplate::default()),
  );
  compilation.set_dependency_template(
    ImportMetaContextDependencyTemplate::template_type(),
    Arc::new(ImportMetaContextDependencyTemplate::default()),
  );
  compilation.set_dependency_template(
    RequireContextDependencyTemplate::template_type(),
    Arc::new(RequireContextDependencyTemplate::default()),
  );
  compilation.set_dependency_template(
    RequireResolveContextDependencyTemplate::template_type(),
    Arc::new(RequireResolveContextDependencyTemplate::default()),
  );
  // hmr dependency templates
  compilation.set_dependency_template(
    ESMAcceptDependencyTemplate::template_type(),
    Arc::new(ESMAcceptDependencyTemplate::default()),
  );
  compilation.set_dependency_template(
    ImportMetaHotAcceptDependencyTemplate::template_type(),
    Arc::new(ImportMetaHotAcceptDependencyTemplate::default()),
  );
  compilation.set_dependency_template(
    ImportMetaHotDeclineDependencyTemplate::template_type(),
    Arc::new(ImportMetaHotDeclineDependencyTemplate::default()),
  );
  compilation.set_dependency_template(
    ModuleHotAcceptDependencyTemplate::template_type(),
    Arc::new(ModuleHotAcceptDependencyTemplate::default()),
  );
  compilation.set_dependency_template(
    ModuleHotDeclineDependencyTemplate::template_type(),
    Arc::new(ModuleHotDeclineDependencyTemplate::default()),
  );
  // url dependency templates
  compilation.set_dependency_template(
    URLDependencyTemplate::template_type(),
    Arc::new(URLDependencyTemplate::default()),
  );
  // worker dependency templates
  compilation.set_dependency_template(
    CreateScriptUrlDependencyTemplate::template_type(),
    Arc::new(CreateScriptUrlDependencyTemplate::default()),
  );
  compilation.set_dependency_template(
    WorkerDependencyTemplate::template_type(),
    Arc::new(WorkerDependencyTemplate::default()),
  );
  compilation.set_dependency_template(
    ExportInfoDependencyTemplate::template_type(),
    Arc::new(ExportInfoDependencyTemplate::default()),
  );
  compilation.set_dependency_template(
    IsIncludedDependencyTemplate::template_type(),
    Arc::new(IsIncludedDependencyTemplate::default()),
  );
  compilation.set_dependency_template(
    ModuleArgumentDependencyTemplate::template_type(),
    Arc::new(ModuleArgumentDependencyTemplate::default()),
  );
  compilation.set_dependency_template(
    PureExpressionDependencyTemplate::template_type(),
    Arc::new(PureExpressionDependencyTemplate::default()),
  );
  // core plugins
  compilation.set_dependency_template(
    CachedConstDependencyTemplate::template_type(),
    Arc::new(CachedConstDependencyTemplate::default()),
  );
  compilation.set_dependency_template(
    ConstDependencyTemplate::template_type(),
    Arc::new(ConstDependencyTemplate::default()),
  );
  compilation.set_dependency_template(
    RuntimeRequirementsDependencyTemplate::template_type(),
    Arc::new(RuntimeRequirementsDependencyTemplate::default()),
  );
  // Rstest
  compilation.set_dependency_factory(
    DependencyType::RstestMockModuleId,
    params.normal_module_factory.clone(),
  );
  Ok(())
}

#[plugin_hook(CompilationAdditionalTreeRuntimeRequirements for JsPlugin)]
async fn additional_tree_runtime_requirements(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &mut RuntimeGlobals,
) -> Result<()> {
  if !runtime_requirements.contains(RuntimeGlobals::STARTUP_NO_DEFAULT)
    && compilation
      .chunk_graph
      .has_chunk_entry_dependent_chunks(chunk_ukey, &compilation.chunk_group_by_ukey)
  {
    runtime_requirements.insert(RuntimeGlobals::ON_CHUNKS_LOADED);
    runtime_requirements.insert(RuntimeGlobals::REQUIRE);
  }
  Ok(())
}

#[plugin_hook(CompilationChunkHash for JsPlugin)]
async fn chunk_hash(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  hasher: &mut RspackHash,
) -> Result<()> {
  self.get_chunk_hash(chunk_ukey, compilation, hasher).await?;
  if compilation
    .chunk_by_ukey
    .expect_get(chunk_ukey)
    .has_runtime(&compilation.chunk_group_by_ukey)
  {
    self
      .update_hash_with_bootstrap(chunk_ukey, compilation, hasher)
      .await?;
  }
  Ok(())
}

#[plugin_hook(CompilationContentHash for JsPlugin)]
async fn content_hash(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  hashes: &mut FxHashMap<SourceType, RspackHash>,
) -> Result<()> {
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
  let mut hasher = hashes
    .entry(SourceType::JavaScript)
    .or_insert_with(|| RspackHash::from(&compilation.options.output));

  if !chunk.has_runtime(&compilation.chunk_group_by_ukey) {
    chunk.id().hash(&mut hasher);
  }

  let module_graph = compilation.get_module_graph();
  let mut ordered_modules = compilation
    .chunk_graph
    .get_chunk_modules_identifier_by_source_type(chunk_ukey, SourceType::JavaScript, &module_graph);
  // SAFETY: module identifier is unique
  ordered_modules.sort_unstable();

  ordered_modules
    .iter()
    .map(|mid| {
      (
        compilation
          .code_generation_results
          .get_hash(mid, Some(chunk.runtime())),
        ChunkGraph::get_module_id(&compilation.module_ids_artifact, *mid),
      )
    })
    .for_each(|(current, id)| {
      if let Some(current) = current {
        current.hash(&mut hasher);
        id.hash(&mut hasher);
      }
    });

  for (runtime_module_identifier, _) in compilation
    .chunk_graph
    .get_chunk_runtime_modules_in_order(chunk_ukey, compilation)
  {
    if let Some(hash) = compilation
      .runtime_modules_hash
      .get(runtime_module_identifier)
    {
      hash.hash(&mut hasher);
    }
  }

  Ok(())
}

#[plugin_hook(CompilationRenderManifest for JsPlugin)]
async fn render_manifest(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  manifest: &mut Vec<RenderManifestEntry>,
  _diagnostics: &mut Vec<Diagnostic>,
) -> Result<()> {
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
  let is_hot_update = matches!(chunk.kind(), ChunkKind::HotUpdate);
  let is_main_chunk = chunk.groups().iter().any(|group_ukey| {
    let group = compilation.chunk_group_by_ukey.expect_get(group_ukey);

    group.is_initial() && group.kind.is_entrypoint() && &group.get_entrypoint_chunk() == chunk_ukey
  });
  let is_runtime_chunk = chunk.has_runtime(&compilation.chunk_group_by_ukey);

  if !is_hot_update
    && is_runtime_chunk
    && !chunk_has_runtime_or_js(
      chunk_ukey,
      &compilation.chunk_graph,
      &compilation.get_module_graph(),
    )
  {
    return Ok(());
  }
  if !is_hot_update
    && !is_main_chunk
    && !is_runtime_chunk
    && !chunk_has_js(
      chunk_ukey,
      &compilation.chunk_graph,
      &compilation.get_module_graph(),
    )
  {
    return Ok(());
  }
  let filename_template = get_js_chunk_filename_template(
    chunk,
    &compilation.options.output,
    &compilation.chunk_group_by_ukey,
  );
  let mut asset_info = AssetInfo::default().with_asset_type(ManifestAssetType::JavaScript);
  asset_info.set_javascript_module(compilation.options.output.module);
  let output_path = compilation
    .get_path_with_info(
      &filename_template,
      PathData::default()
        .chunk_hash_optional(chunk.rendered_hash(
          &compilation.chunk_hashes_artifact,
          compilation.options.output.hash_digest_length,
        ))
        .chunk_id_optional(chunk.id().map(|id| id.as_str()))
        .chunk_name_optional(chunk.name_for_filename_template())
        .content_hash_optional(chunk.rendered_content_hash_by_source_type(
          &compilation.chunk_hashes_artifact,
          &SourceType::JavaScript,
          compilation.options.output.hash_digest_length,
        ))
        .runtime(chunk.runtime().as_str()),
      &mut asset_info,
    )
    .await?;

  let hooks = JsPlugin::get_compilation_hooks(compilation.id());
  let hooks = hooks.read().await;

  let (source, _) = compilation
    .chunk_render_cache_artifact
    .use_cache(compilation, chunk, &SourceType::JavaScript, || async {
      let source = if let Some(source) = hooks
        .render_chunk_content
        .call(compilation, chunk_ukey, &mut asset_info)
        .await?
      {
        source.source
      } else if is_hot_update {
        self
          .render_chunk(compilation, chunk_ukey, &output_path)
          .await?
      } else if is_runtime_chunk {
        self
          .render_main(compilation, chunk_ukey, &output_path)
          .await?
      } else {
        self
          .render_chunk(compilation, chunk_ukey, &output_path)
          .await?
      };
      Ok((CachedSource::new(source).boxed(), Vec::new()))
    })
    .await?;

  manifest.push(RenderManifestEntry {
    source,
    filename: output_path,
    has_filename: false,
    info: asset_info,
    auxiliary: false,
  });
  Ok(())
}

impl Plugin for JsPlugin {
  fn name(&self) -> &'static str {
    "javascript"
  }
  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.compilation.tap(compilation::new(self));
    ctx
      .compilation_hooks
      .additional_tree_runtime_requirements
      .tap(additional_tree_runtime_requirements::new(self));
    ctx.compilation_hooks.chunk_hash.tap(chunk_hash::new(self));
    ctx
      .compilation_hooks
      .content_hash
      .tap(content_hash::new(self));
    ctx
      .compilation_hooks
      .render_manifest
      .tap(render_manifest::new(self));

    ctx.register_parser_and_generator_builder(ModuleType::JsAuto, {
      Box::new(move |_, _| {
        Box::<JavaScriptParserAndGenerator>::default() as Box<dyn ParserAndGenerator>
      })
    });
    ctx.register_parser_and_generator_builder(ModuleType::JsEsm, {
      Box::new(move |_, _| {
        Box::<JavaScriptParserAndGenerator>::default() as Box<dyn ParserAndGenerator>
      })
    });
    ctx.register_parser_and_generator_builder(ModuleType::JsDynamic, {
      Box::new(move |_, _| {
        Box::<JavaScriptParserAndGenerator>::default() as Box<dyn ParserAndGenerator>
      })
    });

    Ok(())
  }

  fn clear_cache(&self, id: CompilationId) {
    crate::plugin::COMPILATION_HOOKS_MAP
      .write()
      .expect("should have js plugin drive")
      .remove(&id);
  }
}

#[derive(Debug, Clone)]
pub struct ExtractedCommentsInfo {
  pub source: BoxSource,
  pub comments_file_name: String,
}

fn chunk_has_js(chunk: &ChunkUkey, chunk_graph: &ChunkGraph, module_graph: &ModuleGraph) -> bool {
  if chunk_graph.get_number_of_entry_modules(chunk) > 0 {
    true
  } else {
    chunk_graph.has_chunk_module_by_source_type(chunk, SourceType::JavaScript, module_graph)
  }
}

fn chunk_has_runtime_or_js(
  chunk: &ChunkUkey,
  chunk_graph: &ChunkGraph,
  module_graph: &ModuleGraph,
) -> bool {
  if chunk_graph
    .get_chunk_runtime_modules_iterable(chunk)
    .next()
    .is_some()
  {
    return true;
  }
  if chunk_graph.has_chunk_module_by_source_type(chunk, SourceType::JavaScript, module_graph) {
    return true;
  }
  false
}
