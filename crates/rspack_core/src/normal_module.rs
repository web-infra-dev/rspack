use std::{
  borrow::Cow,
  sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
  },
};

use derive_more::Debug;
use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{As, AsInner, AsOption, AsPreset},
};
use rspack_collections::{Identifiable, IdentifierMap, IdentifierSet};
use rspack_error::{Diagnosable, Diagnostic, Result, error};
use rspack_fs::ReadableFileSystem;
use rspack_hash::{RspackHash, RspackHashDigest, RspackHasher};
use rspack_hook::define_hook;
use rspack_loader_runner::{AdditionalData, Content, LoaderContext, ResourceData, run_loaders};
use rspack_sources::{
  BoxSource, CachedSource, OriginalSource, RawBufferSource, RawStringSource, SourceExt, SourceMap,
  SourceMapSource, WithoutOriginalOptions,
};
use rspack_util::source_map::{ModuleSourceMapConfig, SourceMapKind};
use serde_json::json;
use tracing::{Instrument, info_span};

use crate::{
  BoxModule, BuildContext, BuildInfo, BuildMeta, ChunkGraph, CodeGenerationResultBuilder,
  Compilation, ConnectionState, Context, DependenciesBlock, DependenciesBlockData,
  DependencyCodeGenerationRef, DependencyId, FactoryMeta, FactoryMetaStore, FreezeLock, GenerateContext, GeneratorOptions,
  ImportPhase, LibIdentOptions, Loaders, Module, ModuleCodeGenerationContext, ModuleGraph,
  ModuleGraphCacheArtifact, ModuleIdentifier, ModuleLayer, ModuleType, NeedBuildContext,
  OptimizationBailoutItem, OutputOptions, ParseContext, ParseResult, ParserAndGenerator,
  ParserOptions, Resolve, ResolvedLoader, ResolvedModuleOptions, RspackLoaderRunnerPlugin,
  RunnerContext, RuntimeGlobals, RuntimeSpec, SideEffectsStateArtifact, SnapshotValidationResult,
  SourceType,
  cache::SnapshotStrategyOptions,
  contextify,
  diagnostics::ModuleBuildError,
  get_context, module_analyzed_side_effect_free, module_declared_side_effect_free,
  module_update_hash,
  new_cache::{FileSystemInfo, Snapshot},
  utils::{SourceSizeCache, SourceSizeCacheSerde},
};

#[cacheable]
#[derive(Debug, Clone)]
pub enum ModuleIssuer {
  Unset,
  None,
  Some(ModuleIdentifier),
}

impl ModuleIssuer {
  pub fn from_identifier(identifier: Option<ModuleIdentifier>) -> Self {
    match identifier {
      Some(id) => Self::Some(id),
      None => Self::None,
    }
  }

  pub fn identifier(&self) -> Option<&ModuleIdentifier> {
    match self {
      ModuleIssuer::Some(id) => Some(id),
      _ => None,
    }
  }

  pub fn get_module<'a>(&self, module_graph: &'a ModuleGraph) -> Option<&'a crate::ModuleRef> {
    if let Some(id) = self.identifier()
      && let Some(module) = module_graph.module_by_identifier(id)
    {
      Some(module)
    } else {
      None
    }
  }
}

define_hook!(NormalModuleReadResource: SeriesBail(resource_data: &ResourceData, fs: &Arc<dyn ReadableFileSystem>) -> Content,tracing=false);
define_hook!(NormalModuleLoader: Series(loader_context: &mut LoaderContext<RunnerContext>),tracing=false);
define_hook!(NormalModuleLoaderStartYielding: Series(loader_context: &mut LoaderContext<RunnerContext>),tracing=false);
define_hook!(NormalModuleBeforeLoaders: Series(module: &mut NormalModule),tracing=false);
define_hook!(NormalModuleAdditionalData: Series(additional_data: &mut Option<&mut AdditionalData>),tracing=false);

#[derive(Debug, Default)]
pub struct NormalModuleHooks {
  pub read_resource: NormalModuleReadResourceHook,
  pub loader: NormalModuleLoaderHook,
  pub loader_yield: NormalModuleLoaderStartYieldingHook,
  pub before_loaders: NormalModuleBeforeLoadersHook,
  pub additional_data: NormalModuleAdditionalDataHook,
}

#[cacheable]
#[derive(Debug)]
pub struct NormalModule {
  id: ModuleIdentifier,
  /// Context of this module
  context: Box<Context>,
  /// Request with loaders from config
  request: String,
  /// Request intended by user (without loaders from config)
  user_request: String,
  /// Request without resolving
  raw_request: String,
  /// The resolved module type of a module
  module_type: ModuleType,
  /// Layer of the module
  layer: Option<ModuleLayer>,
  /// Affiliated parser and generator to the module type
  parser_and_generator: Box<dyn ParserAndGenerator>,
  /// Resource matched with inline match resource, (`!=!` syntax)
  match_resource: Option<ResourceData>,
  /// Resource data (path, query, fragment etc.)
  resource_data: Arc<ResourceData>,
  /// Loaders for the module
  #[debug(skip)]
  pub(crate) loaders: Loaders,

  /// Resolve options derived from [Rule.resolve]
  resolve_options: Option<Arc<Resolve>>,
  /// Parser and generator options derived from [Rule.parser] and [Rule.generator]
  #[cacheable(with=AsInner)]
  parser_and_generator_options: Arc<ResolvedModuleOptions>,
  /// enable/disable extracting source map
  extract_source_map: Option<bool>,

  #[allow(unused)]
  debug_id: usize,
  #[cacheable(with=As<SourceSizeCacheSerde>)]
  cached_source_sizes: SourceSizeCache,

  dependencies_block: DependenciesBlockData,
  factory_meta: FactoryMetaStore,
  #[cacheable(with=AsOption<AsPreset>)]
  source: Option<BoxSource>,
  diagnostics: Vec<Diagnostic>,
  code_generation_dependencies: Option<Vec<DependencyId>>,
  presentational_dependencies: Option<Vec<DependencyCodeGenerationRef>>,
  build_info: FreezeLock<BuildInfo>,
  build_meta: FreezeLock<BuildMeta>,
  parsed: bool,
  force_build: bool,
  source_map_kind: SourceMapKind,
  /// A seal-stage override, excluded from reusable build state.
  #[cacheable(with=rspack_cacheable::with::Skip)]
  asset_filename_override: std::sync::RwLock<Option<crate::Filename>>,
}

static DEBUG_ID: AtomicUsize = AtomicUsize::new(1);

impl NormalModule {
  pub(crate) fn restore_build_meta(&self, build_meta: crate::SharedBuildMeta) {
    self.build_meta.freeze_with(build_meta);
  }

  fn create_id<'request>(
    module_type: &ModuleType,
    layer: Option<&ModuleLayer>,
    request: &'request str,
    phase: ImportPhase,
  ) -> Cow<'request, str> {
    if matches!(module_type, ModuleType::WasmAsync) {
      let mut id = format!("{module_type}|{request}|{}", phase.as_str());
      if let Some(layer) = layer {
        id.push('|');
        id.push_str(layer);
      }
      id.into()
    } else if let Some(layer) = layer {
      format!("{module_type}|{request}|{layer}").into()
    } else if *module_type == ModuleType::JsAuto {
      request.into()
    } else {
      format!("{module_type}|{request}").into()
    }
  }

  #[allow(clippy::too_many_arguments)]
  pub fn new(
    request: String,
    user_request: String,
    raw_request: String,
    module_type: impl Into<ModuleType>,
    layer: Option<ModuleLayer>,
    parser_and_generator: Box<dyn ParserAndGenerator>,
    parser_and_generator_options: Arc<ResolvedModuleOptions>,
    match_resource: Option<ResourceData>,
    resource_data: Arc<ResourceData>,
    resolve_options: Option<Arc<Resolve>>,
    loaders: Loaders,
    context: Option<Context>,
    extract_source_map: Option<bool>,
    import_phase: ImportPhase,
  ) -> Self {
    let module_type = module_type.into();
    let id = Self::create_id(&module_type, layer.as_ref(), &request, import_phase);
    let build_info = BuildInfo {
      import_phase,
      ..Default::default()
    };
    Self {
      id: ModuleIdentifier::from(id.as_ref()),
      context: Box::new(context.unwrap_or_else(|| get_context(&resource_data))),
      request,
      user_request,
      raw_request,
      module_type,
      layer,
      parser_and_generator,
      parser_and_generator_options,
      match_resource,
      resource_data,
      resolve_options,
      loaders,
      debug_id: DEBUG_ID.fetch_add(1, Ordering::Relaxed),
      extract_source_map,

      cached_source_sizes: SourceSizeCache::default(),
      dependencies_block: Default::default(),
      factory_meta: Default::default(),
      asset_filename_override: Default::default(),
      source: None,
      diagnostics: Default::default(),
      code_generation_dependencies: None,
      presentational_dependencies: None,
      build_info: build_info.into(),
      build_meta: Default::default(),
      parsed: false,
      // Mirrors webpack's `_forceBuild`: a new module has no reusable build
      // until its first build starts.
      force_build: true,
      source_map_kind: SourceMapKind::empty(),
    }
  }

  pub fn id(&self) -> ModuleIdentifier {
    self.id
  }

  pub fn match_resource(&self) -> Option<&ResourceData> {
    self.match_resource.as_ref()
  }

  pub fn match_resource_mut(&mut self) -> &mut Option<ResourceData> {
    &mut self.match_resource
  }

  pub fn resource_resolved_data(&self) -> &Arc<ResourceData> {
    &self.resource_data
  }

  pub fn request(&self) -> &str {
    &self.request
  }

  pub fn user_request(&self) -> &str {
    &self.user_request
  }

  pub fn raw_request(&self) -> &str {
    &self.raw_request
  }

  pub fn loaders(&self) -> &[ResolvedLoader] {
    self.loaders.loaders()
  }

  pub fn parser_and_generator(&self) -> &dyn ParserAndGenerator {
    &*self.parser_and_generator
  }

  #[tracing::instrument(
    "NormalModule:build_hash", skip_all,fields(
      resource = self.resource_data.resource()
    )
  )]
  fn init_build_hash(
    &self,
    output_options: &OutputOptions,
    build_meta: &BuildMeta,
  ) -> RspackHashDigest {
    let mut hasher = RspackHasher::from(output_options);
    "source".hash(&mut hasher);
    if let Some(error) = self.first_error() {
      error.message.hash(&mut hasher);
    } else if let Some(s) = &self.source {
      std::hash::Hash::hash(s, &mut hasher);
    }
    "meta".hash(&mut hasher);
    build_meta.hash(&mut hasher);
    hasher.digest(&output_options.hash_digest)
  }

  pub fn get_parser_options(&self) -> Option<&ParserOptions> {
    self.parser_and_generator_options.parser_options()
  }

  pub fn get_generator_options(&self) -> Option<&GeneratorOptions> {
    self.parser_and_generator_options.generator_options()
  }

  pub fn asset_filename_override(&self) -> Option<crate::Filename> {
    self
      .asset_filename_override
      .read()
      .expect("asset filename lock poisoned")
      .clone()
  }

  pub fn set_asset_filename_override(&self, filename: crate::Filename) {
    *self
      .asset_filename_override
      .write()
      .expect("asset filename lock poisoned") = Some(filename);
  }

  async fn create_cache_snapshot(
    &self,
    file_system_info: &FileSystemInfo,
    build_start_time: u64,
  ) -> Result<Option<Snapshot>> {
    if !self.build_info.read().cacheable
      || self
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.is_error())
    {
      return Ok(None);
    }

    let build_info = self.build_info.read();
    Ok(Some(
      file_system_info
        .create_snapshot(
          Some(build_start_time),
          &build_info.dependencies.file,
          &build_info.dependencies.context,
          &build_info.dependencies.missing,
          // Rspack does not expose webpack's `snapshot.module` strategy yet.
          SnapshotStrategyOptions::timestamp(),
        )
        .await?,
    ))
  }
}

impl Identifiable for NormalModule {
  #[inline]
  fn identifier(&self) -> ModuleIdentifier {
    self.id
  }
}

impl DependenciesBlock for NormalModule {
  fn dependencies_block(&self) -> &DependenciesBlockData {
    &self.dependencies_block
  }

  fn dependencies_block_mut(&mut self) -> &mut DependenciesBlockData {
    &mut self.dependencies_block
  }
}

#[cacheable_dyn]
#[async_trait::async_trait]
impl Module for NormalModule {
  fn module_type(&self) -> &ModuleType {
    &self.module_type
  }

  fn source_types(&self, module_graph: &ModuleGraph) -> &[SourceType] {
    self.parser_and_generator.source_types(self, module_graph)
  }

  fn source(&self) -> Option<&BoxSource> {
    self.source.as_ref()
  }

  fn readable_identifier(&self, context: &Context) -> Cow<'_, str> {
    Cow::Owned(context.shorten(&self.user_request))
  }

  fn size(&self, source_type: Option<&SourceType>, _compilation: Option<&Compilation>) -> f64 {
    if let Some(source_type) = source_type {
      // Fast-path: lock-free builtin slots / tiny fallback map for custom source types.
      if let Some(size) = self.cached_source_sizes.get(source_type) {
        return size;
      }

      let size = f64::max(1.0, self.parser_and_generator.size(self, Some(source_type)));
      self.cached_source_sizes.get_or_insert(source_type, size)
    } else {
      f64::max(1.0, self.parser_and_generator.size(self, source_type))
    }
  }

  async fn need_build(&self, context: &NeedBuildContext<'_>) -> Result<bool> {
    if self.force_build {
      return Ok(true);
    }

    if self
      .diagnostics
      .iter()
      .any(|diagnostic| diagnostic.is_error())
    {
      return Ok(true);
    }

    let Some(build_info) = self.build_info.frozen() else {
      return Ok(true);
    };
    if !build_info.cacheable {
      return Ok(true);
    }

    let Some(snapshot) = &build_info.snapshot else {
      return Ok(true);
    };

    if context
      .value_cache_versions
      .has_diff(&build_info.value_dependencies)
    {
      return Ok(true);
    }

    Ok(matches!(
      context
        .file_system_info
        .check_snapshot_valid(snapshot)
        .await?,
      SnapshotValidationResult::Invalid { .. }
    ))
  }

  async fn prepare_for_cache(
    &self,
    file_system_info: &FileSystemInfo,
    build_start_time: u64,
  ) -> Result<()> {
    let snapshot = self
      .create_cache_snapshot(file_system_info, build_start_time)
      .await?;
    self
      .build_info
      .update(|build_info| build_info.snapshot = snapshot);
    Ok(())
  }

  #[tracing::instrument("NormalModule:build", skip_all, fields(
    perfetto.track_name = format!("Module Build"),
    perfetto.process_name = format!("Rspack Build Detail"),
    module.resource = self.resource_resolved_data().resource(),
    module.identifier = self.identifier().as_str(),
    module.loaders = ?self.loaders.loaders().iter().map(|l| l.loader.identifier().as_str()).collect::<Vec<_>>())
  )]
  async fn build(
    mut self: Box<Self>,
    build_context: BuildContext,
    _compilation: Option<&Compilation>,
  ) -> Result<BoxModule> {
    self.dependencies_block = Default::default();
    self.force_build = false;
    self.build_info.get_mut().snapshot = None;
    self.build_info.get_mut().optimization_bailouts.clear();

    // so does webpack
    self.parsed = true;

    let no_parse = if let Some(no_parse) = build_context.compiler_options.module.no_parse.as_ref() {
      no_parse.try_match(self.request.as_str()).await?
    } else {
      false
    };

    build_context
      .plugin_driver
      .normal_module_hooks
      .before_loaders
      .call(self.as_mut())
      .await?;

    let plugin = Arc::new(RspackLoaderRunnerPlugin {
      plugin_driver: build_context.plugin_driver.clone(),
      extract_source_map: self.extract_source_map,
    });

    let compiler_id = build_context.compiler_id;
    let compilation_id = build_context.compilation_id;
    let compiler_options = build_context.compiler_options.clone();
    let resolver_factory = build_context.resolver_factory.clone();
    let fs = build_context.fs.clone();
    let (mut loader_result, err) = run_loaders(
      self.resource_data.clone(),
      Some(plugin.clone()),
      RunnerContext {
        compiler_id,
        compilation_id,
        options: compiler_options,
        fs: fs.clone(),
        loader_cache: build_context.loader_cache,
        file_system_info: build_context.file_system_info,
        resolver_factory,
        source_map_kind: self.source_map_kind,
        loader_context_data: Default::default(),
        module: self,
      },
      fs,
    )
    .instrument(info_span!("NormalModule:run_loaders",))
    .await;
    drop(loader_result.context.loader_context_data);
    self = loader_result.context.module;

    if let Some(err) = err {
      self.build_info.get_mut().cacheable = loader_result.cacheable;
      self.build_info.get_mut().dependencies = loader_result.dependencies;

      self.source = None;

      let current_loader = loader_result.current_loader.map(|current_loader| {
        contextify(
          build_context.compiler_options.context.as_path(),
          current_loader.as_str(),
        )
      });
      let diagnostic = Diagnostic::from(rspack_error::Error::from(ModuleBuildError::new(
        err,
        current_loader,
      )));
      self.diagnostics.push(diagnostic);

      self.build_info.get_mut().hash = Some(self.init_build_hash(
        &build_context.compiler_options.output,
        &self.build_meta.read(),
      ));
      return Ok(BoxModule::new(self));
    };

    build_context
      .plugin_driver
      .normal_module_hooks
      .additional_data
      .call(&mut loader_result.additional_data.as_mut())
      .await?;
    self.add_diagnostics(loader_result.diagnostics);

    let is_binary = self
      .get_generator_options()
      .and_then(|g| match g {
        GeneratorOptions::Asset(g) => g.binary,
        GeneratorOptions::AssetInline(g) => g.binary,
        GeneratorOptions::AssetResource(g) => g.binary,
        _ => None,
      })
      .unwrap_or_else(|| self.module_type.is_binary());

    let content = if is_binary {
      Content::Buffer(loader_result.content.into_bytes())
    } else {
      Content::String(loader_result.content.into_string_lossy())
    };
    let source = self.create_source(
      content,
      loader_result.source_map.map(|source_map| *source_map),
    )?;

    self.build_info.get_mut().cacheable = loader_result.cacheable;
    self.build_info.get_mut().dependencies = loader_result.dependencies;

    if no_parse {
      self.parsed = false;
      self.source = Some(source);
      self.code_generation_dependencies = Some(Vec::new());
      self.presentational_dependencies = Some(Vec::new());

      self.build_info.get_mut().hash = Some(self.init_build_hash(
        &build_context.compiler_options.output,
        &self.build_meta.read(),
      ));

      return Ok(BoxModule::new(self));
    }

    let factory_meta = self.factory_meta.get();
    let (
      ParseResult {
        source,
        dependencies,
        blocks,
        presentational_dependencies,
        code_generation_dependencies,
        side_effects_bailout,
      },
      diagnostics,
    ) = self
      .parser_and_generator
      .parse(ParseContext {
        source: source.clone(),
        module_context: &self.context,
        module_identifier: self.id,
        module_parser_options: self.parser_and_generator_options.parser_options(),
        module_generator_options: self.parser_and_generator_options.generator_options(),
        module_type: &self.module_type,
        module_layer: self.layer.as_ref(),
        module_user_request: &self.user_request,
        module_match_resource: self.match_resource.as_ref(),
        module_source_map_kind: self.source_map_kind,
        loaders: self.loaders.loaders(),
        resource_data: &self.resource_data,
        compiler_options: &build_context.compiler_options,
        additional_data: loader_result.additional_data,
        factory_meta: factory_meta.as_deref(),
        build_info: self.build_info.get_mut(),
        build_meta: self.build_meta.get_mut(),
        parse_meta: loader_result.parse_meta,
        runtime_template: &build_context.runtime_template,
      })
      .await?
      .split_into_parts();
    if diagnostics.iter().any(|d| d.is_error()) {
      self.build_meta = Default::default();
    }
    if !diagnostics.is_empty() {
      self.add_diagnostics(diagnostics);
    }
    if let Some(side_effects_bailout) = side_effects_bailout {
      let short_id = self
        .readable_identifier(&build_context.compiler_options.context)
        .to_string();
      self
        .build_info
        .get_mut()
        .optimization_bailouts
        .push(OptimizationBailoutItem::SideEffects {
          node_type: side_effects_bailout.ty,
          loc: side_effects_bailout.msg,
          short_id,
        });
    }
    // Only side effects used in code_generate can stay here
    // Other side effects should be set outside use_cache
    self.source = Some(source);
    self.code_generation_dependencies = Some(code_generation_dependencies);
    self.presentational_dependencies = Some(presentational_dependencies);

    self.build_info.get_mut().hash = Some(self.init_build_hash(
      &build_context.compiler_options.output,
      &self.build_meta.read(),
    ));

    Ok(BoxModule::new(self).with_dependencies(
      dependencies.into_iter().map(Into::into).collect(),
      blocks.into_iter().map(Into::into).collect(),
    ))
  }

  // #[tracing::instrument("NormalModule::code_generation", skip_all, fields(identifier = ?self.identifier()))]
  async fn code_generation(
    &self,
    code_generation_context: &mut ModuleCodeGenerationContext,
  ) -> Result<CodeGenerationResultBuilder> {
    let ModuleCodeGenerationContext {
      compilation,
      runtime,
      concatenation_scope,
      runtime_template,
    } = code_generation_context;

    if let Some(error) = self.first_error() {
      let mut code_generation_result = CodeGenerationResultBuilder::default();
      let module_graph = compilation.get_module_graph();

      // If the module build failed and the module is able to emit JavaScript source,
      // we should emit an error message to the runtime, otherwise we do nothing.
      if self
        .source_types(module_graph)
        .contains(&SourceType::JavaScript)
      {
        let error = error.render_report(compilation.options.stats.colors)?;
        code_generation_result.add(
          SourceType::JavaScript,
          RawStringSource::from(format!("throw new Error({});\n", json!(error))).boxed(),
        );
      }
      return Ok(code_generation_result);
    }
    let Some(source) = &self.source else {
      return Err(error!(
        "Failed to generate code because ast or source is not set for module {}",
        self.request
      ));
    };

    let mut code_generation_result = CodeGenerationResultBuilder::default();
    if !self.parsed {
      runtime_template
        .runtime_requirements_mut()
        .insert(RuntimeGlobals::MODULE | RuntimeGlobals::EXPORTS | RuntimeGlobals::THIS_AS_EXPORTS);
    }

    let module_graph = compilation.get_module_graph();
    for source_type in self.source_types(module_graph) {
      let in_concatenation = concatenation_scope.is_some();
      let generation_result = self
        .parser_and_generator
        .generate(
          source,
          self,
          &mut GenerateContext {
            compilation,
            runtime_template,
            data: code_generation_result.data_mut(),
            requested_source_type: *source_type,
            module_parser_options: self.parser_and_generator_options.parser_options(),
            module_generator_options: self.parser_and_generator_options.generator_options(),
            runtime: *runtime,
            concatenation_scope: concatenation_scope.as_mut(),
          },
        )
        .await?;
      if in_concatenation {
        code_generation_result.add(*source_type, generation_result);
      } else {
        code_generation_result.add(*source_type, CachedSource::new(generation_result).boxed());
      }
    }
    Ok(code_generation_result)
  }

  async fn get_runtime_hash(
    &self,
    compilation: &Compilation,
    runtime: Option<&RuntimeSpec>,
  ) -> Result<RspackHashDigest> {
    let mut hasher = RspackHasher::from(&compilation.options.output);
    self.build_info.read().hash.hash(&mut hasher);
    // For built failed NormalModule, hash will be calculated by build_info.hash, which contains error message
    if self.source.is_some() && self.parser_and_generator.has_runtime_hash() {
      let runtime_hash = self
        .parser_and_generator
        .get_runtime_hash(self, compilation, runtime)
        .await?;
      runtime_hash.hash(&mut hasher);
    }
    module_update_hash(self, &mut hasher, compilation, runtime);
    Ok(hasher.digest(&compilation.options.output.hash_digest))
  }

  fn name_for_condition(&self) -> Option<Box<str>> {
    // Align with https://github.com/webpack/webpack/blob/8241da7f1e75c5581ba535d127fa66aeb9eb2ac8/lib/NormalModule.js#L375
    let resource = self
      .match_resource()
      .unwrap_or_else(|| &self.resource_data)
      .resource();
    Some(
      rspack_util::identifier::split_at_query_mark(resource)
        .0
        .into(),
    )
  }

  fn lib_ident(&self, options: LibIdentOptions) -> Option<Cow<'_, str>> {
    let mut ident = String::new();
    if let Some(layer) = &self.layer {
      ident += "(";
      ident += layer;
      ident += ")/";
    }
    ident += &contextify(options.context, self.user_request());
    Some(Cow::Owned(ident))
  }

  fn get_resolve_options(&self) -> Option<Arc<Resolve>> {
    self.resolve_options.clone()
  }

  fn get_code_generation_dependencies(&self) -> Option<&[DependencyId]> {
    if let Some(deps) = self.code_generation_dependencies.as_deref()
      && !deps.is_empty()
    {
      Some(deps)
    } else {
      None
    }
  }

  fn get_presentational_dependencies(&self) -> Option<&[DependencyCodeGenerationRef]> {
    if let Some(deps) = self.presentational_dependencies.as_deref()
      && !deps.is_empty()
    {
      Some(deps)
    } else {
      None
    }
  }

  fn get_context(&self) -> Option<Box<Context>> {
    Some(self.context.clone())
  }

  fn get_layer(&self) -> Option<&ModuleLayer> {
    self.layer.as_ref()
  }

  // Port from https://github.com/webpack/webpack/blob/main/lib/NormalModule.js#L1120
  fn get_side_effects_connection_state(
    &self,
    module_graph: &ModuleGraph,
    module_graph_cache: &ModuleGraphCacheArtifact,
    side_effects_state_artifact: &SideEffectsStateArtifact,
    module_chain: &mut IdentifierSet,
    connection_state_cache: &mut IdentifierMap<ConnectionState>,
  ) -> ConnectionState {
    module_graph_cache.cached_get_side_effects_connection_state(self.id(), || {
      if let Some(state) = connection_state_cache.get(&self.id) {
        return *state;
      }

      if let Some(side_effect_free) = module_declared_side_effect_free(self) {
        return ConnectionState::Active(!side_effect_free);
      }

      if module_analyzed_side_effect_free(self, side_effects_state_artifact) == Some(true) {
        // use module chain instead of is_evaluating_side_effects to mut module graph
        if module_chain.contains(&self.identifier()) {
          return ConnectionState::CircularConnection;
        }
        module_chain.insert(self.identifier());
        let mut current = ConnectionState::Active(false);
        for dependency in self.get_dependencies() {
          let state = dependency.get_module_evaluation_side_effects_state(
            module_graph,
            module_graph_cache,
            side_effects_state_artifact,
            module_chain,
            connection_state_cache,
          );
          if matches!(state, ConnectionState::Active(true)) {
            // TODO add optimization bailout
            module_chain.remove(&self.identifier());
            connection_state_cache.insert(self.id, ConnectionState::Active(true));
            return ConnectionState::Active(true);
          } else if !matches!(state, ConnectionState::CircularConnection) {
            current = current + state;
          }
        }
        module_chain.remove(&self.identifier());
        connection_state_cache.insert(self.id, current);
        return current;
      }
      ConnectionState::Active(true)
    })
  }

  fn get_concatenation_bailout_reason(
    &self,
    mg: &ModuleGraph,
    cg: &ChunkGraph,
  ) -> Option<Cow<'static, str>> {
    self
      .parser_and_generator
      .get_concatenation_bailout_reason(self, mg, cg)
  }

  fn factory_meta(&self) -> Option<Arc<FactoryMeta>> {
    self.factory_meta.get()
  }

  fn set_factory_meta(&self, factory_meta: FactoryMeta) {
    self.factory_meta.set(Some(Arc::new(factory_meta)));
  }

  fn reset_for_compilation(&self, factory_meta: Option<Arc<FactoryMeta>>) {
    self.factory_meta.set(factory_meta);
    *self
      .asset_filename_override
      .write()
      .expect("asset filename lock poisoned") = None;
  }

  fn build_info(&self) -> crate::FreezeReadGuard<'_, BuildInfo> {
    self.build_info.read()
  }

  fn freeze_build_info(&self) {
    self.build_info.freeze();
  }

  fn extend_build_assets(&self, assets: crate::CompilationAssets) {
    self.build_info.extend_assets(assets);
  }

  fn build_info_mut(&mut self) -> &mut BuildInfo {
    self.build_info.get_mut()
  }

  fn build_meta(&self) -> crate::FreezeReadGuard<'_, BuildMeta> {
    self.build_meta.read()
  }

  fn freeze_build_meta(&self) -> &triomphe::Arc<BuildMeta> {
    self.build_meta.freeze()
  }
}

impl ModuleSourceMapConfig for NormalModule {
  fn get_source_map_kind(&self) -> &SourceMapKind {
    &self.source_map_kind
  }

  fn set_source_map_kind(&mut self, source_map_kind: SourceMapKind) {
    self.source_map_kind = source_map_kind;
  }
}

impl Diagnosable for NormalModule {
  fn add_diagnostic(&mut self, diagnostic: Diagnostic) {
    self.diagnostics.push(diagnostic);
  }

  fn add_diagnostics(&mut self, mut diagnostics: Vec<Diagnostic>) {
    self.diagnostics.append(&mut diagnostics);
  }

  fn diagnostics(&self) -> Cow<'_, [Diagnostic]> {
    Cow::Borrowed(&self.diagnostics)
  }
}

impl NormalModule {
  fn create_source(
    &self,
    content: Content,
    source_map: Option<SourceMap<'static>>,
  ) -> Result<BoxSource> {
    if content.is_buffer() {
      return Ok(RawBufferSource::from(content.into_bytes()).boxed());
    }
    let source_map_kind = self.get_source_map_kind();
    if source_map_kind.enabled()
      && let Some(source_map) = source_map
    {
      let content = content.into_string_lossy();
      return Ok(
        SourceMapSource::new(WithoutOriginalOptions {
          value: content,
          name: self.request(),
          source_map,
        })
        .boxed(),
      );
    }
    if source_map_kind.enabled()
      && let Content::String(content) = content
    {
      return Ok(OriginalSource::new(content, self.request()).boxed());
    }
    Ok(RawStringSource::from(content.into_string_lossy()).boxed())
  }
}
