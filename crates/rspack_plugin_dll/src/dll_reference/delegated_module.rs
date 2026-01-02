use std::{borrow::Cow, hash::Hash, sync::Arc};

use async_trait::async_trait;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_collections::{Identifiable, Identifier};
use rspack_core::{
  AsyncDependenciesBlockIdentifier, BoxDependency, BuildContext, BuildInfo, BuildMeta, BuildResult,
  CodeGenerationResult, Compilation, ConcatenationScope, Context, DependenciesBlock, DependencyId,
  FactoryMeta, LibIdentOptions, Module, ModuleDependency, ModuleGraph, ModuleId, ModuleType,
  RuntimeGlobals, RuntimeSpec, SourceType, StaticExportsDependency, StaticExportsSpec,
  ValueCacheVersions, impl_module_meta_info, impl_source_map_config, module_update_hash,
  rspack_sources::{BoxSource, OriginalSource, RawStringSource},
};
use rspack_error::{Result, impl_empty_diagnosable_trait};
use rspack_hash::{RspackHash, RspackHashDigest};
use rspack_util::{json_stringify, source_map::ModuleSourceMapConfig};

use super::delegated_source_dependency::DelegatedSourceDependency;
use crate::{DllManifestContentItem, DllManifestContentItemExports};

pub type SourceRequest = String;

#[impl_source_map_config]
#[cacheable]
#[derive(Debug, Default)]
pub struct DelegatedModule {
  source_request: SourceRequest,
  request: Option<ModuleId>,
  delegation_type: String,
  user_request: String,
  original_request: Option<String>,
  delegate_data: DllManifestContentItem,
  dependencies: Vec<DependencyId>,
  blocks: Vec<AsyncDependenciesBlockIdentifier>,
  factory_meta: Option<FactoryMeta>,
  build_info: BuildInfo,
  build_meta: BuildMeta,
}

impl DelegatedModule {
  pub fn new(
    source_request: SourceRequest,
    data: DllManifestContentItem,
    delegation_type: String,
    user_request: String,
    original_request: Option<String>,
  ) -> Self {
    Self {
      source_request,
      request: data.id.clone(),
      delegation_type,
      user_request,
      original_request,
      delegate_data: data,
      ..Default::default()
    }
  }
}

#[cacheable_dyn]
#[async_trait]
impl Module for DelegatedModule {
  impl_module_meta_info!();

  fn module_type(&self) -> &ModuleType {
    &ModuleType::JsDynamic
  }

  fn source_types(&self, _module_graph: &ModuleGraph) -> &[SourceType] {
    &[SourceType::JavaScript]
  }

  fn lib_ident(&self, _options: LibIdentOptions) -> Option<Cow<'_, str>> {
    self.original_request.as_ref().map(|request| request.into())
  }

  fn source(&self) -> Option<&BoxSource> {
    None
  }

  fn readable_identifier(&self, _context: &Context) -> Cow<'_, str> {
    format!(
      "delegated {} from {}",
      self.user_request, self.source_request
    )
    .into()
  }

  fn size(&self, _source_type: Option<&SourceType>, _compilation: Option<&Compilation>) -> f64 {
    42.0
  }

  async fn build(
    &mut self,
    _build_context: BuildContext,
    _compilation: Option<&Compilation>,
  ) -> Result<BuildResult> {
    let dependencies = vec![
      Box::new(DelegatedSourceDependency::new(self.source_request.clone())),
      Box::new(StaticExportsDependency::new(
        match self.delegate_data.exports.clone() {
          Some(exports) => match exports {
            DllManifestContentItemExports::True => StaticExportsSpec::True,
            DllManifestContentItemExports::Vec(vec) => StaticExportsSpec::Array(vec),
          },
          None => StaticExportsSpec::True,
        },
        false,
      )) as BoxDependency,
    ];
    self.build_meta = self.delegate_data.build_meta.clone();
    Ok(BuildResult {
      dependencies,
      ..Default::default()
    })
  }

  async fn code_generation(
    &self,
    compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
    _concatenation_scope: Option<ConcatenationScope>,
  ) -> Result<CodeGenerationResult> {
    let mut runtime_requirements = RuntimeGlobals::default();
    runtime_requirements.insert(RuntimeGlobals::REQUIRE);
    runtime_requirements.insert(RuntimeGlobals::MODULE);
    let mut code_generation_result = CodeGenerationResult {
      runtime_requirements,
      ..Default::default()
    };

    let dep = self.dependencies[0];
    let mg = compilation.get_module_graph();
    let source_module = mg.get_module_by_dependency_id(&dep);
    let dependency = mg
      .try_dependency_by_id(&dep)
      .and_then(|dep| dep.downcast_ref::<DelegatedSourceDependency>())
      .expect("Should be module dependency");

    let str = match source_module {
      Some(_) => {
        let mut s = format!(
          "module.exports = ({})",
          compilation.runtime_template.module_raw(
            compilation,
            &mut code_generation_result.runtime_requirements,
            &dep,
            dependency.request(),
            false,
          )
        );

        let request = json_stringify(
          self
            .request
            .as_ref()
            .expect("manifest content should have `id`."),
        );

        match self.delegation_type.as_ref() {
          "require" => {
            s += &format!("({request})");
          }
          "object" => {
            s += &format!("[{request}]");
          }
          _ => panic!("delegation_type should be 'require' or 'object'"),
        }

        s += ";";

        s
      }
      None => compilation
        .runtime_template
        .throw_missing_module_error_block(&self.source_request),
    };

    let source_map = self.get_source_map_kind();

    let source: BoxSource = if source_map.source_map() || source_map.simple_source_map() {
      Arc::new(OriginalSource::new(str, self.identifier().to_string()))
    } else {
      let raw_source: RawStringSource = str.into();
      Arc::new(raw_source)
    };

    code_generation_result = code_generation_result.with_javascript(source);

    Ok(code_generation_result)
  }

  fn need_build(&self, _value_cache_versions: &ValueCacheVersions) -> bool {
    false
  }

  async fn get_runtime_hash(
    &self,
    compilation: &Compilation,
    runtime: Option<&RuntimeSpec>,
  ) -> Result<RspackHashDigest> {
    let mut hasher = RspackHash::from(&compilation.options.output);
    self.delegation_type.hash(&mut hasher);

    if let Some(request) = &self.request {
      request.hash(&mut hasher);
    }

    module_update_hash(self, &mut hasher, compilation, runtime);

    Ok(hasher.digest(&compilation.options.output.hash_digest))
  }
}

impl Identifiable for DelegatedModule {
  fn identifier(&self) -> Identifier {
    format!(
      "delegated {} from {}",
      self
        .request
        .as_ref()
        .map(|r| r.to_string())
        .unwrap_or_default(),
      self.source_request
    )
    .into()
  }
}

impl DependenciesBlock for DelegatedModule {
  fn add_block_id(&mut self, block: AsyncDependenciesBlockIdentifier) {
    self.blocks.push(block);
  }

  fn get_blocks(&self) -> &[AsyncDependenciesBlockIdentifier] {
    &self.blocks
  }

  fn add_dependency_id(&mut self, dependency: DependencyId) {
    self.dependencies.push(dependency);
  }

  fn get_dependencies(&self) -> &[DependencyId] {
    &self.dependencies
  }

  fn remove_dependency_id(&mut self, dependency: DependencyId) {
    self.dependencies.retain(|d| d != &dependency)
  }
}

impl_empty_diagnosable_trait!(DelegatedModule);
