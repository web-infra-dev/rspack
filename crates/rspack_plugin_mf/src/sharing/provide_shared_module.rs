use std::borrow::Cow;

use async_trait::async_trait;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_collections::{Identifiable, Identifier};
use rspack_core::{
  AsyncDependenciesBlock, AsyncDependenciesBlockIdentifier, BoxDependency, BuildContext, BuildInfo,
  BuildMeta, BuildResult, CodeGenerationResult, Compilation, ConcatenationScope, Context,
  DependenciesBlock, DependencyId, FactoryMeta, LibIdentOptions, Module, ModuleGraph,
  ModuleIdentifier, ModuleType, RuntimeGlobals, RuntimeSpec, SourceType, impl_module_meta_info,
  impl_source_map_config, module_update_hash, rspack_sources::BoxSource,
};
use rspack_error::{Result, impl_empty_diagnosable_trait};
use rspack_hash::{RspackHash, RspackHashDigest};
use rspack_util::source_map::SourceMapKind;

use super::{
  provide_for_shared_dependency::ProvideForSharedDependency,
  provide_shared_plugin::ProvideVersion,
  share_runtime_module::{
    CodeGenerationDataShareInit, DataInitInfo, ProvideSharedInfo, ShareInitData,
  },
};
use crate::ConsumeVersion;

#[impl_source_map_config]
#[cacheable]
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
  singleton: Option<bool>,
  required_version: Option<ConsumeVersion>,
  strict_version: Option<bool>,
  factory_meta: Option<FactoryMeta>,
  build_info: BuildInfo,
  build_meta: BuildMeta,
}

impl ProvideSharedModule {
  #[allow(clippy::too_many_arguments)]
  pub fn new(
    share_scope: String,
    name: String,
    version: ProvideVersion,
    request: String,
    eager: bool,
    singleton: Option<bool>,
    required_version: Option<ConsumeVersion>,
    strict_version: Option<bool>,
  ) -> Self {
    let identifier = format!(
      "provide shared module ({}) {}@{} = {}",
      &share_scope, &name, &version, &request
    );
    Self {
      blocks: Vec::new(),
      dependencies: Vec::new(),
      identifier: ModuleIdentifier::from(identifier.as_ref()),
      lib_ident: format!("webpack/sharing/provide/{}/{}", &share_scope, &name),
      readable_identifier: identifier,
      name,
      share_scope,
      version,
      request,
      eager,
      singleton,
      required_version,
      strict_version,
      factory_meta: None,
      build_info: BuildInfo {
        strict: true,
        ..Default::default()
      },
      build_meta: Default::default(),
      source_map_kind: SourceMapKind::empty(),
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

  fn remove_dependency_id(&mut self, dependency: DependencyId) {
    self.dependencies.retain(|d| d != &dependency)
  }

  fn get_dependencies(&self) -> &[DependencyId] {
    &self.dependencies
  }
}

#[cacheable_dyn]
#[async_trait]
impl Module for ProvideSharedModule {
  impl_module_meta_info!();

  fn size(&self, _source_type: Option<&SourceType>, _compilation: Option<&Compilation>) -> f64 {
    42.0
  }

  fn module_type(&self) -> &ModuleType {
    &ModuleType::ProvideShared
  }

  fn source_types(&self, _module_graph: &ModuleGraph) -> &[SourceType] {
    &[SourceType::ShareInit]
  }

  fn source(&self) -> Option<&BoxSource> {
    None
  }

  fn readable_identifier(&self, _context: &Context) -> Cow<'_, str> {
    self.readable_identifier.as_str().into()
  }

  fn lib_ident(&self, _options: LibIdentOptions) -> Option<Cow<'_, str>> {
    Some(self.lib_ident.as_str().into())
  }

  async fn build(
    &mut self,
    _build_context: BuildContext,
    _: Option<&Compilation>,
  ) -> Result<BuildResult> {
    let mut blocks = vec![];
    let mut dependencies = vec![];
    let dep = Box::new(ProvideForSharedDependency::new(self.request.clone()));
    if self.eager {
      dependencies.push(dep as BoxDependency);
    } else {
      let block = AsyncDependenciesBlock::new(self.identifier, None, None, vec![dep], None);
      blocks.push(Box::new(block));
    }

    Ok(BuildResult {
      dependencies,
      blocks,
      ..Default::default()
    })
  }

  // #[tracing::instrument("ProvideSharedModule::code_generation", skip_all, fields(identifier = ?self.identifier()))]
  async fn code_generation(
    &self,
    compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
    _: Option<ConcatenationScope>,
  ) -> Result<CodeGenerationResult> {
    let mut runtime_template = compilation
      .runtime_template
      .create_module_codegen_runtime_template();
    let mut code_generation_result = CodeGenerationResult::default();
    code_generation_result
      .runtime_requirements
      .insert(RuntimeGlobals::INITIALIZE_SHARING);
    let factory = if self.eager {
      runtime_template.sync_module_factory(&self.get_dependencies()[0], &self.request, compilation)
    } else {
      runtime_template.async_module_factory(&self.get_blocks()[0], &self.request, compilation)
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
            singleton: self.singleton,
            strict_version: self.strict_version,
            required_version: self.required_version.clone(),
          }),
        }],
      });
    code_generation_result
      .runtime_requirements
      .insert(*runtime_template.runtime_requirements());
    Ok(code_generation_result)
  }

  async fn get_runtime_hash(
    &self,
    compilation: &Compilation,
    runtime: Option<&RuntimeSpec>,
  ) -> Result<RspackHashDigest> {
    let mut hasher = RspackHash::from(&compilation.options.output);
    module_update_hash(self, &mut hasher, compilation, runtime);
    Ok(hasher.digest(&compilation.options.output.hash_digest))
  }
}

impl_empty_diagnosable_trait!(ProvideSharedModule);
