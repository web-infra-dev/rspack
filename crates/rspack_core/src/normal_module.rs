use std::{
  borrow::Cow,
  hash::{BuildHasherDefault, Hash},
  sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
  },
};

use dashmap::DashMap;
use derive_more::Debug;
use futures::future::BoxFuture;
use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsMap, AsOption, AsPreset},
};
use rspack_collections::{Identifiable, IdentifierMap, IdentifierSet};
use rspack_error::{Diagnosable, Diagnostic, Result, error};
use rspack_fs::ReadableFileSystem;
use rspack_hash::{RspackHash, RspackHashDigest};
use rspack_hook::define_hook;
use rspack_loader_runner::{AdditionalData, Content, LoaderContext, ResourceData, run_loaders};
use rspack_sources::{
  BoxSource, CachedSource, OriginalSource, RawBufferSource, RawStringSource, SourceExt, SourceMap,
  SourceMapSource, WithoutOriginalOptions,
};
use rspack_util::{
  ext::DynHash,
  source_map::{ModuleSourceMapConfig, SourceMapKind},
};
use rustc_hash::FxHasher;
use serde_json::json;
use tracing::{Instrument, info_span};

use crate::{
  AsyncDependenciesBlockIdentifier, BoxDependencyTemplate, BoxLoader, BoxModule,
  BoxModuleDependency, BuildContext, BuildInfo, BuildMeta, BuildResult, ChunkGraph,
  CodeGenerationResult, Compilation, ConcatenationScope, ConnectionState, Context,
  DependenciesBlock, DependencyId, FactoryMeta, GenerateContext, GeneratorOptions, LibIdentOptions,
  Module, ModuleGraph, ModuleGraphCacheArtifact, ModuleIdentifier, ModuleLayer, ModuleType,
  OutputOptions, ParseContext, ParseResult, ParserAndGenerator, ParserOptions, Resolve,
  RspackLoaderRunnerPlugin, RunnerContext, RuntimeGlobals, RuntimeSpec, SourceType, contextify,
  diagnostics::ModuleBuildError, get_context, module_update_hash,
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

  pub fn get_module<'a>(&self, module_graph: &'a ModuleGraph) -> Option<&'a BoxModule> {
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
define_hook!(NormalModuleLoaderShouldYield: SeriesBail(loader_context: &LoaderContext<RunnerContext>) -> bool,tracing=false);
define_hook!(NormalModuleLoaderStartYielding: Series(loader_context: &mut LoaderContext<RunnerContext>),tracing=false);
define_hook!(NormalModuleBeforeLoaders: Series(module: &mut NormalModule),tracing=false);
define_hook!(NormalModuleAdditionalData: Series(additional_data: &mut Option<&mut AdditionalData>),tracing=false);

#[derive(Debug, Default)]
pub struct NormalModuleHooks {
  pub read_resource: NormalModuleReadResourceHook,
  pub loader: NormalModuleLoaderHook,
  pub loader_should_yield: NormalModuleLoaderShouldYieldHook,
  pub loader_yield: NormalModuleLoaderStartYieldingHook,
  pub before_loaders: NormalModuleBeforeLoadersHook,
  pub additional_data: NormalModuleAdditionalDataHook,
}

#[cacheable]
#[derive(Debug)]
pub struct NormalModuleInner {
  blocks: Vec<AsyncDependenciesBlockIdentifier>,
  dependencies: Vec<DependencyId>,

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
  loaders: Vec<BoxLoader>,

  /// Built source of this module (passed with loaders)
  #[cacheable(with=AsOption<AsPreset>)]
  source: Option<BoxSource>,

  /// Resolve options derived from [Rule.resolve]
  resolve_options: Option<Arc<Resolve>>,
  /// Parser options derived from [Rule.parser]
  parser_options: Option<ParserOptions>,
  /// Generator options derived from [Rule.generator]
  generator_options: Option<GeneratorOptions>,
  /// enable/disable extracting source map
  extract_source_map: Option<bool>,

  #[allow(unused)]
  debug_id: usize,
  #[cacheable(with=AsMap)]
  cached_source_sizes: DashMap<SourceType, f64, BuildHasherDefault<FxHasher>>,
  diagnostics: Vec<Diagnostic>,

  code_generation_dependencies: Option<Vec<BoxModuleDependency>>,
  presentational_dependencies: Option<Vec<BoxDependencyTemplate>>,

  factory_meta: Option<FactoryMeta>,
  build_info: BuildInfo,
  build_meta: BuildMeta,
  parsed: bool,

  source_map_kind: SourceMapKind,
}

#[cacheable]
#[derive(Debug)]
pub enum NormalModule {
  Owned(Box<NormalModuleInner>),
  Transferred,
}

static DEBUG_ID: AtomicUsize = AtomicUsize::new(1);

impl NormalModule {
  #[inline]
  fn inner(&self) -> &NormalModuleInner {
    match self {
      NormalModule::Owned(inner) => inner,
      NormalModule::Transferred => {
        unreachable!("NormalModule ownership has been transferred to loader execution context")
      }
    }
  }

  #[inline]
  fn inner_mut(&mut self) -> &mut NormalModuleInner {
    match self {
      NormalModule::Owned(inner) => inner,
      NormalModule::Transferred => {
        unreachable!("NormalModule ownership has been transferred to loader execution context")
      }
    }
  }

  #[inline]
  async fn with_ownership<R>(
    &mut self,
    operation: impl FnOnce(Self) -> BoxFuture<'static, R>,
    extract_module: impl FnOnce(&mut R) -> Self,
  ) -> R {
    let module = std::mem::replace(self, NormalModule::Transferred);
    let mut result = operation(module).await;
    *self = extract_module(&mut result);
    result
  }

  fn create_id<'request>(
    module_type: &ModuleType,
    layer: Option<&ModuleLayer>,
    request: &'request str,
  ) -> Cow<'request, str> {
    if let Some(layer) = layer {
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
    parser_options: Option<ParserOptions>,
    generator_options: Option<GeneratorOptions>,
    match_resource: Option<ResourceData>,
    resource_data: Arc<ResourceData>,
    resolve_options: Option<Arc<Resolve>>,
    loaders: Vec<BoxLoader>,
    context: Option<Context>,
    extract_source_map: Option<bool>,
  ) -> Self {
    let module_type = module_type.into();
    let id = Self::create_id(&module_type, layer.as_ref(), &request);
    Self::Owned(Box::new(NormalModuleInner {
      blocks: Vec::new(),
      dependencies: Vec::new(),
      id: ModuleIdentifier::from(id.as_ref()),
      context: Box::new(context.unwrap_or_else(|| get_context(&resource_data))),
      request,
      user_request,
      raw_request,
      module_type,
      layer,
      parser_and_generator,
      parser_options,
      generator_options,
      match_resource,
      resource_data,
      resolve_options,
      loaders,
      source: None,
      debug_id: DEBUG_ID.fetch_add(1, Ordering::Relaxed),
      extract_source_map,

      cached_source_sizes: DashMap::default(),
      diagnostics: Default::default(),
      code_generation_dependencies: None,
      presentational_dependencies: None,
      factory_meta: None,
      build_info: Default::default(),
      build_meta: Default::default(),
      parsed: false,
      source_map_kind: SourceMapKind::empty(),
    }))
  }

  pub fn id(&self) -> ModuleIdentifier {
    self.inner().id
  }

  pub fn match_resource(&self) -> Option<&ResourceData> {
    self.inner().match_resource.as_ref()
  }

  pub fn match_resource_mut(&mut self) -> &mut Option<ResourceData> {
    &mut self.inner_mut().match_resource
  }

  pub fn resource_resolved_data(&self) -> &Arc<ResourceData> {
    &self.inner().resource_data
  }

  pub fn request(&self) -> &str {
    &self.inner().request
  }

  pub fn user_request(&self) -> &str {
    &self.inner().user_request
  }

  pub fn user_request_mut(&mut self) -> &mut String {
    &mut self.inner_mut().user_request
  }

  pub fn raw_request(&self) -> &str {
    &self.inner().raw_request
  }

  pub fn loaders(&self) -> &[BoxLoader] {
    &self.inner().loaders
  }

  pub fn parser_and_generator(&self) -> &dyn ParserAndGenerator {
    &*self.inner().parser_and_generator
  }

  pub fn parser_and_generator_mut(&mut self) -> &mut Box<dyn ParserAndGenerator> {
    &mut self.inner_mut().parser_and_generator
  }

  pub fn code_generation_dependencies(&self) -> &Option<Vec<BoxModuleDependency>> {
    &self.inner().code_generation_dependencies
  }

  pub fn code_generation_dependencies_mut(&mut self) -> &mut Option<Vec<BoxModuleDependency>> {
    &mut self.inner_mut().code_generation_dependencies
  }

  pub fn presentational_dependencies(&self) -> &Option<Vec<BoxDependencyTemplate>> {
    &self.inner().presentational_dependencies
  }

  pub fn presentational_dependencies_mut(&mut self) -> &mut Option<Vec<BoxDependencyTemplate>> {
    &mut self.inner_mut().presentational_dependencies
  }

  #[tracing::instrument(
    "NormalModule:build_hash", skip_all,fields(
      resource = self.inner().resource_data.resource()
    )
  )]
  fn init_build_hash(
    &self,
    output_options: &OutputOptions,
    build_meta: &BuildMeta,
  ) -> RspackHashDigest {
    let mut hasher = RspackHash::from(output_options);
    "source".hash(&mut hasher);
    if let Some(error) = self.first_error() {
      error.message.hash(&mut hasher);
    } else if let Some(s) = &self.inner().source {
      s.hash(&mut hasher);
    }
    "meta".hash(&mut hasher);
    build_meta.hash(&mut hasher);
    hasher.digest(&output_options.hash_digest)
  }

  pub fn get_parser_options(&self) -> Option<&ParserOptions> {
    self.inner().parser_options.as_ref()
  }

  pub fn get_generator_options(&self) -> Option<&GeneratorOptions> {
    self.inner().generator_options.as_ref()
  }
}

impl Identifiable for NormalModule {
  #[inline]
  fn identifier(&self) -> ModuleIdentifier {
    self.inner().id
  }
}

impl DependenciesBlock for NormalModule {
  fn add_block_id(&mut self, block: AsyncDependenciesBlockIdentifier) {
    self.inner_mut().blocks.push(block)
  }

  fn get_blocks(&self) -> &[AsyncDependenciesBlockIdentifier] {
    &self.inner().blocks
  }

  fn add_dependency_id(&mut self, dependency: DependencyId) {
    self.inner_mut().dependencies.push(dependency)
  }

  fn remove_dependency_id(&mut self, dependency: DependencyId) {
    self.inner_mut().dependencies.retain(|d| d != &dependency)
  }

  fn get_dependencies(&self) -> &[DependencyId] {
    &self.inner().dependencies
  }
}

#[cacheable_dyn]
#[async_trait::async_trait]
impl Module for NormalModule {
  fn module_type(&self) -> &ModuleType {
    &self.inner().module_type
  }

  fn source_types(&self, module_graph: &ModuleGraph) -> &[SourceType] {
    self
      .inner()
      .parser_and_generator
      .source_types(self, module_graph)
  }

  fn source(&self) -> Option<&BoxSource> {
    self.inner().source.as_ref()
  }

  fn readable_identifier(&self, context: &Context) -> Cow<'_, str> {
    Cow::Owned(context.shorten(&self.inner().user_request))
  }

  fn size(&self, source_type: Option<&SourceType>, _compilation: Option<&Compilation>) -> f64 {
    if let Some(size_ref) = source_type.and_then(|st| self.inner().cached_source_sizes.get(st)) {
      *size_ref
    } else {
      let size = f64::max(
        1.0,
        self.inner().parser_and_generator.size(self, source_type),
      );
      source_type.and_then(|st| self.inner().cached_source_sizes.insert(*st, size));
      size
    }
  }

  #[tracing::instrument("NormalModule:build", skip_all, fields(
    perfetto.track_name = format!("Module Build"),
    perfetto.process_name = format!("Rspack Build Detail"),
    module.resource = self.resource_resolved_data().resource(),
    module.identifier = self.identifier().as_str(),
    module.loaders = ?self.inner().loaders.iter().map(|l| l.identifier().as_str()).collect::<Vec<_>>())
  )]
  async fn build(
    &mut self,
    build_context: BuildContext,
    _compilation: Option<&Compilation>,
  ) -> Result<BuildResult> {
    let inner = self.inner_mut();
    // so does webpack
    inner.parsed = true;

    let no_parse = if let Some(no_parse) = build_context.compiler_options.module.no_parse.as_ref() {
      no_parse.try_match(inner.request.as_str()).await?
    } else {
      false
    };

    build_context
      .plugin_driver
      .normal_module_hooks
      .before_loaders
      .call(self)
      .await?;

    let plugin = Arc::new(RspackLoaderRunnerPlugin {
      plugin_driver: build_context.plugin_driver.clone(),
      extract_source_map: self.inner().extract_source_map,
    });

    let compiler_id = build_context.compiler_id;
    let compilation_id = build_context.compilation_id;
    let compiler_options = build_context.compiler_options.clone();
    let resolver_factory = build_context.resolver_factory.clone();
    let fs = build_context.fs.clone();
    let (mut loader_result, err) = self
      .with_ownership(
        |mut module| {
          Box::pin(async move {
            let inner = module.inner_mut();
            let (loader_result, err) = run_loaders(
              inner.loaders.clone(),
              inner.resource_data.clone(),
              Some(plugin.clone()),
              RunnerContext {
                compiler_id,
                compilation_id,
                options: compiler_options,
                resolver_factory,
                source_map_kind: inner.source_map_kind,
                module,
              },
              fs,
            )
            .instrument(info_span!("NormalModule:run_loaders",))
            .await;
            (loader_result, err)
          })
        },
        |(loader_result, _)| {
          std::mem::replace(&mut loader_result.context.module, NormalModule::Transferred)
        },
      )
      .await;

    let inner = self.inner_mut();
    if let Some(err) = err {
      inner.build_info.cacheable = loader_result.cacheable;
      inner.build_info.file_dependencies = loader_result
        .file_dependencies
        .into_iter()
        .map(Into::into)
        .collect();
      inner.build_info.context_dependencies = loader_result
        .context_dependencies
        .into_iter()
        .map(Into::into)
        .collect();
      inner.build_info.missing_dependencies = loader_result
        .missing_dependencies
        .into_iter()
        .map(Into::into)
        .collect();
      inner.build_info.build_dependencies = loader_result
        .build_dependencies
        .into_iter()
        .map(Into::into)
        .collect();

      inner.source = None;

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
      inner.diagnostics.push(diagnostic);

      self.inner_mut().build_info.hash = Some(self.init_build_hash(
        &build_context.compiler_options.output,
        &self.inner().build_meta,
      ));
      return Ok(BuildResult {
        dependencies: Vec::new(),
        blocks: Vec::new(),
        optimization_bailouts: vec![],
      });
    };

    build_context
      .plugin_driver
      .normal_module_hooks
      .additional_data
      .call(&mut loader_result.additional_data.as_mut())
      .await?;
    self.add_diagnostics(loader_result.diagnostics);

    let inner = self.inner_mut();
    let is_binary = inner
      .generator_options
      .as_ref()
      .and_then(|g| match g {
        GeneratorOptions::Asset(g) => g.binary,
        GeneratorOptions::AssetInline(g) => g.binary,
        GeneratorOptions::AssetResource(g) => g.binary,
        _ => None,
      })
      .unwrap_or(inner.module_type.is_binary());

    let content = if is_binary {
      Content::Buffer(loader_result.content.into_bytes())
    } else {
      Content::String(loader_result.content.into_string_lossy())
    };
    let source = self.create_source(content, loader_result.source_map)?;

    let inner = self.inner_mut();
    inner.build_info.cacheable = loader_result.cacheable;
    inner.build_info.file_dependencies = loader_result
      .file_dependencies
      .into_iter()
      .map(Into::into)
      .collect();
    inner.build_info.context_dependencies = loader_result
      .context_dependencies
      .into_iter()
      .map(Into::into)
      .collect();
    inner.build_info.missing_dependencies = loader_result
      .missing_dependencies
      .into_iter()
      .map(Into::into)
      .collect();
    inner.build_info.build_dependencies = loader_result
      .build_dependencies
      .into_iter()
      .map(Into::into)
      .collect();

    if no_parse {
      inner.parsed = false;
      inner.source = Some(source);
      inner.code_generation_dependencies = Some(Vec::new());
      inner.presentational_dependencies = Some(Vec::new());

      self.inner_mut().build_info.hash = Some(self.init_build_hash(
        &build_context.compiler_options.output,
        &self.inner().build_meta,
      ));

      return Ok(BuildResult {
        dependencies: Vec::new(),
        blocks: Vec::new(),
        optimization_bailouts: Vec::new(),
      });
    }

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
    ) = inner
      .parser_and_generator
      .parse(ParseContext {
        source: source.clone(),
        module_context: &inner.context,
        module_identifier: inner.id,
        module_parser_options: inner.parser_options.as_ref(),
        module_type: &inner.module_type,
        module_layer: inner.layer.as_ref(),
        module_user_request: &inner.user_request,
        module_match_resource: inner.match_resource.as_ref(),
        module_source_map_kind: inner.source_map_kind,
        loaders: &inner.loaders,
        resource_data: &inner.resource_data,
        compiler_options: &build_context.compiler_options,
        additional_data: loader_result.additional_data,
        factory_meta: inner.factory_meta.as_ref(),
        build_info: &mut inner.build_info,
        build_meta: &mut inner.build_meta,
        parse_meta: loader_result.parse_meta,
        runtime_template: &build_context.runtime_template,
      })
      .await?
      .split_into_parts();
    if diagnostics.iter().any(|d| d.is_error()) {
      inner.build_meta = Default::default();
    }
    if !diagnostics.is_empty() {
      self.add_diagnostics(diagnostics);
    }
    let optimization_bailouts = if let Some(side_effects_bailout) = side_effects_bailout {
      let short_id = self.readable_identifier(&build_context.compiler_options.context);
      vec![format!(
        "{} with side_effects in source code at {short_id}:{}",
        side_effects_bailout.ty, side_effects_bailout.msg
      )]
    } else {
      vec![]
    };
    // Only side effects used in code_generate can stay here
    // Other side effects should be set outside use_cache
    let inner = self.inner_mut();
    inner.source = Some(source);
    inner.code_generation_dependencies = Some(code_generation_dependencies);
    inner.presentational_dependencies = Some(presentational_dependencies);

    self.inner_mut().build_info.hash = Some(self.init_build_hash(
      &build_context.compiler_options.output,
      &self.inner().build_meta,
    ));

    Ok(BuildResult {
      dependencies,
      blocks,
      optimization_bailouts,
    })
  }

  // #[tracing::instrument("NormalModule::code_generation", skip_all, fields(identifier = ?self.identifier()))]
  async fn code_generation(
    &self,
    compilation: &Compilation,
    runtime: Option<&RuntimeSpec>,
    mut concatenation_scope: Option<ConcatenationScope>,
  ) -> Result<CodeGenerationResult> {
    if let Some(error) = self.first_error() {
      let mut code_generation_result = CodeGenerationResult::default();
      let module_graph = compilation.get_module_graph();

      // If the module build failed and the module is able to emit JavaScript source,
      // we should emit an error message to the runtime, otherwise we do nothing.
      if self
        .source_types(&module_graph)
        .contains(&SourceType::JavaScript)
      {
        let error = error.render_report(compilation.options.stats.colors)?;
        code_generation_result.add(
          SourceType::JavaScript,
          RawStringSource::from(format!("throw new Error({});\n", json!(error))).boxed(),
        );
        code_generation_result.concatenation_scope = concatenation_scope;
      }
      return Ok(code_generation_result);
    }
    let inner = self.inner();
    let Some(source) = &inner.source else {
      return Err(error!(
        "Failed to generate code because ast or source is not set for module {}",
        inner.request
      ));
    };

    let mut code_generation_result = CodeGenerationResult::default();
    if !inner.parsed {
      code_generation_result
        .runtime_requirements
        .insert(RuntimeGlobals::MODULE);
      code_generation_result
        .runtime_requirements
        .insert(RuntimeGlobals::EXPORTS);
      code_generation_result
        .runtime_requirements
        .insert(RuntimeGlobals::THIS_AS_EXPORTS);
    }

    let module_graph = compilation.get_module_graph();
    for source_type in self.source_types(&module_graph) {
      let generation_result = inner
        .parser_and_generator
        .generate(
          source,
          self,
          &mut GenerateContext {
            compilation,
            runtime_requirements: &mut code_generation_result.runtime_requirements,
            data: &mut code_generation_result.data,
            requested_source_type: *source_type,
            runtime,
            concatenation_scope: concatenation_scope.as_mut(),
          },
        )
        .await?;
      code_generation_result.add(*source_type, CachedSource::new(generation_result).boxed());
    }
    code_generation_result.concatenation_scope = concatenation_scope;
    Ok(code_generation_result)
  }

  async fn get_runtime_hash(
    &self,
    compilation: &Compilation,
    runtime: Option<&RuntimeSpec>,
  ) -> Result<RspackHashDigest> {
    let inner = self.inner();
    let mut hasher = RspackHash::from(&compilation.options.output);
    inner.build_info.hash.dyn_hash(&mut hasher);
    // For built failed NormalModule, hash will be calculated by build_info.hash, which contains error message
    if inner.source.is_some() {
      inner
        .parser_and_generator
        .get_runtime_hash(self, compilation, runtime)
        .await?
        .dyn_hash(&mut hasher);
    }
    module_update_hash(self, &mut hasher, compilation, runtime);
    Ok(hasher.digest(&compilation.options.output.hash_digest))
  }

  fn name_for_condition(&self) -> Option<Box<str>> {
    // Align with https://github.com/webpack/webpack/blob/8241da7f1e75c5581ba535d127fa66aeb9eb2ac8/lib/NormalModule.js#L375
    let resource = self.inner().resource_data.resource();
    let idx = resource.find('?');
    if let Some(idx) = idx {
      Some(resource[..idx].into())
    } else {
      Some(resource.into())
    }
  }

  fn lib_ident(&self, options: LibIdentOptions) -> Option<Cow<'_, str>> {
    let mut ident = String::new();
    if let Some(layer) = &self.inner().layer {
      ident += "(";
      ident += layer;
      ident += ")/";
    }
    ident += &contextify(options.context, self.user_request());
    Some(Cow::Owned(ident))
  }

  fn get_resolve_options(&self) -> Option<Arc<Resolve>> {
    self.inner().resolve_options.clone()
  }

  fn get_code_generation_dependencies(&self) -> Option<&[BoxModuleDependency]> {
    if let Some(deps) = self.inner().code_generation_dependencies.as_deref()
      && !deps.is_empty()
    {
      Some(deps)
    } else {
      None
    }
  }

  fn get_presentational_dependencies(&self) -> Option<&[BoxDependencyTemplate]> {
    if let Some(deps) = self.inner().presentational_dependencies.as_deref()
      && !deps.is_empty()
    {
      Some(deps)
    } else {
      None
    }
  }

  fn get_context(&self) -> Option<Box<Context>> {
    Some(self.inner().context.clone())
  }

  fn get_layer(&self) -> Option<&ModuleLayer> {
    self.inner().layer.as_ref()
  }

  // Port from https://github.com/webpack/webpack/blob/main/lib/NormalModule.js#L1120
  fn get_side_effects_connection_state(
    &self,
    module_graph: &ModuleGraph,
    module_graph_cache: &ModuleGraphCacheArtifact,
    module_chain: &mut IdentifierSet,
    connection_state_cache: &mut IdentifierMap<ConnectionState>,
  ) -> ConnectionState {
    module_graph_cache.cached_get_side_effects_connection_state(self.id(), || {
      if let Some(state) = connection_state_cache.get(&self.inner().id) {
        return *state;
      }

      if let Some(side_effect_free) = self.factory_meta().and_then(|m| m.side_effect_free) {
        return ConnectionState::Active(!side_effect_free);
      }
      if Some(true) == self.build_meta().side_effect_free {
        // use module chain instead of is_evaluating_side_effects to mut module graph
        if module_chain.contains(&self.identifier()) {
          return ConnectionState::CircularConnection;
        }
        module_chain.insert(self.identifier());
        let mut current = ConnectionState::Active(false);
        for dependency_id in self.get_dependencies().iter() {
          if let Some(dependency) = module_graph.dependency_by_id(dependency_id) {
            let state = dependency.get_module_evaluation_side_effects_state(
              module_graph,
              module_graph_cache,
              module_chain,
              connection_state_cache,
            );
            if matches!(state, ConnectionState::Active(true)) {
              // TODO add optimization bailout
              module_chain.remove(&self.identifier());
              connection_state_cache.insert(self.inner().id, ConnectionState::Active(true));
              return ConnectionState::Active(true);
            } else if !matches!(state, ConnectionState::CircularConnection) {
              current = current + state;
            }
          }
        }
        module_chain.remove(&self.identifier());
        connection_state_cache.insert(self.inner().id, current);
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
      .inner()
      .parser_and_generator
      .get_concatenation_bailout_reason(self, mg, cg)
  }

  fn factory_meta(&self) -> Option<&FactoryMeta> {
    self.inner().factory_meta.as_ref()
  }

  fn set_factory_meta(&mut self, factory_meta: FactoryMeta) {
    self.inner_mut().factory_meta = Some(factory_meta);
  }

  fn build_info(&self) -> &BuildInfo {
    &self.inner().build_info
  }

  fn build_info_mut(&mut self) -> &mut BuildInfo {
    &mut self.inner_mut().build_info
  }

  fn build_meta(&self) -> &BuildMeta {
    &self.inner().build_meta
  }

  fn build_meta_mut(&mut self) -> &mut BuildMeta {
    &mut self.inner_mut().build_meta
  }
}

impl ModuleSourceMapConfig for NormalModule {
  fn get_source_map_kind(&self) -> &SourceMapKind {
    &self.inner().source_map_kind
  }

  fn set_source_map_kind(&mut self, source_map_kind: SourceMapKind) {
    self.inner_mut().source_map_kind = source_map_kind;
  }
}

impl Diagnosable for NormalModule {
  fn add_diagnostic(&mut self, diagnostic: Diagnostic) {
    self.inner_mut().diagnostics.push(diagnostic);
  }

  fn add_diagnostics(&mut self, mut diagnostics: Vec<Diagnostic>) {
    self.inner_mut().diagnostics.append(&mut diagnostics);
  }

  fn diagnostics(&self) -> Cow<'_, [Diagnostic]> {
    Cow::Borrowed(&self.inner().diagnostics)
  }
}

impl NormalModule {
  fn create_source(&self, content: Content, source_map: Option<SourceMap>) -> Result<BoxSource> {
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
