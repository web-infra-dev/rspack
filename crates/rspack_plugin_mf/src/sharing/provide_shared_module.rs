use std::{borrow::Cow, hash::Hash};

use async_trait::async_trait;
use rspack_core::{
  async_module_factory, impl_module_meta_info, impl_source_map_config, rspack_sources::Source,
  sync_module_factory, AsyncDependenciesBlock, AsyncDependenciesBlockIdentifier, BoxDependency,
  BuildContext, BuildInfo, BuildMeta, BuildResult, CodeGenerationResult, Compilation,
  ConcatenationScope, Context, DependenciesBlock, DependencyId, FactoryMeta, LibIdentOptions,
  Module, ModuleIdentifier, ModuleType, RuntimeGlobals, RuntimeSpec, SourceType,
};
use rspack_error::{impl_empty_diagnosable_trait, Diagnostic, Result};
use rspack_hash::RspackHash;
use rspack_identifier::{Identifiable, Identifier};
use rspack_util::source_map::SourceMapKind;

use super::{
  provide_for_shared_dependency::ProvideForSharedDependency,
  provide_shared_plugin::ProvideVersion,
  share_runtime_module::{
    CodeGenerationDataShareInit, DataInitInfo, ProvideSharedInfo, ShareInitData,
  },
};

#[impl_source_map_config]
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
  factory_meta: Option<FactoryMeta>,
  build_info: Option<BuildInfo>,
  build_meta: Option<BuildMeta>,
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
      factory_meta: None,
      build_info: None,
      build_meta: None,
      source_map_kind: SourceMapKind::None,
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
  impl_module_meta_info!();

  fn size(&self, _source_type: &SourceType) -> f64 {
    42.0
  }

  fn get_diagnostics(&self) -> Vec<Diagnostic> {
    vec![]
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
    _: Option<&Compilation>,
  ) -> Result<BuildResult> {
    let mut hasher = RspackHash::from(&build_context.compiler_options.output);
    self.update_hash(&mut hasher);
    let hash = hasher.digest(&build_context.compiler_options.output.hash_digest);

    let mut blocks = vec![];
    let mut dependencies = vec![];
    let dep = Box::new(ProvideForSharedDependency::new(self.request.clone()));
    if self.eager {
      dependencies.push(dep as BoxDependency);
    } else {
      let block = AsyncDependenciesBlock::new(self.identifier, None, None, vec![dep]);
      blocks.push(block);
    }

    Ok(BuildResult {
      build_info: BuildInfo {
        hash: Some(hash),
        strict: true,
        ..Default::default()
      },
      build_meta: Default::default(),
      dependencies,
      blocks,
      ..Default::default()
    })
  }

  #[allow(clippy::unwrap_in_result)]
  fn code_generation(
    &self,
    compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
    _: Option<ConcatenationScope>,
  ) -> Result<CodeGenerationResult> {
    let mut code_generation_result = CodeGenerationResult::default();
    code_generation_result
      .runtime_requirements
      .insert(RuntimeGlobals::INITIALIZE_SHARING);
    let factory = if self.eager {
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
    };
    code_generation_result
      .data
      .insert(CodeGenerationDataShareInit {
        items: vec![ShareInitData {
          share_scope: self.share_scope.clone(),
          init_stage: 10,
          init: DataInitInfo::ProvideSharedInfo(ProvideSharedInfo {
            name: self.name.clone(),
            version: self.version.clone(),
            factory,
            eager: self.eager,
          }),
        }],
      });
    Ok(code_generation_result)
  }
}

impl_empty_diagnosable_trait!(ProvideSharedModule);

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
