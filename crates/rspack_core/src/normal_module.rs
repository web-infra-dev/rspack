use std::{
  borrow::Cow,
  fmt::Debug,
  hash::{BuildHasherDefault, Hash},
  sync::{
    atomic::{AtomicUsize, Ordering},
    Mutex,
  },
};

use bitflags::bitflags;
use dashmap::DashMap;
use derivative::Derivative;
use rspack_core_macros::impl_source_map_config;
use rspack_error::{error, Diagnosable, Diagnostic, DiagnosticExt, MietteExt, Result, Severity};
use rspack_hash::RspackHash;
use rspack_identifier::Identifiable;
use rspack_loader_runner::{run_loaders, Content, ResourceData};
use rspack_sources::{
  BoxSource, CachedSource, OriginalSource, RawSource, Source, SourceExt, SourceMap,
  SourceMapSource, WithoutOriginalOptions,
};
use rspack_util::source_map::{ModuleSourceMapConfig, SourceMapKind};
use rustc_hash::FxHashSet as HashSet;
use rustc_hash::FxHasher;
use serde_json::json;

use crate::{
  add_connection_states, contextify, diagnostics::ModuleBuildError, get_context,
  impl_build_info_meta, AsyncDependenciesBlockIdentifier, BoxLoader, BoxModule, BuildContext,
  BuildInfo, BuildMeta, BuildResult, CodeGenerationResult, Compilation, ConcatenationScope,
  ConnectionState, Context, DependenciesBlock, DependencyId, DependencyTemplate, GenerateContext,
  GeneratorOptions, LibIdentOptions, Module, ModuleDependency, ModuleGraph, ModuleIdentifier,
  ModuleType, ParseContext, ParseResult, ParserAndGenerator, ParserOptions, Resolve,
  RspackLoaderRunnerPlugin, RuntimeSpec, SourceType,
};

bitflags! {
  #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
  pub struct ModuleSyntax: u8 {
    const COMMONJS = 1 << 0;
    const ESM = 1 << 1;
  }
}

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

#[impl_source_map_config]
#[derive(Derivative)]
#[derivative(Debug)]
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
  /// Affiliated parser and generator to the module type
  parser_and_generator: Box<dyn ParserAndGenerator>,
  /// Resource matched with inline match resource, (`!=!` syntax)
  match_resource: Option<ResourceData>,
  /// Resource data (path, query, fragment etc.)
  resource_data: ResourceData,
  /// Loaders for the module
  #[derivative(Debug = "ignore")]
  loaders: Vec<BoxLoader>,
  /// Whether loaders list contains inline loader
  contains_inline_loader: bool,

  /// Original content of this module, will be available after module build
  original_source: Option<BoxSource>,
  /// Built source of this module (passed with loaders)
  source: NormalModuleSource,

  /// Resolve options derived from [Rule.resolve]
  resolve_options: Option<Box<Resolve>>,
  /// Parser options derived from [Rule.parser]
  parser_options: Option<ParserOptions>,
  /// Generator options derived from [Rule.generator]
  generator_options: Option<GeneratorOptions>,

  #[allow(unused)]
  debug_id: usize,
  cached_source_sizes: DashMap<SourceType, f64, BuildHasherDefault<FxHasher>>,
  diagnostics: Mutex<Vec<Diagnostic>>,

  code_generation_dependencies: Option<Vec<Box<dyn ModuleDependency>>>,
  presentational_dependencies: Option<Vec<Box<dyn DependencyTemplate>>>,

  build_info: Option<BuildInfo>,
  build_meta: Option<BuildMeta>,
}

#[derive(Debug, Clone)]
pub enum NormalModuleSource {
  Unbuild,
  BuiltSucceed(BoxSource),
  BuiltFailed(Diagnostic),
}

impl NormalModuleSource {
  pub fn new_built(source: BoxSource, mut diagnostics: Vec<Diagnostic>) -> Self {
    diagnostics.retain(|d| d.severity() == Severity::Error);
    // Use the first error as diagnostic
    // See: https://github.com/webpack/webpack/blob/6be4065ade1e252c1d8dcba4af0f43e32af1bdc1/lib/NormalModule.js#L878
    if let Some(d) = diagnostics.into_iter().next() {
      NormalModuleSource::BuiltFailed(d)
    } else {
      NormalModuleSource::BuiltSucceed(source)
    }
  }
}

pub static DEBUG_ID: AtomicUsize = AtomicUsize::new(1);

impl NormalModule {
  fn create_id(module_type: &ModuleType, request: &str) -> String {
    if *module_type == ModuleType::Js {
      request.to_string()
    } else {
      format!("{module_type}|{request}")
    }
  }

  #[allow(clippy::too_many_arguments)]
  pub fn new(
    request: String,
    user_request: String,
    raw_request: String,
    module_type: impl Into<ModuleType>,
    parser_and_generator: Box<dyn ParserAndGenerator>,
    parser_options: Option<ParserOptions>,
    generator_options: Option<GeneratorOptions>,
    match_resource: Option<ResourceData>,
    resource_data: ResourceData,
    resolve_options: Option<Box<Resolve>>,
    loaders: Vec<BoxLoader>,
    contains_inline_loader: bool,
  ) -> Self {
    let module_type = module_type.into();
    let id = Self::create_id(&module_type, &request);
    Self {
      blocks: Vec::new(),
      dependencies: Vec::new(),
      id: ModuleIdentifier::from(id),
      context: Box::new(get_context(&resource_data)),
      request,
      user_request,
      raw_request,
      module_type,
      parser_and_generator,
      parser_options,
      generator_options,
      match_resource,
      resource_data,
      resolve_options,
      loaders,
      contains_inline_loader,
      original_source: None,
      source: NormalModuleSource::Unbuild,
      debug_id: DEBUG_ID.fetch_add(1, Ordering::Relaxed),

      cached_source_sizes: DashMap::default(),
      diagnostics: Mutex::new(Default::default()),
      code_generation_dependencies: None,
      presentational_dependencies: None,
      build_info: None,
      build_meta: None,

      source_map_kind: SourceMapKind::None,
    }
  }

  pub fn id(&self) -> ModuleIdentifier {
    self.id
  }

  pub fn match_resource(&self) -> Option<&ResourceData> {
    self.match_resource.as_ref()
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

  pub fn raw_request(&self) -> &str {
    &self.raw_request
  }

  pub fn source(&self) -> &NormalModuleSource {
    &self.source
  }

  pub fn source_mut(&mut self) -> &mut NormalModuleSource {
    &mut self.source
  }

  pub fn loaders(&self) -> &[BoxLoader] {
    &self.loaders
  }

  pub fn loaders_mut_vec(&mut self) -> &mut Vec<BoxLoader> {
    &mut self.loaders
  }

  pub fn contains_inline_loader(&self) -> bool {
    self.contains_inline_loader
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

  fn get_dependencies(&self) -> &[DependencyId] {
    &self.dependencies
  }

  fn get_presentational_dependencies_for_block(&self) -> Option<&[Box<dyn DependencyTemplate>]> {
    self.get_presentational_dependencies()
  }
}

#[async_trait::async_trait]
impl Module for NormalModule {
  impl_build_info_meta!();

  fn module_type(&self) -> &ModuleType {
    &self.module_type
  }

  fn get_diagnostics(&self) -> Vec<Diagnostic> {
    let guard = self.diagnostics.lock().expect("should have diagnostics");
    guard.clone()
  }

  fn source_types(&self) -> &[SourceType] {
    self.parser_and_generator.source_types()
  }

  fn original_source(&self) -> Option<&dyn Source> {
    self.original_source.as_deref()
  }

  fn readable_identifier(&self, context: &Context) -> Cow<str> {
    Cow::Owned(context.shorten(&self.user_request))
  }

  fn size(&self, source_type: &SourceType) -> f64 {
    if let Some(size_ref) = self.cached_source_sizes.get(source_type) {
      *size_ref
    } else {
      let size = f64::max(1.0, self.parser_and_generator.size(self, source_type));
      self.cached_source_sizes.insert(*source_type, size);
      size
    }
  }

  async fn build(
    &mut self,
    build_context: BuildContext<'_>,
    _compilation: Option<&Compilation>,
  ) -> Result<BuildResult> {
    self.clear_diagnostics();

    let mut build_info = BuildInfo::default();
    let mut build_meta = BuildMeta::default();

    build_context.plugin_driver.before_loaders(self).await?;

    let plugin = RspackLoaderRunnerPlugin {
      plugin_driver: build_context.plugin_driver.clone(),
      normal_module: self,
      current_loader: Default::default(),
    };

    let loader_result = run_loaders(
      &self.loaders,
      &self.resource_data,
      &[&plugin],
      build_context.compiler_context,
    )
    .await;
    let (loader_result, ds) = match loader_result {
      Ok(r) => r.split_into_parts(),
      Err(e) => {
        let mut e = ModuleBuildError(e).boxed();
        {
          let mut current = plugin
            .current_loader
            .lock()
            .expect("should be able to lock");
          if let Some(current) = current.take() {
            e = e.with_help(format!("File was processed with this loader: '{current}'"));
          }
        };
        let d = Diagnostic::from(e);
        self.source = NormalModuleSource::BuiltFailed(d.clone());
        self.add_diagnostic(d);
        let mut hasher = RspackHash::from(&build_context.compiler_options.output);
        self.update_hash(&mut hasher);
        build_meta.hash(&mut hasher);
        build_info.hash = Some(hasher.digest(&build_context.compiler_options.output.hash_digest));
        return Ok(BuildResult {
          build_info,
          build_meta: Default::default(),
          dependencies: Vec::new(),
          blocks: Vec::new(),
          analyze_result: Default::default(),
        });
      }
    };
    self.add_diagnostics(ds);

    let content = if self.module_type().is_binary() {
      Content::Buffer(loader_result.content.into_bytes())
    } else {
      Content::String(loader_result.content.into_string_lossy())
    };
    let original_source = self.create_source(content, loader_result.source_map)?;
    let mut code_generation_dependencies: Vec<Box<dyn ModuleDependency>> = Vec::new();

    let (
      ParseResult {
        source,
        dependencies,
        blocks,
        presentational_dependencies,
        analyze_result,
      },
      ds,
    ) = self
      .parser_and_generator
      .parse(ParseContext {
        source: original_source.clone(),
        module_identifier: self.identifier(),
        module_parser_options: self.parser_options.as_ref(),
        module_type: &self.module_type,
        module_user_request: &self.user_request,
        module_source_map_kind: self.get_source_map_kind().clone(),
        loaders: &self.loaders,
        resource_data: &self.resource_data,
        compiler_options: build_context.compiler_options,
        additional_data: loader_result.additional_data,
        code_generation_dependencies: &mut code_generation_dependencies,
        build_info: &mut build_info,
        build_meta: &mut build_meta,
      })?
      .split_into_parts();
    self.add_diagnostics(ds);
    // Only side effects used in code_generate can stay here
    // Other side effects should be set outside use_cache
    self.original_source = Some(source.clone());
    self.source = NormalModuleSource::new_built(source, self.clone_diagnostics());
    self.code_generation_dependencies = Some(code_generation_dependencies);
    self.presentational_dependencies = Some(presentational_dependencies);

    let mut hasher = RspackHash::from(&build_context.compiler_options.output);
    self.update_hash(&mut hasher);
    build_meta.hash(&mut hasher);

    build_info.hash = Some(hasher.digest(&build_context.compiler_options.output.hash_digest));
    build_info.cacheable = loader_result.cacheable;
    build_info.file_dependencies = loader_result.file_dependencies;
    build_info.context_dependencies = loader_result.context_dependencies;
    build_info.missing_dependencies = loader_result.missing_dependencies;
    build_info.build_dependencies = loader_result.build_dependencies;
    build_info.asset_filenames = loader_result.asset_filenames;

    Ok(BuildResult {
      build_info,
      build_meta,
      dependencies,
      blocks,
      analyze_result,
    })
  }

  fn code_generation(
    &self,
    compilation: &Compilation,
    runtime: Option<&RuntimeSpec>,
    mut concatenation_scope: Option<ConcatenationScope>,
  ) -> Result<CodeGenerationResult> {
    if let NormalModuleSource::BuiltSucceed(source) = &self.source {
      let mut code_generation_result = CodeGenerationResult::default();
      for source_type in self.source_types() {
        let generation_result = self.parser_and_generator.generate(
          source,
          self,
          &mut GenerateContext {
            compilation,
            module_generator_options: self.generator_options.as_ref(),
            runtime_requirements: &mut code_generation_result.runtime_requirements,
            data: &mut code_generation_result.data,
            requested_source_type: *source_type,
            runtime,
            concatenation_scope: concatenation_scope.as_mut(),
          },
        )?;
        code_generation_result.add(*source_type, CachedSource::new(generation_result).boxed());
      }
      code_generation_result.set_hash(
        &compilation.options.output.hash_function,
        &compilation.options.output.hash_digest,
        &compilation.options.output.hash_salt,
      );
      code_generation_result.concatenation_scope = concatenation_scope;
      Ok(code_generation_result)
    } else if let NormalModuleSource::BuiltFailed(error_message) = &self.source {
      let mut code_generation_result = CodeGenerationResult::default();

      // If the module build failed and the module is able to emit JavaScript source,
      // we should emit an error message to the runtime, otherwise we do nothing.
      if self.source_types().contains(&SourceType::JavaScript) {
        let error = error_message.render_report(compilation.options.stats.colors)?;
        code_generation_result.add(
          SourceType::JavaScript,
          RawSource::from(format!("throw new Error({});\n", json!(error))).boxed(),
        );
      }
      code_generation_result.set_hash(
        &compilation.options.output.hash_function,
        &compilation.options.output.hash_digest,
        &compilation.options.output.hash_salt,
      );
      Ok(code_generation_result)
    } else {
      Err(error!(
        "Failed to generate code because ast or source is not set for module {}",
        self.request
      ))
    }
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
    // Align with https://github.com/webpack/webpack/blob/4b4ca3bb53f36a5b8fc6bc1bd976ed7af161bd80/lib/NormalModule.js#L362
    Some(Cow::Owned(contextify(options.context, self.user_request())))
  }

  fn get_resolve_options(&self) -> Option<Box<Resolve>> {
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

  // Port from https://github.com/webpack/webpack/blob/main/lib/NormalModule.js#L1120
  fn get_side_effects_connection_state(
    &self,
    module_graph: &ModuleGraph,
    module_chain: &mut HashSet<ModuleIdentifier>,
  ) -> ConnectionState {
    if let Some(mgm) = module_graph.module_graph_module_by_identifier(&self.identifier()) {
      if let Some(side_effect_free) = mgm.factory_meta.as_ref().and_then(|m| m.side_effect_free) {
        return ConnectionState::Bool(!side_effect_free);
      }
      if let Some(side_effect_free) = self.build_meta().as_ref().and_then(|m| m.side_effect_free)
        && side_effect_free
      {
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
              current = add_connection_states(current, state);
            }
          }
        }
        module_chain.remove(&self.identifier());
        return current;
      }
    }
    ConnectionState::Bool(true)
  }
}

impl Diagnosable for NormalModule {
  fn add_diagnostic(&self, diagnostic: Diagnostic) {
    self
      .diagnostics
      .lock()
      .expect("should be able to lock diagnostics")
      .push(diagnostic);
  }

  fn add_diagnostics(&self, mut diagnostics: Vec<Diagnostic>) {
    self
      .diagnostics
      .lock()
      .expect("should be able to lock diagnostics")
      .append(&mut diagnostics);
  }

  fn clone_diagnostics(&self) -> Vec<Diagnostic> {
    self
      .diagnostics
      .lock()
      .expect("should be able to lock diagnostics")
      .iter()
      .cloned()
      .collect()
  }
}

impl PartialEq for NormalModule {
  fn eq(&self, other: &Self) -> bool {
    self.identifier() == other.identifier()
  }
}

impl Eq for NormalModule {}

impl NormalModule {
  fn create_source(&self, content: Content, source_map: Option<SourceMap>) -> Result<BoxSource> {
    if content.is_buffer() {
      return Ok(RawSource::Buffer(content.into_bytes()).boxed());
    }
    let source_map_kind = self.get_source_map_kind();
    if !matches!(source_map_kind, SourceMapKind::None)
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
    if !matches!(source_map_kind, SourceMapKind::None)
      && let Content::String(content) = content
    {
      return Ok(OriginalSource::new(content, self.request()).boxed());
    }
    Ok(RawSource::from(content.into_string_lossy()).boxed())
  }

  fn clear_diagnostics(&mut self) {
    self
      .diagnostics
      .lock()
      .expect("should be able to lock diagnostics")
      .clear()
  }
}

impl Hash for NormalModule {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    "__rspack_internal__NormalModule".hash(state);
    if let Some(original_source) = &self.original_source {
      original_source.hash(state);
    }
  }
}
