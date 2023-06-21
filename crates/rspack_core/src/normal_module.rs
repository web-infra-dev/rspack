use std::{
  borrow::Cow,
  fmt::Debug,
  hash::{BuildHasherDefault, Hash},
  sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
  },
};

use bitflags::bitflags;
use dashmap::DashMap;
use derivative::Derivative;
use rspack_error::{
  internal_error, Diagnostic, IntoTWithDiagnosticArray, Result, Severity, TWithDiagnosticArray,
};
use rspack_hash::RspackHash;
use rspack_identifier::Identifiable;
use rspack_loader_runner::{run_loaders, Content, ResourceData};
use rspack_sources::{
  BoxSource, CachedSource, OriginalSource, RawSource, Source, SourceExt, SourceMap,
  SourceMapSource, WithoutOriginalOptions,
};
use rustc_hash::FxHasher;
use serde_json::json;

use crate::{
  contextify, get_context, BoxLoader, BoxModule, BuildContext, BuildInfo, BuildMeta, BuildResult,
  CodeGenerationResult, CodeReplaceSourceDependency, Compilation, CompilerOptions, Context,
  Dependency, GenerateContext, GeneratorOptions, LibIdentOptions,
  LoaderRunnerPluginProcessResource, Module, ModuleAst, ModuleDependency, ModuleGraph,
  ModuleIdentifier, ModuleType, ParseContext, ParseResult, ParserAndGenerator, ParserOptions,
  Resolve, SourceType,
};

bitflags! {
  #[derive(Default)]
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
    if let Some(id) = self.identifier() && let Some(module) = module_graph.module_by_identifier(id) {
      Some(module)
    } else {
      None
    }
  }
}

// TODO Here should only has source for string replace codegen, but the tree-shaking analyzer need ast at now, so here use ast as workaround.
#[derive(Debug, Clone, Hash)]
pub struct AstOrSource {
  inner: (Option<ModuleAst>, Option<BoxSource>),
}

impl AstOrSource {
  pub fn new(ast: Option<ModuleAst>, source: Option<BoxSource>) -> Self {
    Self {
      inner: (ast, source),
    }
  }

  pub fn as_ast(&self) -> Option<&ModuleAst> {
    self.inner.0.as_ref()
  }

  pub fn as_source(&self) -> Option<&BoxSource> {
    self.inner.1.as_ref()
  }

  pub fn try_into_ast(self) -> Result<ModuleAst> {
    match self.inner.0 {
      Some(ast) => Ok(ast),
      // TODO: change to user error
      _ => Err(internal_error!("Failed to convert to ast")),
    }
  }

  pub fn try_into_source(self) -> Result<BoxSource> {
    match self.inner.1 {
      Some(source) => Ok(source),
      // TODO: change to user error
      _ => Err(internal_error!("Failed to convert to source")),
    }
  }

  pub fn map<F, G>(self, f: F, g: G) -> Self
  where
    F: FnOnce(ModuleAst) -> ModuleAst,
    G: FnOnce(BoxSource) -> BoxSource,
  {
    let ast = self.inner.0.map(f);
    let source = self.inner.1.map(g);
    Self::new(ast, source)
  }
}

impl From<ModuleAst> for AstOrSource {
  fn from(ast: ModuleAst) -> Self {
    AstOrSource::new(Some(ast), None)
  }
}

impl From<BoxSource> for AstOrSource {
  fn from(source: BoxSource) -> Self {
    AstOrSource::new(None, Some(source))
  }
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct NormalModule {
  id: ModuleIdentifier,
  /// Context of this module
  context: Context,
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

  /// Original content of this module, will be available after module build
  original_source: Option<BoxSource>,
  /// Built AST or source of this module (passed with loaders)
  ast_or_source: NormalModuleAstOrSource,

  /// Resolve options derived from [Rule.resolve]
  resolve_options: Option<Resolve>,
  /// Parser options derived from [Rule.parser]
  parser_options: Option<ParserOptions>,
  /// Generator options derived from [Rule.generator]
  generator_options: Option<GeneratorOptions>,

  options: Arc<CompilerOptions>,
  #[allow(unused)]
  debug_id: usize,
  cached_source_sizes: DashMap<SourceType, f64, BuildHasherDefault<FxHasher>>,

  code_generation_dependencies: Option<Vec<Box<dyn ModuleDependency>>>,
  presentational_dependencies: Option<Vec<Box<dyn Dependency>>>,
  code_replace_source_dependencies: Option<Vec<Box<dyn CodeReplaceSourceDependency>>>,
}

#[derive(Debug)]
pub enum NormalModuleAstOrSource {
  Unbuild,
  BuiltSucceed(AstOrSource),
  BuiltFailed(String),
}

impl NormalModuleAstOrSource {
  pub fn new_built(ast_or_source: AstOrSource, diagnostics: &[Diagnostic]) -> Self {
    if diagnostics.iter().any(|d| d.severity == Severity::Error) {
      NormalModuleAstOrSource::BuiltFailed(
        diagnostics
          .iter()
          .filter(|d| d.severity == Severity::Error)
          .map(|d| d.message.clone())
          .collect::<Vec<String>>()
          .join("\n"),
      )
    } else {
      NormalModuleAstOrSource::BuiltSucceed(ast_or_source)
    }
  }
}

pub static DEBUG_ID: AtomicUsize = AtomicUsize::new(1);

impl NormalModule {
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
    resolve_options: Option<Resolve>,
    loaders: Vec<BoxLoader>,
    options: Arc<CompilerOptions>,
  ) -> Self {
    let module_type = module_type.into();
    let identifier = if module_type == ModuleType::Js {
      request.to_string()
    } else {
      format!("{module_type}|{request}")
    };
    Self {
      id: ModuleIdentifier::from(identifier),
      context: get_context(&resource_data),
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
      original_source: None,
      ast_or_source: NormalModuleAstOrSource::Unbuild,
      debug_id: DEBUG_ID.fetch_add(1, Ordering::Relaxed),

      options,
      cached_source_sizes: DashMap::default(),
      code_generation_dependencies: None,
      presentational_dependencies: None,
      code_replace_source_dependencies: None,
    }
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

  pub fn source(&self) -> Option<&dyn Source> {
    match self.ast_or_source() {
      NormalModuleAstOrSource::BuiltSucceed(ast_or_source) => {
        ast_or_source.as_source().map(|source| source.as_ref())
      }
      _ => None,
    }
  }

  pub fn ast(&self) -> Option<&ModuleAst> {
    match self.ast_or_source() {
      NormalModuleAstOrSource::BuiltSucceed(ast_or_source) => ast_or_source.as_ast(),
      _ => None,
    }
  }

  pub fn ast_or_source(&self) -> &NormalModuleAstOrSource {
    &self.ast_or_source
  }

  pub fn ast_or_source_mut(&mut self) -> &mut NormalModuleAstOrSource {
    &mut self.ast_or_source
  }

  pub fn loaders_mut_vec(&mut self) -> &mut Vec<BoxLoader> {
    &mut self.loaders
  }
}

impl Identifiable for NormalModule {
  #[inline]
  fn identifier(&self) -> ModuleIdentifier {
    self.id
  }
}

#[async_trait::async_trait]
impl Module for NormalModule {
  fn module_type(&self) -> &ModuleType {
    &self.module_type
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
  ) -> Result<TWithDiagnosticArray<BuildResult>> {
    let mut build_info = BuildInfo::default();
    let mut build_meta = BuildMeta::default();
    let mut diagnostics = Vec::new();

    build_context.plugin_driver.before_loaders(self).await?;

    let loader_result = {
      run_loaders(
        &self.loaders,
        &self.resource_data,
        &[Box::new(LoaderRunnerPluginProcessResource {
          plugin_driver: build_context.plugin_driver.clone(),
        })],
        build_context.compiler_context,
      )
      .await
    };
    let (loader_result, ds) = match loader_result {
      Ok(r) => r.split_into_parts(),
      Err(e) => {
        self.ast_or_source = NormalModuleAstOrSource::BuiltFailed(e.to_string());
        let mut hasher = RspackHash::from(&build_context.compiler_options.output);
        self.update_hash(&mut hasher);
        build_meta.hash(&mut hasher);
        build_info.hash = Some(hasher.digest(&build_context.compiler_options.output.hash_digest));
        return Ok(
          BuildResult {
            build_info,
            build_meta: Default::default(),
            dependencies: Vec::new(),
          }
          .with_diagnostic(e.into()),
        );
      }
    };
    diagnostics.extend(ds);

    let original_source = self.create_source(loader_result.content, loader_result.source_map)?;
    let mut code_generation_dependencies: Vec<Box<dyn ModuleDependency>> = Vec::new();

    let (
      ParseResult {
        ast_or_source,
        dependencies,
        presentational_dependencies,
        code_replace_source_dependencies,
      },
      ds,
    ) = self
      .parser_and_generator
      .parse(ParseContext {
        source: original_source.clone(),
        module_identifier: self.identifier(),
        module_parser_options: self.parser_options.as_ref(),
        module_type: &self.module_type,
        resource_data: &self.resource_data,
        compiler_options: build_context.compiler_options,
        additional_data: loader_result.additional_data,
        code_generation_dependencies: &mut code_generation_dependencies,
        build_info: &mut build_info,
        build_meta: &mut build_meta,
      })?
      .split_into_parts();
    diagnostics.extend(ds);

    // Only side effects used in code_generate can stay here
    // Other side effects should be set outside use_cache
    self.original_source = Some(original_source);
    self.ast_or_source = NormalModuleAstOrSource::new_built(ast_or_source, &diagnostics);
    self.code_generation_dependencies = Some(code_generation_dependencies);
    self.presentational_dependencies = Some(presentational_dependencies);
    self.code_replace_source_dependencies = Some(code_replace_source_dependencies);

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

    Ok(
      BuildResult {
        build_info,
        build_meta,
        dependencies,
      }
      .with_diagnostic(diagnostics),
    )
  }

  fn code_generation(&self, compilation: &Compilation) -> Result<CodeGenerationResult> {
    if let NormalModuleAstOrSource::BuiltSucceed(ast_or_source) = self.ast_or_source() {
      let mut code_generation_result = CodeGenerationResult::default();
      for source_type in self.source_types() {
        let mut generation_result = self.parser_and_generator.generate(
          ast_or_source,
          self,
          &mut GenerateContext {
            compilation,
            module_generator_options: self.generator_options.as_ref(),
            runtime_requirements: &mut code_generation_result.runtime_requirements,
            data: &mut code_generation_result.data,
            requested_source_type: *source_type,
          },
        )?;
        generation_result.ast_or_source = generation_result
          .ast_or_source
          .map(|i| i, |s| CachedSource::new(s).boxed());
        code_generation_result.add(*source_type, generation_result);
      }
      code_generation_result.set_hash(
        &compilation.options.output.hash_function,
        &compilation.options.output.hash_digest,
        &compilation.options.output.hash_salt,
      );
      Ok(code_generation_result)
    } else if let NormalModuleAstOrSource::BuiltFailed(error_message) = self.ast_or_source() {
      let mut code_generation_result = CodeGenerationResult::default();

      // If the module build failed and the module is able to emit JavaScript source,
      // we should emit an error message to the runtime, otherwise we do nothing.
      if self.source_types().contains(&SourceType::JavaScript) {
        code_generation_result.add(
          SourceType::JavaScript,
          AstOrSource::new(
            None,
            Some(RawSource::from(format!("throw new Error({});\n", json!(error_message))).boxed()),
          ),
        );
      }
      code_generation_result.set_hash(
        &compilation.options.output.hash_function,
        &compilation.options.output.hash_digest,
        &compilation.options.output.hash_salt,
      );
      Ok(code_generation_result)
    } else {
      Err(internal_error!(
        "Failed to generate code because ast or source is not set for module {}",
        self.request
      ))
    }
  }

  fn name_for_condition(&self) -> Option<Cow<str>> {
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

  fn get_resolve_options(&self) -> Option<&Resolve> {
    self.resolve_options.as_ref()
  }

  fn get_code_generation_dependencies(&self) -> Option<&[Box<dyn ModuleDependency>]> {
    if let Some(deps) = self.code_generation_dependencies.as_deref() && !deps.is_empty() {
      Some(deps)
    } else {
      None
    }
  }

  fn get_presentational_dependencies(&self) -> Option<&[Box<dyn Dependency>]> {
    if let Some(deps) = self.presentational_dependencies.as_deref() && !deps.is_empty() {
      Some(deps)
    } else {
      None
    }
  }

  fn get_string_replace_generation_dependencies(
    &self,
  ) -> Option<&[Box<dyn CodeReplaceSourceDependency>]> {
    if let Some(deps) = self.code_replace_source_dependencies.as_deref() && !deps.is_empty() {
      Some(deps)
    } else {
      None
    }
  }

  fn get_context(&self) -> Option<&Context> {
    Some(&self.context)
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
    if self.options.devtool.enabled() && let Some(source_map) = source_map {
      let content = content.try_into_string()?;
      return Ok(
        SourceMapSource::new(WithoutOriginalOptions {
          value: content,
          name: self.request(),
          source_map,
        })
        .boxed(),
      );
    }
    if self.options.devtool.source_map() && let Content::String(content) = content {
      return Ok(OriginalSource::new(content, self.request()).boxed());
    }
    Ok(RawSource::from(content.into_bytes()).boxed())
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
