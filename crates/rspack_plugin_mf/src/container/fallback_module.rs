use std::borrow::Cow;
use std::hash::Hash;

use async_trait::async_trait;
use rspack_core::{
  rspack_sources::{RawSource, Source, SourceExt},
  AsyncDependenciesBlockIdentifier, BoxDependency, BuildContext, BuildInfo, BuildResult, ChunkUkey,
  CodeGenerationResult, Compilation, Context, DependenciesBlock, DependencyId, LibIdentOptions,
  Module, ModuleIdentifier, ModuleType, RuntimeGlobals, RuntimeSpec, SourceType,
};
use rspack_error::{IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};
use rspack_hash::RspackHash;
use rspack_identifier::{Identifiable, Identifier};

use super::fallback_item_dependency::FallbackItemDependency;
use crate::utils::json_stringify;

#[derive(Debug)]
pub struct FallbackModule {
  blocks: Vec<AsyncDependenciesBlockIdentifier>,
  dependencies: Vec<DependencyId>,
  identifier: ModuleIdentifier,
  readable_identifier: String,
  lib_ident: String,
  requests: Vec<String>,
}

impl FallbackModule {
  pub fn new(requests: Vec<String>) -> Self {
    let identifier = format!("fallback {}", requests.join(" "));
    let lib_ident = format!(
      "webpack/container/fallback/{}/and {} more",
      requests
        .first()
        .expect("should have at one more requests in FallbackModule"),
      requests.len() - 1
    );
    Self {
      blocks: Default::default(),
      dependencies: Default::default(),
      identifier: ModuleIdentifier::from(identifier.as_str()),
      readable_identifier: identifier,
      lib_ident,
      requests,
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
  fn size(&self, _source_type: &SourceType) -> f64 {
    self.requests.len() as f64 * 5.0 + 42.0
  }

  fn module_type(&self) -> &ModuleType {
    &ModuleType::Fallback
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
    build_context: BuildContext<'_>,
  ) -> Result<TWithDiagnosticArray<BuildResult>> {
    let mut hasher = RspackHash::from(&build_context.compiler_options.output);
    self.update_hash(&mut hasher);

    let build_info = BuildInfo {
      strict: true,
      hash: Some(hasher.digest(&build_context.compiler_options.output.hash_digest)),
      ..Default::default()
    };

    let mut dependencies: Vec<BoxDependency> = Vec::new();
    for request in &self.requests {
      dependencies.push(Box::new(FallbackItemDependency::new(request.clone())))
    }

    Ok(
      BuildResult {
        build_info,
        build_meta: Default::default(),
        dependencies,
        blocks: Vec::new(),
        analyze_result: Default::default(),
      }
      .with_empty_diagnostic(),
    )
  }

  #[allow(clippy::unwrap_in_result)]
  fn code_generation(
    &self,
    compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) -> Result<CodeGenerationResult> {
    let mut codegen = CodeGenerationResult::default();
    codegen.runtime_requirements.insert(RuntimeGlobals::MODULE);
    let ids: Vec<_> = self
      .get_dependencies()
      .iter()
      .filter_map(|dep| compilation.module_graph.get_module(dep))
      .filter_map(|module| {
        compilation
          .chunk_graph
          .get_module_id(module.identifier())
          .as_deref()
      })
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
}

impl Hash for FallbackModule {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    "__rspack_internal__FallbackModule".hash(state);
    self.identifier().hash(state);
  }
}

impl PartialEq for FallbackModule {
  fn eq(&self, other: &Self) -> bool {
    self.identifier() == other.identifier()
  }
}

impl Eq for FallbackModule {}
