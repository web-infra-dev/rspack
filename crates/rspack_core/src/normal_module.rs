use std::{
  borrow::Cow,
  fmt::Debug,
  hash::Hash,
  sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
  },
};

use bitflags::bitflags;
use dashmap::DashMap;
use hashbrown::HashSet;
use serde_json::json;

use rspack_error::{
  Diagnostic, Error, IntoTWithDiagnosticArray, Result, Severity, TWithDiagnosticArray,
};
use rspack_loader_runner::{Content, ResourceData};
use rspack_sources::{
  BoxSource, OriginalSource, RawSource, Source, SourceExt, SourceMap, SourceMapSource,
  WithoutOriginalOptions,
};

use crate::{
  ast::javascript::Ast as JsAst, BuildContext, BuildResult, CodeGenerationResult, GenerateContext,
  Module, ParseContext, ParseResult, ParserAndGenerator,
};
use crate::{
  Compilation, CompilerOptions, Context, Dependency, ModuleAst, ModuleGraph, ModuleGraphConnection,
  ModuleType, ResolveKind, SourceType,
};

bitflags! {
  pub struct ModuleSyntax: u8 {
    const COMMONJS = 1 << 0;
    const DYNAMIC_IMPORT = 1 << 1;
  }
}

#[derive(Debug)]
pub struct ModuleGraphModule {
  // Only user defined entry module has name for now.
  pub name: Option<String>,

  // edges from module to module
  pub outgoing_connections: HashSet<u32>,
  pub incoming_connections: HashSet<u32>,

  pub id: String,
  // pub exec_order: usize,
  pub module_identifier: ModuleIdentifier,
  // TODO remove this since its included in module
  pub module_type: ModuleType,
  pub all_dependencies: Vec<Dependency>,
  pub(crate) pre_order_index: Option<usize>,
  pub post_order_index: Option<usize>,
  pub module_syntax: ModuleSyntax,
}

impl ModuleGraphModule {
  pub fn new(
    name: Option<String>,
    id: String,
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
      module_identifier,
      all_dependencies: dependencies,
      module_type,
      pre_order_index: None,
      post_order_index: None,
      module_syntax: ModuleSyntax::empty(),
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
    AstOrSource::Ast(ModuleAst::JavaScript(JsAst::new(program)))
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
pub struct NormalModule {
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
  /// Resource data (path, url, etc.)
  resource_data: ResourceData,

  /// Original content of this module, will be available after module build
  original_source: Option<Box<dyn Source>>,
  /// Built AST or source of this module (passed with loaders)
  ast_or_source: NormalModuleAstOrSource,

  options: Arc<CompilerOptions>,
  #[allow(unused)]
  debug_id: u32,
  cached_source_sizes: DashMap<SourceType, f64>,
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
      ast_or_source: NormalModuleAstOrSource::Unbuild,
      debug_id: DEBUG_ID.fetch_add(1, Ordering::Relaxed),

      options,
      cached_source_sizes: DashMap::new(),
    }
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

  fn identifier(&self) -> Cow<str> {
    Cow::Borrowed(&self.request)
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
    let mut diagnostics = Vec::new();
    let loader_result = build_context
      .loader_runner_runner
      .run(self.resource_data.clone(), build_context.resolved_loaders)
      .await;
    let (loader_result, ds) = match loader_result {
      Ok(r) => r.split_into_parts(),
      Err(e) => {
        self.ast_or_source = NormalModuleAstOrSource::BuiltFailed(e.to_string());
        return Ok(BuildResult::default().with_diagnostic(e.into()));
      }
    };
    diagnostics.extend(ds);

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
      ds,
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
    diagnostics.extend(ds);

    self.original_source = Some(original_source);
    self.ast_or_source = NormalModuleAstOrSource::new_built(ast_or_source, &diagnostics);

    Ok(BuildResult { dependencies }.with_diagnostic(diagnostics))
  }

  fn code_generation(&self, compilation: &Compilation) -> Result<CodeGenerationResult> {
    if let NormalModuleAstOrSource::BuiltSucceed(ast_or_source) = self.ast_or_source() {
      let mut code_generation_result = CodeGenerationResult::default();
      let mut runtime_requirements = HashSet::new();
      for source_type in self.source_types() {
        let generation_result = self.parser_and_generator.generate(
          ast_or_source,
          self,
          &mut GenerateContext {
            compilation,
            runtime_requirements: &mut runtime_requirements,
            requested_source_type: *source_type,
          },
        )?;
        code_generation_result.add(*source_type, generation_result);
      }
      code_generation_result
        .runtime_requirements
        .extend(runtime_requirements);
      Ok(code_generation_result)
    } else if let NormalModuleAstOrSource::BuiltFailed(error_message) = self.ast_or_source() {
      let mut code_generation_result = CodeGenerationResult::default();
      for source_type in self.source_types() {
        code_generation_result.add(
          *source_type,
          AstOrSource::Source(
            RawSource::from(format!("throw new Error({});\n", json!(error_message))).boxed(),
          ),
        );
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

impl PartialEq for NormalModule {
  fn eq(&self, other: &Self) -> bool {
    self.identifier() == other.identifier()
  }
}

impl Eq for NormalModule {}

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

impl Hash for NormalModule {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    "__rspack_internal__NormalModule".hash(state);
    self.identifier().hash(state);
  }
}
