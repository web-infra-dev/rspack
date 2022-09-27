use rspack_error::{Error, IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};
use rspack_loader_runner::{Content, Loader, ResourceData};
use rspack_sources::{
  BoxSource, OriginalSource, RawSource, Source, SourceExt, SourceMap, SourceMapSource,
  WithoutOriginalOptions,
};

use std::{collections::HashMap, fmt::Debug, sync::Arc};

use crate::{
  Compilation, CompilationContext, CompilerContext, CompilerOptions, Dependency,
  LoaderRunnerRunner, ModuleAst, ModuleDependency, ModuleGraph, ModuleType, ResolveKind,
  SourceType,
};

#[derive(Debug)]
pub struct ModuleGraphModule {
  // Only user defined entry module has name for now.
  pub name: Option<String>,
  pub id: String,
  // pub exec_order: usize,
  pub uri: String,
  pub module: NormalModule,
  // TODO remove this since its included in module
  pub module_type: ModuleType,
  all_dependencies: Vec<Dependency>,
  pub(crate) pre_order_index: Option<usize>,
  pub post_order_index: Option<usize>,
}

impl ModuleGraphModule {
  pub fn new(
    name: Option<String>,
    id: String,
    uri: String,
    module: NormalModule,
    dependencies: Vec<Dependency>,
    module_type: ModuleType,
  ) -> Self {
    Self {
      name,
      id,
      // exec_order: usize::MAX,
      uri,
      module,
      all_dependencies: dependencies,
      module_type,
      pre_order_index: None,
      post_order_index: None,
    }
  }

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

impl NormalModule {
  pub fn new(
    request: String,
    user_request: String,
    raw_request: String,
    module_type: impl Into<ModuleType>,
    parser_and_generator: Box<dyn ParserAndGenerator>,
    resource_data: ResourceData,
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
    }
  }

  #[inline(always)]
  pub fn module_type(&self) -> &ModuleType {
    &self.module_type
  }

  #[inline(always)]
  pub fn source_types(&self) -> &[SourceType] {
    self.parser_and_generator.source_types()
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

  pub fn identifier(&self) -> String {
    self.request.to_owned()
  }

  pub fn original_source(&self) -> Option<&dyn Source> {
    self.original_source.as_deref()
  }

  pub fn source(&self) -> Option<&dyn Source> {
    self
      .ast_or_source()
      .map(|ast_or_source| ast_or_source.as_source().map(|source| source.as_ref()))
      .flatten()
  }

  pub fn ast(&self) -> Option<&ModuleAst> {
    self
      .ast_or_source()
      .map(|ast_or_source| ast_or_source.as_ast())
      .flatten()
  }

  pub fn ast_or_source(&self) -> Option<&AstOrSource> {
    self.ast_or_source.as_ref()
  }

  pub async fn build(
    &mut self,
    build_context: BuildContext<'_>,
  ) -> Result<TWithDiagnosticArray<BuildResult>> {
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

    Ok(IntoTWithDiagnosticArray::with_diagnostic(
      BuildResult { dependencies },
      diagnostics,
    ))
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
      Err(Error::InternalError(
        "Failed to generate code because ast or source is not set".into(),
      ))
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
    // TODO: change back to self.context.options.devtool
    match (true, content, source_map) {
      (true, content, Some(map)) => {
        let content = content.try_into_string()?;
        Ok(
          SourceMapSource::new(WithoutOriginalOptions {
            value: content,
            name: uri,
            source_map: map,
          })
          .boxed(),
        )
      }
      (true, Content::String(content), None) => Ok(OriginalSource::new(content, uri).boxed()),
      (true, Content::Buffer(content), None) => Ok(RawSource::from(content).boxed()),
      (_, Content::String(content), _) => Ok(RawSource::from(content).boxed()),
      (_, Content::Buffer(content), _) => Ok(RawSource::from(content).boxed()),
    }
  }
}

pub type BoxModule = Box<dyn Module>;
