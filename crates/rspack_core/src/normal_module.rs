use std::{
  collections::HashMap,
  fmt::Debug,
  sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
  },
};

use dashmap::DashMap;
use hashbrown::HashSet;

use rspack_error::{Error, IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};
use rspack_loader_runner::{Content, Loader, ResourceData};
use rspack_sources::{
  BoxSource, OriginalSource, RawSource, Source, SourceExt, SourceMap, SourceMapSource,
  WithoutOriginalOptions,
};

use crate::{
  Compilation, CompilationContext, CompilerContext, CompilerOptions, Dependency,
  LoaderRunnerRunner, ModuleAst, ModuleDependency, ModuleGraph, ModuleGraphConnection, ModuleType,
  ResolveKind, SourceType,
};

#[derive(Debug)]
pub struct ModuleGraphModule {
  // Only user defined entry module has name for now.
  pub name: Option<String>,

  // edges from module to module
  pub outgoing_connections: HashSet<u32>,
  pub incoming_connections: HashSet<u32>,

  pub id: String,
  // pub exec_order: usize,
  pub uri: String,
  // TODO: change to ModuleIdentifier
  // pub module: NormalModule,
  pub module_identifier: ModuleIdentifier,
  // TODO remove this since its included in module
  pub module_type: ModuleType,
  pub all_dependencies: Vec<Dependency>,
  pub(crate) pre_order_index: Option<usize>,
  pub post_order_index: Option<usize>,
}

impl ModuleGraphModule {
  pub fn new(
    name: Option<String>,
    id: String,
    uri: String,
    module_identifier: ModuleIdentifier,
    dependencies: Vec<Dependency>,
    module_type: ModuleType,
  ) -> Self {
    Self {
      name,

      outgoing_connections: Default::default(),
      incoming_connections: Default::default(),

      id,
      // exec_order: usize::MAX,
      uri,
      module_identifier,
      all_dependencies: dependencies,
      module_type,
      pre_order_index: None,
      post_order_index: None,
    }
  }

  pub fn add_incoming_connection(&mut self, connection_id: u32) {
    self.incoming_connections.insert(connection_id);
  }

  pub fn add_outgoing_connection(&mut self, connection_id: u32) {
    self.outgoing_connections.insert(connection_id);
  }

  pub fn incoming_connections_unordered<'m>(
    &self,
    module_graph: &'m ModuleGraph,
  ) -> Result<impl Iterator<Item = &'m ModuleGraphConnection>> {
    let result = self
      .incoming_connections
      .iter()
      .map(|connection_id| {
        module_graph
          .connection_by_connection_id(*connection_id)
          .ok_or_else(|| {
            Error::InternalError(format!(
              "connection_id_to_connection does not have connection_id: {}",
              connection_id
            ))
          })
      })
      .collect::<Result<Vec<_>>>()?
      .into_iter();

    Ok(result)
  }

  pub fn outgoing_connections_unordered<'m>(
    &self,
    module_graph: &'m ModuleGraph,
  ) -> Result<impl Iterator<Item = &'m ModuleGraphConnection>> {
    let result = self
      .outgoing_connections
      .iter()
      .map(|connection_id| {
        module_graph
          .connection_by_connection_id(*connection_id)
          .ok_or_else(|| {
            Error::InternalError(format!(
              "connection_id_to_connection does not have connection_id: {}",
              connection_id
            ))
          })
      })
      .collect::<Result<Vec<_>>>()?
      .into_iter();

    Ok(result)
  }

  // pub fn all_dependencies(&mut self) -> Vec<&ModuleDependency> {
  //   self
  //     .outgoing_connections_unordered()
  //     .map(|conn| &conn.dependency)
  //     .collect()
  // }

  pub fn depended_modules<'a>(&self, module_graph: &'a ModuleGraph) -> Vec<&'a ModuleGraphModule> {
    self
      .all_dependencies
      .iter()
      .filter(|dep| !matches!(dep.detail.kind, ResolveKind::DynamicImport))
      .filter_map(|dep| module_graph.module_by_dependency(dep))
      .collect()
  }

  pub fn dynamic_depended_modules<'a>(
    &self,
    module_graph: &'a ModuleGraph,
  ) -> Vec<&'a ModuleGraphModule> {
    self
      .all_dependencies
      .iter()
      .filter(|dep| matches!(dep.detail.kind, ResolveKind::DynamicImport))
      .filter_map(|dep| module_graph.module_by_dependency(dep))
      .collect()
  }
}

pub trait Module: Debug + Send + Sync {
  fn module_type(&self) -> ModuleType;

  fn source_types(&self) -> &[SourceType];

  fn original_source(&self) -> &dyn Source;

  fn render(
    &self,
    requested_source_type: SourceType,
    module: &ModuleGraphModule,
    compilation: &Compilation,
  ) -> Result<Option<BoxSource>>;

  fn dependencies(&mut self) -> Vec<ModuleDependency> {
    vec![]
  }
}

#[derive(Debug, Clone)]
pub struct GenerationResult {
  pub ast_or_source: AstOrSource,
}

impl From<AstOrSource> for GenerationResult {
  fn from(ast_or_source: AstOrSource) -> Self {
    GenerationResult { ast_or_source }
  }
}

#[derive(Debug, Clone)]
pub enum AstOrSource {
  Ast(ModuleAst),
  Source(BoxSource),
}

impl AstOrSource {
  pub fn is_ast(&self) -> bool {
    matches!(self, AstOrSource::Ast(_))
  }

  pub fn is_source(&self) -> bool {
    matches!(self, AstOrSource::Source(_))
  }

  pub fn as_ast(&self) -> Option<&ModuleAst> {
    match self {
      AstOrSource::Ast(ast) => Some(ast),
      _ => None,
    }
  }

  pub fn as_source(&self) -> Option<&BoxSource> {
    match self {
      AstOrSource::Source(source) => Some(source),
      _ => None,
    }
  }

  pub fn try_into_ast(self) -> Result<ModuleAst> {
    match self {
      AstOrSource::Ast(ast) => Ok(ast),
      // TODO: change to user error
      _ => Err(Error::InternalError("Failed to convert to ast".into())),
    }
  }

  pub fn try_into_source(self) -> Result<BoxSource> {
    match self {
      AstOrSource::Source(source) => Ok(source),
      // TODO: change to user error
      _ => Err(Error::InternalError("Failed to convert to source".into())),
    }
  }
}

impl From<ModuleAst> for AstOrSource {
  fn from(ast: ModuleAst) -> Self {
    AstOrSource::Ast(ast)
  }
}

impl From<swc_ecma_ast::Program> for AstOrSource {
  fn from(program: swc_ecma_ast::Program) -> Self {
    AstOrSource::Ast(ModuleAst::JavaScript(program))
  }
}

impl From<swc_css::ast::Stylesheet> for AstOrSource {
  fn from(style_sheet: swc_css::ast::Stylesheet) -> Self {
    AstOrSource::Ast(ModuleAst::Css(style_sheet))
  }
}

impl From<BoxSource> for AstOrSource {
  fn from(source: BoxSource) -> Self {
    AstOrSource::Source(source)
  }
}
#[derive(Debug)]
pub struct ParseContext<'a> {
  pub source: Box<dyn Source>,
  pub module_type: &'a ModuleType,
  pub resource_data: &'a ResourceData,
  pub compiler_options: &'a CompilerOptions,
  pub meta: Option<String>,
}

#[derive(Debug)]
pub struct ParseResult {
  pub dependencies: Vec<ModuleDependency>,
  pub ast_or_source: AstOrSource,
}

pub trait ParserAndGenerator: Send + Sync + Debug {
  fn source_types(&self) -> &[SourceType];
  fn parse(&mut self, parse_context: ParseContext) -> Result<TWithDiagnosticArray<ParseResult>>;
  fn size(&self, module: &NormalModule, source_type: &SourceType) -> f64;
  fn generate(
    &self,
    requested_source_type: SourceType,
    ast_or_source: &AstOrSource,
    module: &ModuleGraphModule,
    compilation: &Compilation,
  ) -> Result<GenerationResult>;
}

#[derive(Debug)]
pub struct NormalModule {
  request: String,
  user_request: String,
  raw_request: String,
  module_type: ModuleType,
  parser_and_generator: Box<dyn ParserAndGenerator>,
  resource_data: ResourceData,

  original_source: Option<Box<dyn Source>>,
  ast_or_source: Option<AstOrSource>,

  options: Arc<CompilerOptions>,
  #[allow(unused)]
  debug_id: u32,
  cached_source_sizes: DashMap<SourceType, f64>,

  // FIXME: dirty workaround to support external module
  skip_build: bool,
}

#[derive(Debug, Default)]
pub struct CodeGenerationResult {
  inner: HashMap<SourceType, GenerationResult>,
  // TODO: add runtime requirements
  // runtime_requirements: Vec<RuntimeRequirements>,
}

impl CodeGenerationResult {
  pub fn inner(&self) -> &HashMap<SourceType, GenerationResult> {
    &self.inner
  }

  pub fn get(&self, source_type: SourceType) -> Option<&GenerationResult> {
    self.inner.get(&source_type)
  }

  pub(super) fn add(&mut self, source_type: SourceType, generation_result: GenerationResult) {
    let result = self.inner.insert(source_type, generation_result);
    debug_assert!(result.is_none());
  }
}

#[derive(Debug)]
pub struct BuildResult {
  pub dependencies: Vec<ModuleDependency>,
}

pub struct BuildContext<'a> {
  pub loader_runner_runner: &'a LoaderRunnerRunner,
  pub resolved_loaders: Vec<&'a dyn Loader<CompilerContext, CompilationContext>>,
  pub compiler_options: &'a CompilerOptions,
}

pub type ModuleIdentifier = String;
pub static DEBUG_ID: AtomicU32 = AtomicU32::new(1);

impl NormalModule {
  pub fn new(
    request: String,
    user_request: String,
    raw_request: String,
    module_type: impl Into<ModuleType>,
    parser_and_generator: Box<dyn ParserAndGenerator>,
    resource_data: ResourceData,
    options: Arc<CompilerOptions>,
  ) -> Self {
    Self {
      request,
      user_request,
      raw_request,
      module_type: module_type.into(),
      parser_and_generator,
      resource_data,

      original_source: None,
      ast_or_source: None,
      debug_id: DEBUG_ID.fetch_add(1, Ordering::Relaxed),

      options,
      cached_source_sizes: DashMap::new(),
      skip_build: false,
    }
  }

  #[inline(always)]
  pub fn module_type(&self) -> ModuleType {
    self.module_type
  }

  #[inline(always)]
  pub fn source_types(&self) -> &[SourceType] {
    self.parser_and_generator.source_types()
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

  pub fn identifier(&self) -> ModuleIdentifier {
    self.request.to_owned()
  }

  pub fn original_source(&self) -> Option<&dyn Source> {
    self.original_source.as_deref()
  }

  // FIXME: dirty workaround to support external module
  pub fn set_skip_build(&mut self, skip_build: bool) {
    self.skip_build = skip_build;
  }

  pub fn source(&self) -> Option<&dyn Source> {
    self
      .ast_or_source()
      .and_then(|ast_or_source| ast_or_source.as_source().map(|source| source.as_ref()))
  }

  pub fn ast(&self) -> Option<&ModuleAst> {
    self
      .ast_or_source()
      .and_then(|ast_or_source| ast_or_source.as_ast())
  }

  pub fn ast_or_source(&self) -> Option<&AstOrSource> {
    self.ast_or_source.as_ref()
  }

  pub fn size(&self, source_type: &SourceType) -> f64 {
    if let Some(size_ref) = self.cached_source_sizes.get(source_type) {
      *size_ref
    } else {
      let size = f64::max(1.0, self.parser_and_generator.size(self, source_type));
      self.cached_source_sizes.insert(*source_type, size);
      size
    }
  }

  pub async fn build(
    &mut self,
    build_context: BuildContext<'_>,
  ) -> Result<TWithDiagnosticArray<BuildResult>> {
    // FIXME: dirty workaround for external module
    if self.skip_build {
      let (parse_result, diagnostics) = self
        .parser_and_generator
        .parse(ParseContext {
          source: RawSource::from("").boxed(),
          module_type: &self.module_type,
          resource_data: &self.resource_data,
          compiler_options: build_context.compiler_options,
          meta: None,
        })?
        .split_into_parts();

      self.ast_or_source = Some(parse_result.ast_or_source);

      return Ok(
        BuildResult {
          dependencies: parse_result.dependencies,
        }
        .with_diagnostic(diagnostics),
      );
    }

    let loader_result = build_context
      .loader_runner_runner
      .run(self.resource_data.clone(), build_context.resolved_loaders)
      .await?;

    let original_source = self.create_source(
      &self.resource_data.resource,
      loader_result.content,
      loader_result.source_map,
    )?;

    let (
      ParseResult {
        ast_or_source,
        dependencies,
      },
      diagnostics,
    ) = self
      .parser_and_generator
      .parse(ParseContext {
        source: original_source.clone(),
        module_type: &self.module_type,
        resource_data: &self.resource_data,
        compiler_options: build_context.compiler_options,
        meta: loader_result.meta,
      })?
      .split_into_parts();

    self.original_source = Some(original_source);
    self.ast_or_source = Some(ast_or_source);

    Ok(BuildResult { dependencies }.with_diagnostic(diagnostics))
  }

  pub fn code_generation(
    &self,
    module_graph_module: &ModuleGraphModule,
    compilation: &Compilation,
  ) -> Result<CodeGenerationResult> {
    if let Some(ast_or_source) = self.ast_or_source() {
      let mut code_generation_result = CodeGenerationResult::default();

      for source_type in self.source_types() {
        let generation_result = self.parser_and_generator.generate(
          *source_type,
          ast_or_source,
          module_graph_module,
          compilation,
        )?;

        code_generation_result.add(*source_type, generation_result);
      }

      Ok(code_generation_result)
    } else {
      Err(Error::InternalError(format!(
        "Failed to generate code because ast or source is not set for module {}",
        self.request
      )))
    }
  }
}

impl std::cmp::PartialEq for NormalModule {
  fn eq(&self, other: &Self) -> bool {
    self.identifier() == other.identifier()
  }
}

impl std::cmp::Eq for NormalModule {}

impl NormalModule {
  fn create_source(
    &self,
    uri: &str,
    content: Content,
    source_map: Option<SourceMap>,
  ) -> Result<BoxSource> {
    if self.options.devtool.enabled() && let Some(source_map) = source_map {
      let content = content.try_into_string()?;
      return Ok(
        SourceMapSource::new(WithoutOriginalOptions {
          value: content,
          name: uri,
          source_map,
        })
        .boxed(),
      );
    }
    if self.options.devtool.source_map() && let Content::String(content) = content {
      return Ok(OriginalSource::new(content, uri).boxed());
    }
    Ok(RawSource::from(content.into_bytes()).boxed())
  }
}

pub type BoxModule = Box<dyn Module>;
