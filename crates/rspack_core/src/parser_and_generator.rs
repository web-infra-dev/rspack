use std::fmt::Debug;

use rspack_error::{Result, TWithDiagnosticArray};
use rspack_loader_runner::ResourceData;
use rspack_sources::BoxSource;

use crate::{
  AssetGeneratorOptions, AssetParserOptions, AstOrSource, BuildInfo, BuildMeta, CodeGenerationData,
  Compilation, CompilerOptions, Dependency, GenerationResult, Module, ModuleDependency,
  ModuleIdentifier, ModuleType, RuntimeGlobals, SourceType,
};

#[derive(Debug)]
pub struct ParseContext<'a> {
  pub source: BoxSource,
  pub module_identifier: ModuleIdentifier,
  pub module_type: &'a ModuleType,
  pub module_parser_options: Option<&'a AssetParserOptions>,
  pub resource_data: &'a ResourceData,
  pub compiler_options: &'a CompilerOptions,
  pub additional_data: Option<String>,
  pub code_generation_dependencies: &'a mut Vec<Box<dyn ModuleDependency>>,
  pub build_info: &'a mut BuildInfo,
  pub build_meta: &'a mut BuildMeta,
}

#[derive(Debug)]
pub struct ParseResult {
  pub dependencies: Vec<Box<dyn ModuleDependency>>,
  pub presentational_dependencies: Vec<Box<dyn Dependency>>,
  pub ast_or_source: AstOrSource,
}

#[derive(Debug)]
pub struct GenerateContext<'a> {
  pub compilation: &'a Compilation,
  pub module_generator_options: Option<&'a AssetGeneratorOptions>,
  pub runtime_requirements: &'a mut RuntimeGlobals,
  pub data: &'a mut CodeGenerationData,
  pub requested_source_type: SourceType,
}

pub trait ParserAndGenerator: Send + Sync + Debug {
  /// The source types that the generator can generate (the source types you can make requests for)
  fn source_types(&self) -> &[SourceType];
  /// Parse the source and return the dependencies and the ast or source
  fn parse(&mut self, parse_context: ParseContext) -> Result<TWithDiagnosticArray<ParseResult>>;
  /// Size of the original source
  fn size(&self, module: &dyn Module, source_type: &SourceType) -> f64;
  /// Generate source or AST based on the built source or AST
  fn generate(
    &self,
    ast_or_source: &AstOrSource,
    module: &dyn Module,
    generate_context: &mut GenerateContext,
  ) -> Result<GenerationResult>;
}
