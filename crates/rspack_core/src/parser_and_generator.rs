use std::{collections::HashMap, fmt::Debug};

use rkyv::AlignedVec;
use rspack_error::{Result, TWithDiagnosticArray};
use rspack_loader_runner::{AdditionalData, ResourceData};
use rspack_sources::BoxSource;

use crate::RuntimeSpec;
use crate::{
  tree_shaking::visitor::OptimizeAnalyzeResult, BoxDependency, BuildExtraDataType, BuildInfo,
  BuildMeta, CodeGenerationData, Compilation, CompilerOptions, DependencyTemplate,
  GeneratorOptions, Module, ModuleDependency, ModuleIdentifier, ModuleType, ParserOptions,
  RuntimeGlobals, SourceType,
};

#[derive(Debug)]
pub struct ParseContext<'a> {
  pub source: BoxSource,
  pub module_identifier: ModuleIdentifier,
  pub module_type: &'a ModuleType,
  pub module_user_request: &'a str,
  pub module_parser_options: Option<&'a ParserOptions>,
  pub resource_data: &'a ResourceData,
  pub compiler_options: &'a CompilerOptions,
  pub additional_data: AdditionalData,
  pub code_generation_dependencies: &'a mut Vec<Box<dyn ModuleDependency>>,
  pub build_info: &'a mut BuildInfo,
  pub build_meta: &'a mut BuildMeta,
}

#[derive(Debug)]
pub struct ParseResult {
  pub dependencies: Vec<BoxDependency>,
  pub presentational_dependencies: Vec<Box<dyn DependencyTemplate>>,
  pub source: BoxSource,
  pub analyze_result: OptimizeAnalyzeResult,
}

#[derive(Debug)]
pub struct GenerateContext<'a> {
  pub compilation: &'a Compilation,
  pub module_generator_options: Option<&'a GeneratorOptions>,
  pub runtime_requirements: &'a mut RuntimeGlobals,
  pub data: &'a mut CodeGenerationData,
  pub requested_source_type: SourceType,
  pub runtime: Option<&'a RuntimeSpec>,
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
    source: &BoxSource,
    module: &dyn Module,
    generate_context: &mut GenerateContext,
  ) -> Result<BoxSource>;
  /// Store parser&generator data to cache
  fn store(&self, _extra_data: &mut HashMap<BuildExtraDataType, AlignedVec>) {}
  /// Resume parser&generator data from cache
  fn resume(&mut self, _extra_data: &HashMap<BuildExtraDataType, AlignedVec>) {}
}
