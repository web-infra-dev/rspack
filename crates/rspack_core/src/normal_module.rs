use rspack_error::{Error, Result};
use rspack_loader_runner::{Loader, ResourceData};
use rspack_sources::{BoxSource, Source, SourceMap};

use std::fmt::Debug;

use crate::{
  Compilation, CompilationContext, CompilerContext, Dependency, LoaderRunnerRunner, ModuleAst,
  ModuleDependency, ModuleGraph, ModuleType, ResolveKind, SourceType,
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

#[derive(Debug)]
pub struct GenerationResult {
  pub ast_or_source: AstOrSource,
  pub source_map: Option<SourceMap>,
}

#[derive(Debug)]
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
}

pub trait ParserAndGenerator: Send + Sync + Debug {
  fn parse(&self, source: &dyn Source) -> Result<AstOrSource>;
  fn generate(&self, ast_or_source: &AstOrSource) -> Result<GenerationResult>;
}

#[derive(Debug)]
pub struct NormalModule {
  request: String,
  user_request: String,
  raw_request: String,
  module_type: ModuleType,
  source_types: Vec<SourceType>,
  parser_and_generator: Box<dyn ParserAndGenerator>,
  resource_data: ResourceData,

  ast_or_source: Option<AstOrSource>,
}

#[derive(Debug)]
pub struct CodeGenerationResult {
  inner: GenerationResult,
  // TODO: add runtime requirements
  // runtime_requirements: Vec<RuntimeRequirements>,
}

impl CodeGenerationResult {
  pub fn inner(&self) -> &GenerationResult {
    &self.inner
  }
}

impl NormalModule {
  pub fn new(
    request: String,
    user_request: String,
    raw_request: String,
    module_type: impl Into<ModuleType>,
    source_types: impl IntoIterator<Item = SourceType>,
    parser_and_generator: impl ParserAndGenerator + 'static,
    resource_data: ResourceData,
  ) -> Self {
    Self {
      request,
      user_request,
      raw_request,
      module_type: module_type.into(),
      source_types: source_types.into_iter().collect(),
      parser_and_generator: Box::new(parser_and_generator),
      resource_data,

      ast_or_source: None,
    }
  }

  #[inline(always)]
  pub fn module_type(&self) -> &ModuleType {
    &self.module_type
  }

  #[inline(always)]
  pub fn source_types(&self) -> &[SourceType] {
    &self.source_types
  }

  pub fn identifier(&self) -> String {
    self.request.to_owned()
  }

  pub fn original_source(&self) -> Option<&dyn Source> {
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

  pub fn update_source_types(&mut self, source_types: impl IntoIterator<Item = SourceType>) {
    self.source_types = source_types.into_iter().collect();
  }

  pub async fn build(
    &mut self,
    loader_runner_runner: &LoaderRunnerRunner,
    resolved_loaders: impl IntoIterator<Item = &dyn Loader<CompilerContext, CompilationContext>>,
  ) -> Result<()> {
    let loader_result = loader_runner_runner
      .run(self.resource_data, resolved_loaders)
      .await?;

    let ast_or_source = self.parser_and_generator.parse(&self.request)?;
    self.ast_or_source = Some(ast_or_source);

    Ok(())
  }

  pub async fn code_generation(&self) -> Result<CodeGenerationResult> {
    if let Some(ast_or_source) = self.ast_or_source() {
      let generate_result = self.parser_and_generator.generate(ast_or_source)?;
      Ok(CodeGenerationResult {
        inner: generate_result,
      })
    } else {
      Err(Error::InternalError(
        "Failed to generate code because ast or source is not set".into(),
      ))
    }
  }
}

pub type BoxModule = Box<dyn Module>;
