use std::hash::Hash;
use std::sync::Arc;

use async_trait::async_trait;
use rspack_core::rspack_sources::BoxSource;
use rspack_core::{
  get_js_chunk_filename_template, ChunkGraph, ChunkKind, ChunkUkey, Compilation,
  CompilationAdditionalTreeRuntimeRequirements, CompilationChunkHash, CompilationContentHash,
  CompilationParams, CompilationRenderManifest, CompilerCompilation, CompilerOptions,
  DependencyType, ErrorSpan, IgnoreErrorModuleFactory, ModuleGraph, ModuleType, ParserAndGenerator,
  PathData, Plugin, PluginContext, RenderManifestEntry, RuntimeGlobals, SelfModuleFactory,
  SourceType,
};
use rspack_error::{Diagnostic, Result};
use rspack_hash::RspackHash;
use rspack_hook::plugin_hook;
use rustc_hash::FxHashMap;

use crate::parser_and_generator::JavaScriptParserAndGenerator;
use crate::{JsPlugin, JsPluginInner};

#[plugin_hook(CompilerCompilation for JsPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  params: &mut CompilationParams,
) -> Result<()> {
  // HarmonyModulesPlugin
  compilation.set_dependency_factory(
    DependencyType::EsmImport(ErrorSpan::default()),
    params.normal_module_factory.clone(),
  );
  compilation.set_dependency_factory(
    DependencyType::EsmImportSpecifier,
    params.normal_module_factory.clone(),
  );
  compilation.set_dependency_factory(
    DependencyType::EsmExport(ErrorSpan::default()),
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
  // RequireContextPlugin
  compilation.set_dependency_factory(
    DependencyType::RequireContext,
    params.context_module_factory.clone(),
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
  // other
  compilation.set_dependency_factory(
    DependencyType::WebpackIsIncluded,
    Arc::new(IgnoreErrorModuleFactory {
      normal_module_factory: params.normal_module_factory.clone(),
    }),
  );
  compilation.set_dependency_factory(
    DependencyType::CjsSelfReference,
    Arc::new(SelfModuleFactory {}),
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
fn chunk_hash(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  hasher: &mut RspackHash,
) -> Result<()> {
  self.get_chunk_hash(chunk_ukey, compilation, hasher)?;
  if compilation
    .chunk_by_ukey
    .expect_get(chunk_ukey)
    .has_runtime(&compilation.chunk_group_by_ukey)
  {
    self.update_hash_with_bootstrap(chunk_ukey, compilation, hasher)
  }
  Ok(())
}

#[plugin_hook(CompilationContentHash for JsPlugin)]
fn content_hash(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  hashes: &mut FxHashMap<SourceType, RspackHash>,
) -> Result<()> {
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
  let mut hasher = hashes
    .entry(SourceType::JavaScript)
    .or_insert_with(|| RspackHash::from(&compilation.options.output));

  if chunk.has_runtime(&compilation.chunk_group_by_ukey) {
    self.update_hash_with_bootstrap(chunk_ukey, compilation, hasher)
  } else {
    chunk.id.hash(&mut hasher);
    chunk.ids.hash(&mut hasher);
  }

  self.get_chunk_hash(chunk_ukey, compilation, hasher)?;

  let module_graph = compilation.get_module_graph();
  let mut ordered_modules = compilation.chunk_graph.get_chunk_modules_by_source_type(
    chunk_ukey,
    SourceType::JavaScript,
    &module_graph,
  );
  // SAFETY: module identifier is unique
  ordered_modules.sort_unstable_by_key(|m| m.identifier().as_str());

  ordered_modules
    .iter()
    .map(|mgm| {
      (
        compilation
          .code_generation_results
          .get_hash(&mgm.identifier(), Some(&chunk.runtime)),
        compilation.chunk_graph.get_module_id(mgm.identifier()),
      )
    })
    .for_each(|(current, id)| {
      if let Some(current) = current {
        current.hash(&mut hasher);
        id.hash(&mut hasher);
      }
    });

  for (runtime_module_idenfitier, _) in compilation
    .chunk_graph
    .get_chunk_runtime_modules_in_order(chunk_ukey, compilation)
  {
    if let Some((hash, _)) = compilation
      .runtime_module_code_generation_results
      .get(runtime_module_idenfitier)
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
  let source = if matches!(chunk.kind, ChunkKind::HotUpdate) {
    self.render_chunk(compilation, chunk_ukey).await?
  } else if chunk.has_runtime(&compilation.chunk_group_by_ukey) {
    self.render_main(compilation, chunk_ukey).await?
  } else {
    if !chunk_has_js(
      chunk_ukey,
      &compilation.chunk_graph,
      &compilation.get_module_graph(),
    ) {
      return Ok(());
    }

    self.render_chunk(compilation, chunk_ukey).await?
  };

  let filename_template = get_js_chunk_filename_template(
    chunk,
    &compilation.options.output,
    &compilation.chunk_group_by_ukey,
  );
  let (output_path, mut asset_info) = compilation.get_path_with_info(
    filename_template,
    PathData::default()
      .chunk(chunk)
      .content_hash_optional(
        chunk
          .content_hash
          .get(&SourceType::JavaScript)
          .map(|i| i.rendered(compilation.options.output.hash_digest_length)),
      )
      .runtime(&chunk.runtime),
  )?;
  asset_info.set_javascript_module(compilation.options.output.module);
  manifest.push(RenderManifestEntry::new(
    source,
    output_path,
    asset_info,
    false,
    false,
  ));
  Ok(())
}

#[async_trait]
impl Plugin for JsPlugin {
  fn name(&self) -> &'static str {
    "javascript"
  }
  fn apply(
    &self,
    ctx: PluginContext<&mut rspack_core::ApplyContext>,
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
      .additional_tree_runtime_requirements
      .tap(additional_tree_runtime_requirements::new(self));
    ctx
      .context
      .compilation_hooks
      .chunk_hash
      .tap(chunk_hash::new(self));
    ctx
      .context
      .compilation_hooks
      .content_hash
      .tap(content_hash::new(self));
    ctx
      .context
      .compilation_hooks
      .render_manifest
      .tap(render_manifest::new(self));

    ctx.context.register_parser_and_generator_builder(
      ModuleType::Js,
      Box::new(|_, _| Box::new(JavaScriptParserAndGenerator) as Box<dyn ParserAndGenerator>),
    );
    ctx.context.register_parser_and_generator_builder(
      ModuleType::JsEsm,
      Box::new(|_, _| Box::new(JavaScriptParserAndGenerator) as Box<dyn ParserAndGenerator>),
    );
    ctx.context.register_parser_and_generator_builder(
      ModuleType::JsDynamic,
      Box::new(|_, _| Box::new(JavaScriptParserAndGenerator) as Box<dyn ParserAndGenerator>),
    );

    Ok(())
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
    chunk_graph
      .get_chunk_modules_iterable_by_source_type(chunk, SourceType::JavaScript, module_graph)
      .next()
      .is_some()
  }
}
