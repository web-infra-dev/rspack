use std::borrow::Cow;

use async_trait::async_trait;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_collections::{Identifiable, Identifier};
use rspack_core::{
  AsyncDependenciesBlock, AsyncDependenciesBlockIdentifier, BoxDependency, BuildContext, BuildInfo,
  BuildMeta, BuildMetaExportsType, BuildResult, CodeGenerationResult, Compilation,
  ConcatenationScope, Context, DependenciesBlock, DependencyId, FactoryMeta, LibIdentOptions,
  Module, ModuleDependency, ModuleGraph, ModuleIdentifier, ModuleType, RuntimeGlobals, RuntimeSpec,
  SourceType, StaticExportsDependency, StaticExportsSpec, impl_module_meta_info, module_raw,
  module_update_hash, returning_function,
  rspack_sources::{BoxSource, RawStringSource, SourceExt},
};
use rspack_error::{Result, impl_empty_diagnosable_trait};
use rspack_hash::{RspackHash, RspackHashDigest};
use rspack_util::source_map::{ModuleSourceMapConfig, SourceMapKind};
use rustc_hash::FxHashSet;

use super::share_container_dependency::ShareContainerDependency;
use crate::utils::json_stringify;

#[cacheable]
#[derive(Debug)]
pub struct ShareContainerEntryModule {
  blocks: Vec<AsyncDependenciesBlockIdentifier>,
  dependencies: Vec<DependencyId>,
  identifier: ModuleIdentifier,
  lib_ident: String,
  name: String,
  share_name: String,
  request: String,
  version: String,
  global_name: String,
  factory_meta: Option<FactoryMeta>,
  build_info: BuildInfo,
  build_meta: BuildMeta,
  source_map_kind: SourceMapKind,
}

impl ShareContainerEntryModule {
  pub fn new(
    name: String,
    share_name: String,
    request: String,
    version: String,
    global_name: String,
  ) -> Self {
    let lib_ident = format!("webpack/share/container/{}", &name);
    Self {
      blocks: Vec::new(),
      dependencies: Vec::new(),
      identifier: ModuleIdentifier::from(format!(
        "share container entry {}@{}",
        &share_name, &version,
      )),
      lib_ident,
      name,
      share_name,
      request,
      version,
      global_name,
      factory_meta: None,
      build_info: BuildInfo {
        strict: true,
        top_level_declarations: Some(FxHashSet::default()),
        ..Default::default()
      },
      build_meta: BuildMeta {
        exports_type: BuildMetaExportsType::Namespace,
        ..Default::default()
      },
      source_map_kind: SourceMapKind::empty(),
    }
  }
}

impl Identifiable for ShareContainerEntryModule {
  fn identifier(&self) -> Identifier {
    self.identifier
  }
}

impl DependenciesBlock for ShareContainerEntryModule {
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
impl Module for ShareContainerEntryModule {
  impl_module_meta_info!();

  fn size(&self, _source_type: Option<&SourceType>, _compilation: Option<&Compilation>) -> f64 {
    42.0
  }

  fn module_type(&self) -> &ModuleType {
    &ModuleType::JsDynamic
  }

  fn source_types(&self, _module_graph: &ModuleGraph) -> &[SourceType] {
    &[SourceType::JavaScript]
  }

  fn source(&self) -> Option<&BoxSource> {
    None
  }

  fn readable_identifier(&self, _context: &Context) -> Cow<'_, str> {
    "share container entry".into()
  }

  fn lib_ident(&self, _options: LibIdentOptions) -> Option<Cow<'_, str>> {
    Some(self.lib_ident.as_str().into())
  }

  async fn build(
    &mut self,
    _build_context: BuildContext,
    _: Option<&Compilation>,
  ) -> Result<BuildResult> {
    let mut dependencies: Vec<BoxDependency> = Vec::new();
    dependencies.push(Box::new(ShareContainerDependency::new(
      self.share_name.clone(),
      self.request.clone(),
    )));

    dependencies.push(Box::new(StaticExportsDependency::new(
      StaticExportsSpec::Array(vec!["get".into(), "init".into()]),
      false,
    )));

    Ok(BuildResult {
      dependencies,
      blocks: Vec::<Box<AsyncDependenciesBlock>>::new(),
      ..Default::default()
    })
  }

  async fn code_generation(
    &self,
    compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
    _: Option<ConcatenationScope>,
  ) -> Result<CodeGenerationResult> {
    let mut code_generation_result = CodeGenerationResult::default();
    code_generation_result
      .runtime_requirements
      .insert(RuntimeGlobals::DEFINE_PROPERTY_GETTERS);
    code_generation_result
      .runtime_requirements
      .insert(RuntimeGlobals::EXPORTS);

    let module_graph = compilation.get_module_graph();
    let mut module_map_entries = Vec::new();
    for dependency_id in self.get_dependencies() {
      let dependency = module_graph
        .dependency_by_id(dependency_id)
        .expect("share container dependency should exist");
      if let Some(dependency) = dependency.downcast_ref::<ShareContainerDependency>() {
        let module_expr = module_raw(
          compilation,
          &mut code_generation_result.runtime_requirements,
          dependency_id,
          dependency.user_request(),
          false,
        );
        let factory = returning_function(&compilation.options.output.environment, &module_expr, "");
        let loader = format!(
          "function() {{ return Promise.resolve().then({}); }}",
          returning_function(&compilation.options.output.environment, &factory, "")
        );
        module_map_entries.push(format!(
          "{}: {{ loader: {}, promise: undefined }}",
          json_stringify(&dependency.share_key),
          loader
        ));
      }
    }

    let module_map = module_map_entries.join(",\n");

    let source = format!(
      "const __container_name__ = {};
const moduleMap = {{
{}
}};

function load(module) {{
  const entry = moduleMap[module];
  if (!entry) {{
    return Promise.reject(new Error(\"Shared module \" + module + \" is not available in container \" + __container_name__ + \".\"));
  }}
  if (!entry.promise) {{
    entry.promise = entry.loader();
  }}
  return entry.promise;
}}

export function get(module) {{
  return load(module).then(factory => factory());
}}

export function init(shareScope, initScope) {{
  return Promise.resolve();
}}
",
      json_stringify(&self.global_name),
      module_map
    );

    code_generation_result.add(
      SourceType::JavaScript,
      RawStringSource::from(source).boxed(),
    );
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

impl_empty_diagnosable_trait!(ShareContainerEntryModule);

impl ModuleSourceMapConfig for ShareContainerEntryModule {
  fn get_source_map_kind(&self) -> &SourceMapKind {
    &self.source_map_kind
  }

  fn set_source_map_kind(&mut self, source_map: SourceMapKind) {
    self.source_map_kind = source_map;
  }
}
