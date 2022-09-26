use rspack_error::{Error, Result};
use rspack_loader_runner::{Content, Loader, ResourceData};
use rspack_sources::{
  BoxSource, OriginalSource, RawSource, Source, SourceExt, SourceMap, SourceMapSource,
  WithoutOriginalOptions,
};

use std::{collections::HashMap, fmt::Debug};

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

#[derive(Debug, Clone)]
pub struct GenerationResult {
  pub ast_or_source: AstOrSource,
  pub source_map: Option<SourceMap>,
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

pub trait ParserAndGenerator: Send + Sync + Debug {
  fn parse(&self, source: &dyn Source) -> Result<AstOrSource>;
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
  source_types: Vec<SourceType>,
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

  pub fn code_generation_result(&self, source_type: SourceType) -> Option<&GenerationResult> {
    self.inner.get(&source_type)
  }

  pub(super) fn add(&mut self, source_type: SourceType, generation_result: GenerationResult) {
    let result = self.inner.insert(source_type, generation_result);
    debug_assert!(result.is_none());
  }
}

impl NormalModule {
  pub fn new(
    request: String,
    user_request: String,
    raw_request: String,
    module_type: impl Into<ModuleType>,
    source_types: impl IntoIterator<Item = SourceType>,
    parser_and_generator: Box<dyn ParserAndGenerator>,
    resource_data: ResourceData,
  ) -> Self {
    Self {
      request,
      user_request,
      raw_request,
      module_type: module_type.into(),
      source_types: source_types.into_iter().collect(),
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
    &self.source_types
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

  pub fn update_source_types(&mut self, source_types: impl IntoIterator<Item = SourceType>) {
    self.source_types = source_types.into_iter().collect();
  }

  pub async fn build(
    &mut self,
    loader_runner_runner: &LoaderRunnerRunner,
    resolved_loaders: impl IntoIterator<Item = &dyn Loader<CompilerContext, CompilationContext>>,
  ) -> Result<()> {
    let loader_result = loader_runner_runner
      .run(self.resource_data.clone(), resolved_loaders)
      .await?;

    let original_source = self.create_source(
      &self.resource_data.resource,
      loader_result.content,
      loader_result.source_map,
    )?;

    let ast_or_source = self.parser_and_generator.parse(&original_source)?;

    self.original_source = Some(original_source);
    self.ast_or_source = Some(ast_or_source);

    Ok(())
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

  // TODO: temporary workaround for dependency
  pub fn dependencies(&self) -> Vec<ModuleDependency> {
    vec![]
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
