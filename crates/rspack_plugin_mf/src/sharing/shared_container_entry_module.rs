use std::borrow::Cow;

use async_trait::async_trait;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_collections::{Identifiable, Identifier};
use rspack_core::{
  AsyncDependenciesBlock, AsyncDependenciesBlockIdentifier, BoxDependency, BuildContext, BuildInfo,
  BuildMeta, BuildMetaExportsType, BuildResult, CodeGenerationResult, Compilation,
  ConcatenationScope, Context, DependenciesBlock, DependencyId, FactoryMeta, LibIdentOptions,
  Module, ModuleDependency, ModuleGraph, ModuleIdentifier, ModuleType, RuntimeGlobals, RuntimeSpec,
  SourceType, StaticExportsDependency, StaticExportsSpec, impl_module_meta_info,
  module_update_hash,
  rspack_sources::{BoxSource, RawStringSource, SourceExt},
};
use rspack_error::{Result, impl_empty_diagnosable_trait};
use rspack_hash::{RspackHash, RspackHashDigest};
use rspack_util::source_map::{ModuleSourceMapConfig, SourceMapKind};
use rustc_hash::FxHashSet;

use super::shared_container_dependency::SharedContainerDependency;

#[cacheable]
#[derive(Debug)]
pub struct SharedContainerEntryModule {
  blocks: Vec<AsyncDependenciesBlockIdentifier>,
  dependencies: Vec<DependencyId>,
  identifier: ModuleIdentifier,
  lib_ident: String,
  name: String,
  request: String,
  version: String,
  factory_meta: Option<FactoryMeta>,
  build_info: BuildInfo,
  build_meta: BuildMeta,
  source_map_kind: SourceMapKind,
}

impl SharedContainerEntryModule {
  pub fn new(name: String, request: String, version: String) -> Self {
    let lib_ident = format!("webpack/share/container/{}", &name);
    Self {
      blocks: Vec::new(),
      dependencies: Vec::new(),
      identifier: ModuleIdentifier::from(format!("share container entry {}@{}", &name, &version,)),
      lib_ident,
      name,
      request,
      version,
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

impl Identifiable for SharedContainerEntryModule {
  fn identifier(&self) -> Identifier {
    self.identifier
  }
}

impl DependenciesBlock for SharedContainerEntryModule {
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
impl Module for SharedContainerEntryModule {
  impl_module_meta_info!();

  fn size(&self, _source_type: Option<&SourceType>, _compilation: Option<&Compilation>) -> f64 {
    42.0
  }

  fn module_type(&self) -> &ModuleType {
    &ModuleType::ShareContainerShared
  }

  fn source_types(&self, _module_graph: &ModuleGraph) -> &[SourceType] {
    &[SourceType::JavaScript, SourceType::Expose]
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
    let dependencies: Vec<BoxDependency> = vec![
      Box::new(StaticExportsDependency::new(
        StaticExportsSpec::Array(vec!["get".into(), "init".into()]),
        false,
      )),
      Box::new(SharedContainerDependency::new(self.name.clone())),
    ];

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
    code_generation_result
      .runtime_requirements
      .insert(RuntimeGlobals::REQUIRE);

    let module_graph = compilation.get_module_graph();
    let mut factory = String::new();
    for dependency_id in self.get_dependencies() {
      let dependency = module_graph
        .dependency_by_id(dependency_id)
        .expect("share container dependency should exist");
      if let Some(dependency) = dependency.downcast_ref::<SharedContainerDependency>() {
        let module_expr = compilation.runtime_template.module_raw(
          compilation,
          &mut code_generation_result.runtime_requirements,
          dependency_id,
          dependency.user_request(),
          false,
        );
        factory = compilation
          .runtime_template
          .returning_function(&module_expr, "");
      }
    }

    let federation_global = format!(
      "{}.federation",
      compilation
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::REQUIRE)
    );

    // Generate installInitialConsumes function using returning_function
    let install_initial_consumes_call = r#"localBundlerRuntime.installInitialConsumes({ 
        installedModules: localInstalledModules, 
        initialConsumes: __webpack_require__.consumesLoadingData.initialConsumes, 
        moduleToHandlerMapping: __webpack_require__.federation.consumesLoadingModuleToHandlerMapping || {}, 
        webpackRequire: __webpack_require__, 
        asyncLoad: true 
      })"#;
    let install_initial_consumes_fn = compilation
      .runtime_template
      .returning_function(install_initial_consumes_call, "");

    // Create initShareContainer function using basic_function, supporting multi-statement body
    let init_body = format!(
      r#"
    var installedModules = {{}};
    {federation_global}.instance = mfInstance;
    {federation_global}.bundlerRuntime = bundlerRuntime;
    
    // Save parameters to local variables to avoid closure issues
    var localBundlerRuntime = bundlerRuntime;
    var localInstalledModules = installedModules;
    
    if(!__webpack_require__.consumesLoadingData){{return; }}
    {federation_global}.installInitialConsumes = {install_initial_consumes_fn};
    
    return {federation_global}.installInitialConsumes();
  "#,
      federation_global = federation_global,
      install_initial_consumes_fn = install_initial_consumes_fn
    );
    let init_share_container_fn = compilation
      .runtime_template
      .basic_function("mfInstance, bundlerRuntime", &init_body);

    // Generate the final source string
    let source = format!(
      r#"
      __webpack_require__.federation = {{ instance: undefined,bundlerRuntime: undefined }}
      var factory = ()=>{factory};
      var initShareContainer = {init_share_container_fn};
{runtime}(exports, {{ 
	get: function() {{ return factory;}},
	init: function() {{ return initShareContainer;}}
}});
"#,
      runtime = compilation
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::DEFINE_PROPERTY_GETTERS),
      factory = factory,
      init_share_container_fn = init_share_container_fn
    );

    // Update the code generation result with the generated source
    code_generation_result =
      code_generation_result.with_javascript(RawStringSource::from(source).boxed());
    code_generation_result.add(SourceType::Expose, RawStringSource::from_static("").boxed());
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

impl_empty_diagnosable_trait!(SharedContainerEntryModule);

impl ModuleSourceMapConfig for SharedContainerEntryModule {
  fn get_source_map_kind(&self) -> &SourceMapKind {
    &self.source_map_kind
  }

  fn set_source_map_kind(&mut self, source_map: SourceMapKind) {
    self.source_map_kind = source_map;
  }
}
