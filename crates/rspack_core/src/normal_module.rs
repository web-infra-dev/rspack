use std::{
  borrow::Cow,
  hash::{BuildHasherDefault, Hash},
  ptr::NonNull,
  sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
  },
};

use dashmap::DashMap;
use derive_more::Debug;
use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsMap, AsOption, AsPreset, Skip},
};
use rspack_collections::{Identifiable, IdentifierSet};
use rspack_error::{error, Diagnosable, Diagnostic, DiagnosticExt, NodeError, Result, Severity};
use rspack_hash::{RspackHash, RspackHashDigest};
use rspack_hook::define_hook;
use rspack_loader_runner::{run_loaders, AdditionalData, Content, LoaderContext, ResourceData};
use rspack_macros::impl_source_map_config;
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

use crate::{
  contextify,
  diagnostics::{CapturedLoaderError, ModuleBuildError},
  get_context, impl_module_meta_info, module_update_hash, AsyncDependenciesBlockIdentifier,
  BoxLoader, BoxModule, BuildContext, BuildInfo, BuildMeta, BuildResult, ChunkGraph,
  CodeGenerationResult, Compilation, ConcatenationScope, ConnectionState, Context,
  DependenciesBlock, DependencyId, DependencyTemplate, FactoryMeta, GenerateContext,
  GeneratorOptions, LibIdentOptions, Module, ModuleDependency, ModuleGraph, ModuleIdentifier,
  ModuleLayer, ModuleType, OutputOptions, ParseContext, ParseResult, ParserAndGenerator,
  ParserOptions, Resolve, RspackLoaderRunnerPlugin, RunnerContext, RuntimeGlobals, RuntimeSpec,
  SourceType,
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

define_hook!(NormalModuleReadResource: AsyncSeriesBail(resource_data: &ResourceData) -> Content);
define_hook!(NormalModuleLoader: SyncSeries(loader_context: &mut LoaderContext<RunnerContext>));
define_hook!(NormalModuleLoaderShouldYield: SyncSeriesBail(loader_context: &LoaderContext<RunnerContext>) -> bool);
define_hook!(NormalModuleLoaderStartYielding: AsyncSeries(loader_context: &mut LoaderContext<RunnerContext>));
define_hook!(NormalModuleBeforeLoaders: SyncSeries(module: &mut NormalModule));
define_hook!(NormalModuleAdditionalData: AsyncSeries(additional_data: &mut Option<&mut AdditionalData>));

#[derive(Debug, Default)]
pub struct NormalModuleHooks {
  pub read_resource: NormalModuleReadResourceHook,
  pub loader: NormalModuleLoaderHook,
  pub loader_should_yield: NormalModuleLoaderShouldYieldHook,
  pub loader_yield: NormalModuleLoaderStartYieldingHook,
  pub before_loaders: NormalModuleBeforeLoadersHook,
  pub additional_data: NormalModuleAdditionalDataHook,
}

#[impl_source_map_config]
#[cacheable]
#[derive(Debug)]
pub struct NormalModule {
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

  #[allow(unused)]
  debug_id: usize,
  #[cacheable(with=AsMap)]
  cached_source_sizes: DashMap<SourceType, f64, BuildHasherDefault<FxHasher>>,
  #[cacheable(with=Skip)]
  diagnostics: Vec<Diagnostic>,

  code_generation_dependencies: Option<Vec<Box<dyn ModuleDependency>>>,
  presentational_dependencies: Option<Vec<Box<dyn DependencyTemplate>>>,

  factory_meta: Option<FactoryMeta>,
  build_info: BuildInfo,
  build_meta: BuildMeta,
  parsed: bool,
}

static DEBUG_ID: AtomicUsize = AtomicUsize::new(1);

impl NormalModule {
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
  ) -> Self {
    let module_type = module_type.into();
    let id = Self::create_id(&module_type, layer.as_ref(), &request);
    Self {
      blocks: Vec::new(),
      dependencies: Vec::new(),
      id: ModuleIdentifier::from(id.as_ref()),
      context: Box::new(get_context(&resource_data)),
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

      cached_source_sizes: DashMap::default(),
      diagnostics: Default::default(),
      code_generation_dependencies: None,
      presentational_dependencies: None,
      factory_meta: None,
      build_info: Default::default(),
      build_meta: Default::default(),
      parsed: false,
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

  pub fn resource_resolved_data(&self) -> &ResourceData {
    &self.resource_data
  }

  pub fn request(&self) -> &str {
    &self.request
  }

  pub fn user_request(&self) -> &str {
    &self.user_request
  }

  pub fn user_request_mut(&mut self) -> &mut String {
    &mut self.user_request
  }

  pub fn raw_request(&self) -> &str {
    &self.raw_request
  }

  pub fn loaders(&self) -> &[BoxLoader] {
    &self.loaders
  }

  pub fn parser_and_generator(&self) -> &dyn ParserAndGenerator {
    &*self.parser_and_generator
  }

  pub fn parser_and_generator_mut(&mut self) -> &mut Box<dyn ParserAndGenerator> {
    &mut self.parser_and_generator
  }

  pub fn code_generation_dependencies(&self) -> &Option<Vec<Box<dyn ModuleDependency>>> {
    &self.code_generation_dependencies
  }

  pub fn code_generation_dependencies_mut(
    &mut self,
  ) -> &mut Option<Vec<Box<dyn ModuleDependency>>> {
    &mut self.code_generation_dependencies
  }

  pub fn presentational_dependencies(&self) -> &Option<Vec<Box<dyn DependencyTemplate>>> {
    &self.presentational_dependencies
  }

  pub fn presentational_dependencies_mut(
    &mut self,
  ) -> &mut Option<Vec<Box<dyn DependencyTemplate>>> {
    &mut self.presentational_dependencies
  }

  #[tracing::instrument("NormalModule:build_hash")]
  fn init_build_hash(
    &self,
    output_options: &OutputOptions,
    build_meta: &BuildMeta,
  ) -> RspackHashDigest {
    let mut hasher = RspackHash::from(output_options);
    "source".hash(&mut hasher);
    if let Some(error) = self.first_error() {
      error.message().hash(&mut hasher);
    } else if let Some(s) = &self.source {
      s.hash(&mut hasher);
    }
    "meta".hash(&mut hasher);
    build_meta.hash(&mut hasher);
    hasher.digest(&output_options.hash_digest)
  }

  pub fn get_generator_options(&self) -> Option<&GeneratorOptions> {
    self.generator_options.as_ref()
  }
}

impl Identifiable for NormalModule {
  #[inline]
  fn identifier(&self) -> ModuleIdentifier {
    self.id
  }
}

impl DependenciesBlock for NormalModule {
  fn add_block_id(&mut self, block: AsyncDependenciesBlockIdentifier) {
    self.blocks.push(block)
  }

  fn get_blocks(&self) -> &[AsyncDependenciesBlockIdentifier] {
    &self.blocks
  }

  fn add_dependency_id(&mut self, dependency: DependencyId) {
    self.dependencies.push(dependency)
  }

  fn remove_dependency_id(&mut self, dependency: DependencyId) {
    self.dependencies.retain(|d| d != &dependency)
  }

  fn get_dependencies(&self) -> &[DependencyId] {
    &self.dependencies
  }
}

#[cacheable_dyn]
#[async_trait::async_trait]
impl Module for NormalModule {
  impl_module_meta_info!();

  fn module_type(&self) -> &ModuleType {
    &self.module_type
  }

  fn source_types(&self) -> &[SourceType] {
    self.parser_and_generator.source_types()
  }

  fn source(&self) -> Option<&BoxSource> {
    self.source.as_ref()
  }

  fn readable_identifier(&self, context: &Context) -> Cow<str> {
    Cow::Owned(context.shorten(&self.user_request))
  }

  fn size(&self, source_type: Option<&SourceType>, _compilation: Option<&Compilation>) -> f64 {
    if let Some(size_ref) = source_type.and_then(|st| self.cached_source_sizes.get(st)) {
      *size_ref
    } else {
      let size = f64::max(1.0, self.parser_and_generator.size(self, source_type));
      source_type.and_then(|st| self.cached_source_sizes.insert(*st, size));
      size
    }
  }

  #[tracing::instrument("NormalModule:build", skip_all, fields(
    module.resource = self.resource_resolved_data().resource.as_str(),
    module.identifier = self.identifier().as_str(),
    module.loaders = ?self.loaders.iter().map(|l| l.identifier().as_str()).collect::<Vec<_>>())
  )]
  async fn build(
    &mut self,
    build_context: BuildContext,
    _compilation: Option<&Compilation>,
  ) -> Result<BuildResult> {
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
      .call(self)?;

    let plugin = Arc::new(RspackLoaderRunnerPlugin {
      plugin_driver: build_context.plugin_driver.clone(),
      current_loader: Default::default(),
    });

    let loader_result = run_loaders(
      self.loaders.clone(),
      self.resource_data.clone(),
      Some(plugin.clone()),
      RunnerContext {
        compiler_id: build_context.compiler_id,
        compilation_id: build_context.compilation_id,
        options: build_context.compiler_options.clone(),
        resolver_factory: build_context.resolver_factory.clone(),
        #[allow(clippy::unwrap_used)]
        module: NonNull::new(self).unwrap(),
        module_source_map_kind: self.source_map_kind,
      },
      build_context.fs.clone(),
    )
    .await;
    let (mut loader_result, ds) = match loader_result {
      Ok(r) => r.split_into_parts(),
      Err(mut r) => {
        let diagnostic = if let Some(captured_error) = r.downcast_mut::<CapturedLoaderError>() {
          self.build_info.cacheable = captured_error.cacheable;
          self.build_info.file_dependencies = captured_error
            .take_file_dependencies()
            .into_iter()
            .map(Into::into)
            .collect();
          self.build_info.context_dependencies = captured_error
            .take_context_dependencies()
            .into_iter()
            .map(Into::into)
            .collect();
          self.build_info.missing_dependencies = captured_error
            .take_missing_dependencies()
            .into_iter()
            .map(Into::into)
            .collect();
          self.build_info.build_dependencies = captured_error
            .take_build_dependencies()
            .into_iter()
            .map(Into::into)
            .collect();

          let stack = captured_error.take_stack();
          Diagnostic::from(
            ModuleBuildError(error!(if captured_error.hide_stack.unwrap_or_default() {
              captured_error.take_message()
            } else {
              stack
                .clone()
                .unwrap_or_else(|| captured_error.take_message())
            }))
            .boxed(),
          )
          .with_stack(stack)
          .with_hide_stack(captured_error.hide_stack)
        } else {
          self.build_info.cacheable = false;
          if let Some(file_path) = &self.resource_data.resource_path {
            if file_path.is_absolute() {
              self
                .build_info
                .file_dependencies
                .insert(file_path.clone().into_std_path_buf().into());
            }
          }
          let node_error = r.downcast_ref::<NodeError>();
          let stack = node_error.and_then(|e| e.stack.clone());
          let hide_stack = node_error.and_then(|e| e.hide_stack);
          let e = ModuleBuildError(r).boxed();
          Diagnostic::from(e)
            .with_stack(stack)
            .with_hide_stack(hide_stack)
        };

        self.source = None;
        self.add_diagnostic(diagnostic);

        self.build_info.hash =
          Some(self.init_build_hash(&build_context.compiler_options.output, &self.build_meta));
        return Ok(BuildResult {
          dependencies: Vec::new(),
          blocks: Vec::new(),
          optimization_bailouts: vec![],
        });
      }
    };
    build_context
      .plugin_driver
      .normal_module_hooks
      .additional_data
      .call(&mut loader_result.additional_data.as_mut())
      .await?;
    self.add_diagnostics(ds);

    let content = if self.module_type().is_binary() {
      Content::Buffer(loader_result.content.into_bytes())
    } else {
      Content::String(loader_result.content.into_string_lossy())
    };
    let source = self.create_source(content, loader_result.source_map)?;

    self.build_info.cacheable = loader_result.cacheable;
    self.build_info.file_dependencies = loader_result
      .file_dependencies
      .into_iter()
      .map(Into::into)
      .collect();
    self.build_info.context_dependencies = loader_result
      .context_dependencies
      .into_iter()
      .map(Into::into)
      .collect();
    self.build_info.missing_dependencies = loader_result
      .missing_dependencies
      .into_iter()
      .map(Into::into)
      .collect();
    self.build_info.build_dependencies = loader_result
      .build_dependencies
      .into_iter()
      .map(Into::into)
      .collect();

    if no_parse {
      self.parsed = false;
      self.source = Some(source);
      self.code_generation_dependencies = Some(Vec::new());
      self.presentational_dependencies = Some(Vec::new());

      self.build_info.hash =
        Some(self.init_build_hash(&build_context.compiler_options.output, &self.build_meta));

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
    ) = self
      .parser_and_generator
      .parse(ParseContext {
        source: source.clone(),
        module_context: &self.context,
        module_identifier: self.identifier(),
        module_parser_options: self.parser_options.as_ref(),
        module_type: &self.module_type,
        module_layer: self.layer.as_ref(),
        module_user_request: &self.user_request,
        module_source_map_kind: *self.get_source_map_kind(),
        loaders: &self.loaders,
        resource_data: &self.resource_data,
        compiler_options: &build_context.compiler_options,
        additional_data: loader_result.additional_data,
        build_info: &mut self.build_info,
        build_meta: &mut self.build_meta,
        parse_meta: loader_result.parse_meta,
      })?
      .split_into_parts();
    if diagnostics.iter().any(|d| d.severity() == Severity::Error) {
      self.build_meta = Default::default();
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
    self.source = Some(source);
    self.code_generation_dependencies = Some(code_generation_dependencies);
    self.presentational_dependencies = Some(presentational_dependencies);

    self.build_info.hash =
      Some(self.init_build_hash(&build_context.compiler_options.output, &self.build_meta));

    Ok(BuildResult {
      dependencies,
      blocks,
      optimization_bailouts,
    })
  }

  // #[tracing::instrument("NormalModule::code_generation", skip_all, fields(identifier = ?self.identifier()))]
  fn code_generation(
    &self,
    compilation: &Compilation,
    runtime: Option<&RuntimeSpec>,
    mut concatenation_scope: Option<ConcatenationScope>,
  ) -> Result<CodeGenerationResult> {
    if let Some(error) = self.first_error() {
      let mut code_generation_result = CodeGenerationResult::default();

      // If the module build failed and the module is able to emit JavaScript source,
      // we should emit an error message to the runtime, otherwise we do nothing.
      if self.source_types().contains(&SourceType::JavaScript) {
        let error = error.render_report(compilation.options.stats.colors)?;
        code_generation_result.add(
          SourceType::JavaScript,
          RawStringSource::from(format!("throw new Error({});\n", json!(error))).boxed(),
        );
        code_generation_result.concatenation_scope = concatenation_scope;
      }
      return Ok(code_generation_result);
    }
    let Some(source) = &self.source else {
      return Err(error!(
        "Failed to generate code because ast or source is not set for module {}",
        self.request
      ));
    };

    let mut code_generation_result = CodeGenerationResult::default();
    if !self.parsed {
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
    for source_type in self.source_types() {
      let generation_result = self.parser_and_generator.generate(
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
      )?;
      code_generation_result.add(*source_type, CachedSource::new(generation_result).boxed());
    }
    code_generation_result.concatenation_scope = concatenation_scope;
    Ok(code_generation_result)
  }

  fn update_hash(
    &self,
    hasher: &mut dyn std::hash::Hasher,
    compilation: &Compilation,
    runtime: Option<&RuntimeSpec>,
  ) -> Result<()> {
    self.build_info.hash.dyn_hash(hasher);
    // For built failed NormalModule, hash will be calculated by build_info.hash, which contains error message
    if self.source.is_some() {
      self
        .parser_and_generator
        .update_hash(self, hasher, compilation, runtime)?;
    }
    module_update_hash(self, hasher, compilation, runtime);
    Ok(())
  }

  fn name_for_condition(&self) -> Option<Box<str>> {
    // Align with https://github.com/webpack/webpack/blob/8241da7f1e75c5581ba535d127fa66aeb9eb2ac8/lib/NormalModule.js#L375
    let resource = self.resource_data.resource.as_str();
    let idx = resource.find('?');
    if let Some(idx) = idx {
      Some(resource[..idx].into())
    } else {
      Some(resource.into())
    }
  }

  fn lib_ident(&self, options: LibIdentOptions) -> Option<Cow<str>> {
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

  fn get_code_generation_dependencies(&self) -> Option<&[Box<dyn ModuleDependency>]> {
    if let Some(deps) = self.code_generation_dependencies.as_deref()
      && !deps.is_empty()
    {
      Some(deps)
    } else {
      None
    }
  }

  fn get_presentational_dependencies(&self) -> Option<&[Box<dyn DependencyTemplate>]> {
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
    module_chain: &mut IdentifierSet,
  ) -> ConnectionState {
    if let Some(side_effect_free) = self.factory_meta().and_then(|m| m.side_effect_free) {
      return ConnectionState::Bool(!side_effect_free);
    }
    if Some(true) == self.build_meta().side_effect_free {
      // use module chain instead of is_evaluating_side_effects to mut module graph
      if module_chain.contains(&self.identifier()) {
        return ConnectionState::CircularConnection;
      }
      module_chain.insert(self.identifier());
      let mut current = ConnectionState::Bool(false);
      for dependency_id in self.get_dependencies().iter() {
        if let Some(dependency) = module_graph.dependency_by_id(dependency_id) {
          let state =
            dependency.get_module_evaluation_side_effects_state(module_graph, module_chain);
          if matches!(state, ConnectionState::Bool(true)) {
            // TODO add optimization bailout
            module_chain.remove(&self.identifier());
            return ConnectionState::Bool(true);
          } else if !matches!(state, ConnectionState::CircularConnection) {
            current = current + state;
          }
        }
      }
      module_chain.remove(&self.identifier());
      return current;
    }
    ConnectionState::Bool(true)
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
}

impl Diagnosable for NormalModule {
  fn add_diagnostic(&mut self, diagnostic: Diagnostic) {
    self.diagnostics.push(diagnostic);
  }

  fn add_diagnostics(&mut self, mut diagnostics: Vec<Diagnostic>) {
    self.diagnostics.append(&mut diagnostics);
  }

  fn diagnostics(&self) -> Cow<[Diagnostic]> {
    Cow::Borrowed(&self.diagnostics)
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
