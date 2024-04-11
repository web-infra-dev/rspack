use std::fmt::Debug;
use std::path::PathBuf;
use std::sync::Arc;

use rspack_error::Diagnostic;
use rspack_loader_runner::ResourceData;
use rspack_sources::BoxSource;
use rustc_hash::FxHashSet as HashSet;

use crate::{
  Context, ContextModuleFactory, DependencyCategory, DependencyType, ErrorSpan, FactoryMeta,
  ModuleIdentifier, NormalModuleFactory, Resolve,
};

#[derive(Debug)]
pub struct AssetEmittedInfo {
  pub source: BoxSource,
  pub output_path: PathBuf,
  pub target_path: PathBuf,
}

#[derive(Debug)]
pub struct NormalModuleCreateData {
  pub raw_request: String,
  pub request: String,
  pub user_request: String,
  pub resource_resolve_data: ResourceData,
  pub match_resource: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BeforeResolveArgs {
  pub request: String,
  pub context: String,
}

#[derive(Debug)]
pub struct AfterResolveArgs<'a> {
  pub request: &'a str,
  pub context: &'a str,
  pub file_dependencies: &'a HashSet<PathBuf>,
  pub context_dependencies: &'a HashSet<PathBuf>,
  pub missing_dependencies: &'a HashSet<PathBuf>,
  pub factory_meta: &'a FactoryMeta,
  pub diagnostics: &'a mut Vec<Diagnostic>,
  pub create_data: Option<NormalModuleCreateData>,
}

#[derive(Debug)]
pub struct ResolveArgs<'a> {
  pub importer: Option<&'a ModuleIdentifier>,
  pub issuer: Option<&'a str>,
  pub context: Context,
  pub specifier: &'a str,
  pub dependency_type: &'a DependencyType,
  pub dependency_category: &'a DependencyCategory,
  pub span: Option<ErrorSpan>,
  pub resolve_options: Option<Box<Resolve>>,
  pub resolve_to_context: bool,
  pub optional: bool,
  pub file_dependencies: &'a mut HashSet<PathBuf>,
  pub missing_dependencies: &'a mut HashSet<PathBuf>,
}

#[derive(Debug)]
pub struct CompilationParams {
  pub normal_module_factory: Arc<NormalModuleFactory>,
  pub context_module_factory: Arc<ContextModuleFactory>,
}
