use std::{
  borrow::Cow,
  fmt::Debug,
  hash::Hash,
  path::Path,
  sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
  },
};

use bitflags::bitflags;
use dashmap::DashMap;
use hashbrown::{HashMap, HashSet};
use serde_json::json;
use ustr::ustr;

use rspack_error::{
  internal_error, Diagnostic, Error, IntoTWithDiagnosticArray, Result, Severity,
  TWithDiagnosticArray,
};
use rspack_loader_runner::{Content, ResourceData};
use rspack_sources::{
  BoxSource, OriginalSource, RawSource, Source, SourceExt, SourceMap, SourceMapSource,
  WithoutOriginalOptions,
};

use crate::{
  contextify, identifier::Identifiable, BuildContext, BuildResult, ChunkGraph,
  CodeGenerationResult, Compilation, CompilerOptions, Context, Dependency, GenerateContext,
  LibIdentOptions, Module, ModuleAst, ModuleGraph, ModuleGraphConnection, ModuleIdentifier,
  ModuleType, ParseContext, ParseResult, ParserAndGenerator, Resolve, ResolveKind, SourceType,
};

bitflags! {
  pub struct ModuleSyntax: u8 {
    const COMMONJS = 1 << 0;
    const DYNAMIC_IMPORT = 1 << 1;
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

  pub fn readable_identifier<'a>(
    &self,
    module_graph: &'a ModuleGraph,
    context: &Context,
  ) -> Option<Cow<'a, str>> {
    if let Some(id) = self.identifier() && let Some(issuer) = module_graph.module_by_identifier(id) {
      Some(issuer.readable_identifier(context))
    } else {
      None
    }
  }
}

#[derive(Debug)]
pub struct ModuleGraphModule {
  // Only user defined entry module has name for now.
  pub name: Option<String>,

  // edges from module to module
  pub outgoing_connections: HashSet<u32>,
  pub incoming_connections: HashSet<u32>,

  issuer: ModuleIssuer,

  // pub exec_order: usize,
  pub module_identifier: ModuleIdentifier,
  // TODO remove this since its included in module
  pub module_type: ModuleType,
  pub all_dependencies: Vec<Dependency>,
  pub(crate) pre_order_index: Option<usize>,
  pub post_order_index: Option<usize>,
  pub module_syntax: ModuleSyntax,
  pub used: bool,
}

impl ModuleGraphModule {
  pub fn new(
    name: Option<String>,
    module_identifier: ModuleIdentifier,
    dependencies: Vec<Dependency>,
    module_type: ModuleType,
    default_used: bool,
  ) -> Self {
    Self {
      name,

      outgoing_connections: Default::default(),
      incoming_connections: Default::default(),

      issuer: ModuleIssuer::Unset,
      // exec_order: usize::MAX,
      module_identifier,
      all_dependencies: dependencies,
      module_type,
      pre_order_index: None,
      post_order_index: None,
      module_syntax: ModuleSyntax::empty(),
      used: default_used,
    }
  }

  pub fn id<'chunk_graph>(&self, chunk_graph: &'chunk_graph ChunkGraph) -> &'chunk_graph str {
    chunk_graph
      .get_module_id(&self.module_identifier)
      .as_ref()
      .expect("module id not found")
      .as_str()
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
            Error::InternalError(internal_error!(format!(
              "connection_id_to_connection does not have connection_id: {}",
              connection_id
            )))
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
            Error::InternalError(internal_error!(format!(
              "connection_id_to_connection does not have connection_id: {}",
              connection_id
            )))
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

  pub fn set_issuer_if_unset(&mut self, issuer: Option<ModuleIdentifier>) {
    if matches!(self.issuer, ModuleIssuer::Unset) {
      self.issuer = ModuleIssuer::from_identifier(issuer);
    }
  }

  pub fn get_issuer(&self) -> &ModuleIssuer {
    &self.issuer
  }
}

#[derive(Debug, Clone, Hash)]
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
      _ => Err(Error::InternalError(internal_error!(
        "Failed to convert to ast".into()
      ))),
    }
  }

  pub fn try_into_source(self) -> Result<BoxSource> {
    match self {
      AstOrSource::Source(source) => Ok(source),
      // TODO: change to user error
      _ => Err(Error::InternalError(internal_error!(
        "Failed to convert to source".into()
      ))),
    }
  }
}

impl From<ModuleAst> for AstOrSource {
  fn from(ast: ModuleAst) -> Self {
    AstOrSource::Ast(ast)
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

  resolve_options: Option<Resolve>,

  options: Arc<CompilerOptions>,
  #[allow(unused)]
  debug_id: u32,
  cached_source_sizes: DashMap<SourceType, f64>,

  code_generation_dependencies: Option<Vec<Dependency>>,
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

pub static DEBUG_ID: AtomicU32 = AtomicU32::new(1);

impl NormalModule {
  #[allow(clippy::too_many_arguments)]
  pub fn new(
    request: String,
    user_request: String,
    raw_request: String,
    module_type: impl Into<ModuleType>,
    parser_and_generator: Box<dyn ParserAndGenerator>,
    resource_data: ResourceData,
    resolve_options: Option<Resolve>,
    options: Arc<CompilerOptions>,
  ) -> Self {
    Self {
      request,
      user_request,
      raw_request,
      module_type: module_type.into(),
      parser_and_generator,
      resource_data,
      resolve_options,

      original_source: None,
      ast_or_source: NormalModuleAstOrSource::Unbuild,
      debug_id: DEBUG_ID.fetch_add(1, Ordering::Relaxed),

      options,
      cached_source_sizes: DashMap::new(),
      code_generation_dependencies: None,
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

impl Identifiable for NormalModule {
  fn identifier(&self) -> ModuleIdentifier {
    ustr(&format!("{}|{}", self.module_type, &self.request))
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

  fn context(&self) -> Option<&Path> {
    Path::new(&self.resource_resolved_data().resource_path).parent()
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
    let mut code_generation_dependencies = Vec::new();

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
        additional_data: loader_result.additional_data,
        code_generation_dependencies: &mut code_generation_dependencies,
      })?
      .split_into_parts();
    diagnostics.extend(ds);

    self.original_source = Some(original_source);
    self.ast_or_source = NormalModuleAstOrSource::new_built(ast_or_source, &diagnostics);
    self.code_generation_dependencies = Some(
      code_generation_dependencies
        .into_iter()
        .map(|d| Dependency {
          parent_module_identifier: Some(self.identifier()),
          detail: d,
        })
        .collect(),
    );

    Ok(
      BuildResult {
        cacheable: loader_result.cacheable,
        build_dependencies: vec![],
        dependencies,
      }
      .with_diagnostic(diagnostics),
    )
  }

  fn code_generation(&self, compilation: &Compilation) -> Result<CodeGenerationResult> {
    if let NormalModuleAstOrSource::BuiltSucceed(ast_or_source) = self.ast_or_source() {
      let mut code_generation_result = CodeGenerationResult::default();
      let mut data = HashMap::new();
      let mut runtime_requirements = HashSet::new();
      for source_type in self.source_types() {
        let generation_result = self.parser_and_generator.generate(
          ast_or_source,
          self,
          &mut GenerateContext {
            compilation,
            runtime_requirements: &mut runtime_requirements,
            data: &mut data,
            requested_source_type: *source_type,
            code_generation_results: &compilation.code_generation_results,
          },
        )?;
        code_generation_result.add(*source_type, generation_result);
      }
      code_generation_result.data.extend(data);
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
      Err(Error::InternalError(internal_error!(format!(
        "Failed to generate code because ast or source is not set for module {}",
        self.request
      ))))
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

  fn get_code_generation_dependencies(&self) -> Option<&[Dependency]> {
    if let Some(deps) = self.code_generation_dependencies.as_deref() && !deps.is_empty() {
      Some(deps)
    } else {
      None
    }
  }

  fn get_resolve_options(&self) -> Option<&Resolve> {
    self.resolve_options.as_ref()
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
    if let Some(original_source) = &self.original_source {
      original_source.hash(state);
    }
  }
}
