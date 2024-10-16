use std::borrow::Cow;

use async_trait::async_trait;
use rspack_collections::{Identifiable, Identifier};
use rspack_core::{
  impl_module_meta_info, impl_source_map_config, module_update_hash,
  rspack_sources::{RawSource, Source, SourceExt},
  AsyncDependenciesBlockIdentifier, BoxDependency, BuildContext, BuildInfo, BuildMeta, BuildResult,
  ChunkUkey, CodeGenerationResult, Compilation, ConcatenationScope, Context, DependenciesBlock,
  DependencyId, FactoryMeta, LibIdentOptions, Module, ModuleIdentifier, ModuleType, RuntimeGlobals,
  RuntimeSpec, SourceType,
};
use rspack_error::{impl_empty_diagnosable_trait, Diagnostic, Result};
use rspack_util::{itoa, source_map::SourceMapKind};

use super::fallback_item_dependency::FallbackItemDependency;
use crate::utils::json_stringify;

#[impl_source_map_config]
#[derive(Debug)]
pub struct FallbackModule {
  blocks: Vec<AsyncDependenciesBlockIdentifier>,
  dependencies: Vec<DependencyId>,
  identifier: ModuleIdentifier,
  readable_identifier: String,
  lib_ident: String,
  requests: Vec<String>,
  factory_meta: Option<FactoryMeta>,
  build_info: Option<BuildInfo>,
  build_meta: Option<BuildMeta>,
}

impl FallbackModule {
  pub fn new(requests: Vec<String>) -> Self {
    let identifier = format!("fallback {}", requests.join(" "));
    let lib_ident = format!(
      "webpack/container/fallback/{}/and {} more",
      requests
        .first()
        .expect("should have at one more requests in FallbackModule"),
      itoa!(requests.len() - 1)
    );
    Self {
      blocks: Default::default(),
      dependencies: Default::default(),
      identifier: ModuleIdentifier::from(identifier.as_str()),
      readable_identifier: identifier,
      lib_ident,
      requests,
      factory_meta: None,
      build_info: None,
      build_meta: None,
      source_map_kind: SourceMapKind::empty(),
    }
  }
}

impl Identifiable for FallbackModule {
  fn identifier(&self) -> Identifier {
    self.identifier
  }
}

impl DependenciesBlock for FallbackModule {
  fn add_block_id(&mut self, block: AsyncDependenciesBlockIdentifier) {
    self.blocks.push(block)
  }

  fn get_blocks(&self) -> &[AsyncDependenciesBlockIdentifier] {
    &self.blocks
  }

  fn add_dependency_id(&mut self, dependency: DependencyId) {
    self.dependencies.push(dependency)
  }

  fn get_dependencies(&self) -> &[DependencyId] {
    &self.dependencies
  }
}

#[async_trait]
impl Module for FallbackModule {
  impl_module_meta_info!();

  fn size(&self, _source_type: Option<&SourceType>, _compilation: &Compilation) -> f64 {
    self.requests.len() as f64 * 5.0 + 42.0
  }

  fn module_type(&self) -> &ModuleType {
    &ModuleType::Fallback
  }

  fn get_diagnostics(&self) -> Vec<Diagnostic> {
    vec![]
  }

  fn source_types(&self) -> &[SourceType] {
    &[SourceType::JavaScript]
  }

  fn original_source(&self) -> Option<&dyn Source> {
    None
  }

  fn readable_identifier(&self, _context: &Context) -> Cow<str> {
    self.readable_identifier.as_str().into()
  }

  fn lib_ident(&self, _options: LibIdentOptions) -> Option<Cow<str>> {
    Some(self.lib_ident.as_str().into())
  }

  fn chunk_condition(&self, chunk: &ChunkUkey, compilation: &Compilation) -> Option<bool> {
    Some(compilation.chunk_graph.get_number_of_entry_modules(chunk) > 0)
  }

  async fn build(
    &mut self,
    _build_context: BuildContext<'_>,
    _: Option<&Compilation>,
  ) -> Result<BuildResult> {
    let build_info = BuildInfo {
      strict: true,
      ..Default::default()
    };

    let mut dependencies: Vec<BoxDependency> = Vec::new();
    for request in &self.requests {
      dependencies.push(Box::new(FallbackItemDependency::new(request.clone())))
    }

    Ok(BuildResult {
      build_info,
      build_meta: Default::default(),
      dependencies,
      blocks: Vec::new(),
      optimization_bailouts: vec![],
    })
  }

  // #[tracing::instrument("FallbackModule::code_generation", skip_all, fields(identifier = ?self.identifier()))]
  fn code_generation(
    &self,
    compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
    _: Option<ConcatenationScope>,
  ) -> Result<CodeGenerationResult> {
    let mut codegen = CodeGenerationResult::default();
    let module_graph = compilation.get_module_graph();
    codegen.runtime_requirements.insert(RuntimeGlobals::MODULE);
    codegen.runtime_requirements.insert(RuntimeGlobals::REQUIRE);
    let ids: Vec<_> = self
      .get_dependencies()
      .iter()
      .filter_map(|dep| module_graph.get_module_by_dependency_id(dep))
      .filter_map(|module| compilation.chunk_graph.get_module_id(module.identifier()))
      .collect();
    let code = format!(
      r#"
var ids = {ids};
var error, result, i = 0;
var loop = function(next) {{
  while(i < ids.length) {{
    try {{ next = {require}(ids[i++]); }} catch(e) {{ return handleError(e); }}
    if(next) return next.then ? next.then(handleResult, handleError) : handleResult(next);
  }}
  if(error) throw error;
}};
var handleResult = function(result) {{
  if(result) return result;
  return loop();
}};
var handleError = function(e) {{
  error = e;
  return loop();
}};
module.exports = loop();
"#,
      ids = json_stringify(&ids),
      require = RuntimeGlobals::REQUIRE,
    );
    codegen = codegen.with_javascript(RawSource::from(code).boxed());
    Ok(codegen)
  }

  fn update_hash(
    &self,
    hasher: &mut dyn std::hash::Hasher,
    compilation: &Compilation,
    runtime: Option<&RuntimeSpec>,
  ) -> Result<()> {
    module_update_hash(self, hasher, compilation, runtime);
    Ok(())
  }
}

impl_empty_diagnosable_trait!(FallbackModule);
