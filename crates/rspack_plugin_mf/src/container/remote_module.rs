use std::borrow::Cow;

use async_trait::async_trait;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_collections::{Identifiable, Identifier};
use rspack_core::{
  AsyncDependenciesBlockIdentifier, BoxDependency, BuildContext, BuildInfo, BuildMeta, BuildResult,
  ChunkGraph, CodeGenerationResult, Compilation, ConcatenationScope, Context, DependenciesBlock,
  Dependency, DependencyId, ExportsType, FactoryMeta, LibIdentOptions, Module, ModuleGraph,
  ModuleIdentifier, ModuleType, RuntimeSpec, SourceType, impl_module_meta_info,
  impl_source_map_config, module_update_hash,
  rspack_sources::{BoxSource, RawStringSource, SourceExt},
};
use rspack_error::{Result, impl_empty_diagnosable_trait};
use rspack_hash::{RspackHash, RspackHashDigest};
use rspack_util::source_map::SourceMapKind;

use super::{
  fallback_dependency::FallbackDependency, federation_modules_plugin::FederationModulesPlugin,
  remote_to_external_dependency::RemoteToExternalDependency,
};
use crate::{
  CodeGenerationDataShareInit, ShareInitData, sharing::share_runtime_module::DataInitInfo,
};

#[impl_source_map_config]
#[cacheable]
#[derive(Debug)]
pub struct RemoteModule {
  blocks: Vec<AsyncDependenciesBlockIdentifier>,
  dependencies: Vec<DependencyId>,
  identifier: ModuleIdentifier,
  readable_identifier: String,
  lib_ident: String,
  request: String,
  external_requests: Vec<String>,
  pub internal_request: String,
  pub share_scope: String,
  pub remote_key: String,
  factory_meta: Option<FactoryMeta>,
  build_info: BuildInfo,
  build_meta: BuildMeta,
}

impl RemoteModule {
  pub fn new(
    request: String,
    external_requests: Vec<String>,
    internal_request: String,
    share_scope: String,
    remote_key: String,
  ) -> Self {
    let readable_identifier = format!("remote {}", &request);
    let lib_ident = format!("webpack/container/remote/{}", &request);
    Self {
      blocks: Default::default(),
      dependencies: Default::default(),
      identifier: ModuleIdentifier::from(format!(
        "remote ({}) {} {}",
        share_scope,
        external_requests.join(" "),
        internal_request
      )),
      readable_identifier,
      lib_ident,
      request,
      external_requests,
      internal_request,
      share_scope,
      remote_key,
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

impl Identifiable for RemoteModule {
  fn identifier(&self) -> Identifier {
    self.identifier
  }
}

impl DependenciesBlock for RemoteModule {
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
impl Module for RemoteModule {
  impl_module_meta_info!();

  fn size(&self, _source_type: Option<&SourceType>, _compilation: Option<&Compilation>) -> f64 {
    6.0
  }

  fn module_type(&self) -> &ModuleType {
    &ModuleType::Remote
  }

  fn source_types(&self, _module_graph: &ModuleGraph) -> &[SourceType] {
    &[SourceType::Remote, SourceType::ShareInit]
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

  fn name_for_condition(&self) -> Option<Box<str>> {
    Some(self.request.as_str().into())
  }

  fn get_exports_type(
    &self,
    _module_graph: &ModuleGraph,
    _module_graph_cache: &rspack_core::ModuleGraphCacheArtifact,
    _strict: bool,
  ) -> ExportsType {
    ExportsType::Dynamic
  }

  async fn build(
    &mut self,
    build_context: BuildContext,
    _compilation: Option<&Compilation>,
  ) -> Result<BuildResult> {
    let mut dependencies: Vec<BoxDependency> = Vec::new();

    if self.external_requests.len() == 1 {
      let dep = RemoteToExternalDependency::new(self.external_requests[0].clone());

      // Call federation hooks here using the BuildContext - this runs before optimize_chunks!
      let hooks =
        FederationModulesPlugin::get_compilation_hooks_by_id(build_context.compilation_id);
      hooks
        .add_remote_dependency
        .lock()
        .await
        .call(&dep as &dyn Dependency)
        .await?;

      dependencies.push(Box::new(dep));
    } else {
      let dep = FallbackDependency::new(self.external_requests.clone());

      // Call federation hooks here using the BuildContext - this runs before optimize_chunks!
      let hooks =
        FederationModulesPlugin::get_compilation_hooks_by_id(build_context.compilation_id);
      hooks
        .add_remote_dependency
        .lock()
        .await
        .call(&dep as &dyn Dependency)
        .await?;

      dependencies.push(Box::new(dep));
    }

    Ok(BuildResult {
      dependencies,
      blocks: Vec::new(),
      optimization_bailouts: vec![],
    })
  }

  // #[tracing::instrument("RemoteModule::code_generation", skip_all, fields(identifier = ?self.identifier()))]
  async fn code_generation(
    &self,
    compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
    _: Option<ConcatenationScope>,
  ) -> Result<CodeGenerationResult> {
    let mut codegen = CodeGenerationResult::default();
    let module_graph = compilation.get_module_graph();
    let module = module_graph.get_module_by_dependency_id(&self.dependencies[0]);
    let id = module
      .and_then(|m| ChunkGraph::get_module_id(&compilation.module_ids_artifact, m.identifier()));
    codegen.add(SourceType::Remote, RawStringSource::from_static("").boxed());
    codegen.data.insert(CodeGenerationDataShareInit {
      items: vec![ShareInitData {
        share_scope: self.share_scope.clone(),
        init_stage: 20,
        init: DataInitInfo::ExternalModuleId(id.cloned()),
      }],
    });
    Ok(codegen)
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

impl_empty_diagnosable_trait!(RemoteModule);
