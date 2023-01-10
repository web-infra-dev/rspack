use std::fmt::Debug;

use rspack_error::{Result, TWithDiagnosticArray};
use rspack_loader_runner::ResourceData;
use rspack_sources::Source;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{
  AssetGeneratorOptions, AssetParserOptions, AstOrSource, BuildInfo, CodeGenerationResults,
  Compilation, CompilerOptions, GenerationResult, Module, ModuleDependency, ModuleIdentifier,
  ModuleType, SourceType,
};

#[derive(Debug)]
pub struct ParseContext<'a> {
  pub source: Box<dyn Source>,
  pub module_identifier: ModuleIdentifier,
  pub module_type: &'a ModuleType,
  pub module_parser_options: Option<&'a AssetParserOptions>,
  pub resource_data: &'a ResourceData,
  pub compiler_options: &'a CompilerOptions,
  pub additional_data: Option<String>,
  pub code_generation_dependencies: &'a mut Vec<Box<dyn ModuleDependency>>,
  pub build_info: &'a mut BuildInfo,
}

#[derive(Debug)]
pub struct ParseResult {
  pub dependencies: Vec<Box<dyn ModuleDependency>>,
  pub ast_or_source: AstOrSource,
}

#[derive(Debug)]
pub struct GenerateContext<'a> {
  pub compilation: &'a Compilation,
  pub module_generator_options: Option<&'a AssetGeneratorOptions>,
  pub runtime_requirements: &'a mut HashSet<String>,
  pub data: &'a mut HashMap<String, String>,
  pub requested_source_type: SourceType,
  pub code_generation_results: &'a CodeGenerationResults,
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
