use std::{borrow::Cow, hash::Hash};

use async_trait::async_trait;
use rspack_error::{IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};
use rspack_hash::RspackHash;
use rspack_identifier::{Identifiable, Identifier};
use rspack_sources::Source;

use super::{
  provide_for_shared_dependency::ProvideForSharedDependency,
  provide_shared_plugin::ProvideVersion,
  share_runtime_module::{CodeGenerationDataShareInit, ShareInitData},
};
use crate::{
  async_module_factory, sync_module_factory, AsyncDependenciesBlock,
  AsyncDependenciesBlockIdentifier, BoxDependency, BuildContext, BuildInfo, BuildResult,
  CodeGenerationResult, Compilation, Context, DependenciesBlock, DependencyId, LibIdentOptions,
  Module, ModuleIdentifier, ModuleType, RuntimeGlobals, RuntimeSpec, SourceType,
};

#[derive(Debug)]
pub struct ProvideSharedModule {
  blocks: Vec<AsyncDependenciesBlockIdentifier>,
  dependencies: Vec<DependencyId>,
  identifier: ModuleIdentifier,
  lib_ident: String,
  readable_identifier: String,
  name: String,
  share_scope: String,
  version: ProvideVersion,
  request: String,
  eager: bool,
}

impl ProvideSharedModule {
  pub fn new(
    share_scope: String,
    name: String,
    version: ProvideVersion,
    request: String,
    eager: bool,
  ) -> Self {
    let identifier = format!(
      "provide shared module ({}) {}@{} = {}",
      &share_scope, &name, &version, &request
    );
    Self {
      blocks: Vec::new(),
      dependencies: Vec::new(),
      identifier: ModuleIdentifier::from(identifier.clone()),
      lib_ident: format!("webpack/sharing/provide/{}/{}", &share_scope, &name),
      readable_identifier: identifier,
      name,
      share_scope,
      version,
      request,
      eager,
    }
  }
}

impl Identifiable for ProvideSharedModule {
  fn identifier(&self) -> Identifier {
    self.identifier
  }
}

impl DependenciesBlock for ProvideSharedModule {
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
impl Module for ProvideSharedModule {
  fn size(&self, _source_type: &SourceType) -> f64 {
    42.0
  }

  fn module_type(&self) -> &ModuleType {
    &ModuleType::ProvideShared
  }

  fn source_types(&self) -> &[SourceType] {
    &[SourceType::ShareInit]
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

  async fn build(
    &mut self,
    build_context: BuildContext<'_>,
  ) -> Result<TWithDiagnosticArray<BuildResult>> {
    let mut hasher = RspackHash::from(&build_context.compiler_options.output);
    self.update_hash(&mut hasher);
    let hash = hasher.digest(&build_context.compiler_options.output.hash_digest);

    let mut blocks = vec![];
    let mut dependencies = vec![];
    let dep = Box::new(ProvideForSharedDependency::new(self.request.clone()));
    if self.eager {
      dependencies.push(dep as BoxDependency);
    } else {
      let mut block = AsyncDependenciesBlock::new(self.identifier, "", None);
      block.add_dependency(dep);
      blocks.push(block);
    }

    Ok(
      BuildResult {
        build_info: BuildInfo {
          hash: Some(hash),
          strict: true,
          ..Default::default()
        },
        build_meta: Default::default(),
        dependencies,
        blocks,
        ..Default::default()
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
    let mut code_generation_result = CodeGenerationResult::default();
    code_generation_result
      .runtime_requirements
      .insert(RuntimeGlobals::INITIALIZE_SHARING);
    let init = format!(
      "register({}, {}, {}{})",
      serde_json::to_string(&self.name).expect("ProvideSharedModule name should able to json to_string"),
      serde_json::to_string(&self.version.to_string()).expect("ProvideVersion::Version should able to json to_string in ProvideSharedModule::code_generation"),
      if self.eager {
        sync_module_factory(
          &self.get_dependencies()[0],
          &self.request,
          compilation,
          &mut code_generation_result.runtime_requirements,
        )
      } else {
        async_module_factory(
          &self.get_blocks()[0],
          &self.request,
          compilation,
          &mut code_generation_result.runtime_requirements,
        )
      },
      if self.eager { ", 1" } else { "" },
    );
    code_generation_result
      .data
      .insert(CodeGenerationDataShareInit {
        items: vec![ShareInitData {
          share_scope: self.share_scope.clone(),
          init_stage: 10,
          init,
        }],
      });
    Ok(code_generation_result)
  }
}

impl Hash for ProvideSharedModule {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    "__rspack_internal__ProvideSharedModule".hash(state);
    self.identifier().hash(state);
  }
}

impl PartialEq for ProvideSharedModule {
  fn eq(&self, other: &Self) -> bool {
    self.identifier() == other.identifier()
  }
}

impl Eq for ProvideSharedModule {}
