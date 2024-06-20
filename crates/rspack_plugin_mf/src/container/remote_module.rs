use std::borrow::Cow;
use std::hash::Hash;

use async_trait::async_trait;
use rspack_core::{
  impl_module_meta_info, impl_source_map_config,
  rspack_sources::{RawSource, Source, SourceExt},
  AsyncDependenciesBlockIdentifier, BoxDependency, BuildContext, BuildInfo, BuildMeta, BuildResult,
  CodeGenerationResult, Compilation, ConcatenationScope, Context, DependenciesBlock, DependencyId,
  FactoryMeta, LibIdentOptions, Module, ModuleIdentifier, ModuleType, RuntimeSpec, SourceType,
};
use rspack_error::{impl_empty_diagnosable_trait, Diagnostic, Result};
use rspack_hash::RspackHash;
use rspack_identifier::{Identifiable, Identifier};
use rspack_util::source_map::SourceMapKind;

use super::{
  fallback_dependency::FallbackDependency,
  remote_to_external_dependency::RemoteToExternalDependency,
};
use crate::{
  sharing::share_runtime_module::DataInitInfo, CodeGenerationDataShareInit, ShareInitData,
};

#[impl_source_map_config]
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
  build_info: Option<BuildInfo>,
  build_meta: Option<BuildMeta>,
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
      build_info: None,
      build_meta: None,
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

  fn get_dependencies(&self) -> &[DependencyId] {
    &self.dependencies
  }
}

#[async_trait]
impl Module for RemoteModule {
  impl_module_meta_info!();

  fn size(&self, _source_type: Option<&SourceType>, _compilation: &Compilation) -> f64 {
    6.0
  }

  fn module_type(&self) -> &ModuleType {
    &ModuleType::Remote
  }

  fn get_diagnostics(&self) -> Vec<Diagnostic> {
    vec![]
  }

  fn source_types(&self) -> &[SourceType] {
    &[SourceType::Remote, SourceType::ShareInit]
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

  fn name_for_condition(&self) -> Option<Box<str>> {
    Some(self.request.as_str().into())
  }

  async fn build(
    &mut self,
    build_context: BuildContext<'_>,
    _: Option<&Compilation>,
  ) -> Result<BuildResult> {
    let mut hasher = RspackHash::from(&build_context.compiler_options.output);
    self.update_hash(&mut hasher);

    let build_info = BuildInfo {
      strict: true,
      hash: Some(hasher.digest(&build_context.compiler_options.output.hash_digest)),
      ..Default::default()
    };

    let mut dependencies: Vec<BoxDependency> = Vec::new();
    if self.external_requests.len() == 1 {
      let dep = RemoteToExternalDependency::new(self.external_requests[0].clone());
      dependencies.push(Box::new(dep));
    } else {
      let dep = FallbackDependency::new(self.external_requests.clone());
      dependencies.push(Box::new(dep));
    }

    Ok(BuildResult {
      build_info,
      build_meta: Default::default(),
      dependencies,
      blocks: Vec::new(),
      analyze_result: Default::default(),
      optimization_bailouts: vec![],
    })
  }

  #[allow(clippy::unwrap_in_result)]
  fn code_generation(
    &self,
    compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
    _: Option<ConcatenationScope>,
  ) -> Result<CodeGenerationResult> {
    let mut codegen = CodeGenerationResult::default();
    let module_graph = compilation.get_module_graph();
    let module = module_graph.get_module_by_dependency_id(&self.dependencies[0]);
    let id = module.and_then(|m| {
      compilation
        .chunk_graph
        .get_module_id(m.identifier())
        .as_deref()
    });
    codegen.add(SourceType::Remote, RawSource::from("").boxed());
    codegen.data.insert(CodeGenerationDataShareInit {
      items: vec![ShareInitData {
        share_scope: self.share_scope.clone(),
        init_stage: 20,
        init: DataInitInfo::ExternalModuleId(id.map(|i| i.to_owned())),
      }],
    });
    Ok(codegen)
  }
}

impl_empty_diagnosable_trait!(RemoteModule);

impl Hash for RemoteModule {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    "__rspack_internal__RemoteModule".hash(state);
    self.identifier().hash(state);
  }
}

impl PartialEq for RemoteModule {
  fn eq(&self, other: &Self) -> bool {
    self.identifier() == other.identifier()
  }
}

impl Eq for RemoteModule {}
