use std::hash::Hash;
use std::sync::Arc;

use async_trait::async_trait;
use rspack_core::rspack_sources::BoxSource;
use rspack_core::{
  get_js_chunk_filename_template, AdditionalChunkRuntimeRequirementsArgs, ChunkGraph,
  ChunkHashArgs, ChunkKind, ChunkUkey, Compilation, CompilationParams, CompilerOptions,
  DependencyType, ErrorSpan, IgnoreErrorModuleFactory, ModuleGraph, ModuleType, ParserAndGenerator,
  PathData, Plugin, PluginAdditionalChunkRuntimeRequirementsOutput, PluginChunkHashHookOutput,
  PluginContext, PluginRenderManifestHookOutput, RenderManifestEntry, RuntimeGlobals,
  SelfModuleFactory, SourceType,
};
use rspack_error::{IntoTWithDiagnosticArray, Result};
use rspack_hash::RspackHash;
use rspack_hook::AsyncSeries2;

use crate::parser_and_generator::JavaScriptParserAndGenerator;
use crate::JsPlugin;

struct JsPluginCompilationHook;

#[async_trait]
impl AsyncSeries2<Compilation, CompilationParams> for JsPluginCompilationHook {
  async fn run(&self, compilation: &mut Compilation, params: &mut CompilationParams) -> Result<()> {
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
      DependencyType::ContextElement,
      params.normal_module_factory.clone(),
    );
    // ImportMetaContextPlugin
    compilation.set_dependency_factory(
      DependencyType::ImportMetaContext,
      params.context_module_factory.clone(),
    );
    compilation.set_dependency_factory(
      DependencyType::ContextElement,
      params.normal_module_factory.clone(),
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
    compilation
      .set_dependency_factory(DependencyType::NewUrl, params.normal_module_factory.clone());
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
      .tap(Box::new(JsPluginCompilationHook));

    let create_parser_and_generator =
      move || Box::new(JavaScriptParserAndGenerator) as Box<dyn ParserAndGenerator>;

    ctx
      .context
      .register_parser_and_generator_builder(ModuleType::Js, Box::new(create_parser_and_generator));
    ctx.context.register_parser_and_generator_builder(
      ModuleType::JsEsm,
      Box::new(create_parser_and_generator),
    );
    ctx.context.register_parser_and_generator_builder(
      ModuleType::JsDynamic,
      Box::new(create_parser_and_generator),
    );

    Ok(())
  }

  async fn chunk_hash(
    &self,
    _ctx: PluginContext,
    args: &mut ChunkHashArgs<'_>,
  ) -> PluginChunkHashHookOutput {
    self
      .get_chunk_hash(&args.chunk_ukey, args.compilation, args.hasher)
      .await?;
    if args
      .chunk()
      .has_runtime(&args.compilation.chunk_group_by_ukey)
    {
      self.update_hash_with_bootstrap(&args.chunk_ukey, args.compilation, args.hasher)
    }
    Ok(())
  }

  async fn content_hash(
    &self,
    _ctx: rspack_core::PluginContext,
    args: &rspack_core::ContentHashArgs<'_>,
  ) -> rspack_core::PluginContentHashHookOutput {
    let compilation = &args.compilation;
    let chunk = args.chunk();
    let mut hasher = RspackHash::from(&compilation.options.output);

    if chunk.has_runtime(&args.compilation.chunk_group_by_ukey) {
      self.update_hash_with_bootstrap(&args.chunk_ukey, args.compilation, &mut hasher)
    } else {
      chunk.id.hash(&mut hasher);
      chunk.ids.hash(&mut hasher);
    }

    self
      .get_chunk_hash(&args.chunk_ukey, args.compilation, &mut hasher)
      .await?;

    let mut ordered_modules = compilation.chunk_graph.get_chunk_modules_by_source_type(
      &args.chunk_ukey,
      SourceType::JavaScript,
      compilation.get_module_graph(),
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
      .get_chunk_runtime_modules_in_order(&args.chunk_ukey, compilation)
    {
      if let Some((hash, _)) = compilation
        .runtime_module_code_generation_results
        .get(runtime_module_idenfitier)
      {
        hash.hash(&mut hasher);
      }
    }

    Ok(Some((
      SourceType::JavaScript,
      hasher.digest(&compilation.options.output.hash_digest),
    )))
  }

  async fn render_manifest(
    &self,
    _ctx: PluginContext,
    args: rspack_core::RenderManifestArgs<'_>,
  ) -> PluginRenderManifestHookOutput {
    let compilation = args.compilation;
    let chunk = args.chunk();
    let source = if matches!(chunk.kind, ChunkKind::HotUpdate) {
      self.render_chunk_impl(&args).await?
    } else if chunk.has_runtime(&compilation.chunk_group_by_ukey) {
      self.render_main(&args).await?
    } else {
      if !chunk_has_js(
        &args.chunk_ukey,
        &compilation.chunk_graph,
        compilation.get_module_graph(),
      ) {
        return Ok(vec![].with_empty_diagnostic());
      }

      self.render_chunk_impl(&args).await?
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
    );
    asset_info.set_javascript_module(compilation.options.output.module);
    Ok(
      vec![RenderManifestEntry::new(
        source,
        output_path,
        asset_info,
        false,
        false,
      )]
      .with_empty_diagnostic(),
    )
  }

  async fn additional_tree_runtime_requirements(
    &self,
    _ctx: PluginContext,
    args: &mut AdditionalChunkRuntimeRequirementsArgs,
  ) -> PluginAdditionalChunkRuntimeRequirementsOutput {
    let compilation = &mut args.compilation;
    let runtime_requirements = &mut args.runtime_requirements;
    if !runtime_requirements.contains(RuntimeGlobals::STARTUP_NO_DEFAULT)
      && compilation
        .chunk_graph
        .has_chunk_entry_dependent_chunks(args.chunk, &compilation.chunk_group_by_ukey)
    {
      runtime_requirements.insert(RuntimeGlobals::ON_CHUNKS_LOADED);
      runtime_requirements.insert(RuntimeGlobals::REQUIRE);
    }
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
