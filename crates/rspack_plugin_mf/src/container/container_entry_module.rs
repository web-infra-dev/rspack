use std::borrow::Cow;

use async_trait::async_trait;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_collections::{Identifiable, Identifier};
use rspack_core::{
  AsyncDependenciesBlock, AsyncDependenciesBlockIdentifier, BoxDependency, BuildContext, BuildInfo,
  BuildMeta, BuildMetaExportsType, BuildResult, ChunkGroupOptions, CodeGenerationResult,
  Compilation, ConcatenationScope, Context, DependenciesBlock, Dependency, DependencyId,
  ExportsArgument, FactoryMeta, GroupOptions, LibIdentOptions, Module,
  ModuleCodegenRuntimeTemplate, ModuleDependency, ModuleGraph, ModuleIdentifier, ModuleType,
  RuntimeGlobals, RuntimeSpec, SourceType, StaticExportsDependency, StaticExportsSpec,
  impl_module_meta_info, impl_source_map_config, module_update_hash,
  rspack_sources::{BoxSource, RawStringSource, SourceExt},
};
use rspack_error::{Result, impl_empty_diagnosable_trait};
use rspack_hash::{RspackHash, RspackHashDigest};
use rspack_util::source_map::SourceMapKind;
use rustc_hash::FxHashSet;

use super::{
  container_exposed_dependency::ContainerExposedDependency, container_plugin::ExposeOptions,
};
use crate::utils::json_stringify;

#[impl_source_map_config]
#[cacheable]
#[derive(Debug)]
pub struct ContainerEntryModule {
  blocks: Vec<AsyncDependenciesBlockIdentifier>,
  dependencies: Vec<DependencyId>,
  identifier: ModuleIdentifier,
  lib_ident: String,
  exposes: Vec<(String, ExposeOptions)>,
  share_scope: String,
  factory_meta: Option<FactoryMeta>,
  build_info: BuildInfo,
  build_meta: BuildMeta,
  enhanced: bool,
}

impl ContainerEntryModule {
  pub fn new(
    name: String,
    exposes: Vec<(String, ExposeOptions)>,
    share_scope: String,
    enhanced: bool,
  ) -> Self {
    let lib_ident = format!("webpack/container/entry/{}", &name);
    Self {
      blocks: Vec::new(),
      dependencies: Vec::new(),
      identifier: ModuleIdentifier::from(format!(
        "container entry ({}) {}",
        share_scope,
        json_stringify(&exposes),
      )),
      lib_ident,
      exposes,
      share_scope,
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
      enhanced,
    }
  }

  pub fn exposes(&self) -> &[(String, ExposeOptions)] {
    &self.exposes
  }
}

impl Identifiable for ContainerEntryModule {
  fn identifier(&self) -> Identifier {
    self.identifier
  }
}

impl DependenciesBlock for ContainerEntryModule {
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
impl Module for ContainerEntryModule {
  impl_module_meta_info!();

  fn size(&self, _source_type: Option<&SourceType>, _compilation: Option<&Compilation>) -> f64 {
    42.0
  }

  fn module_type(&self) -> &ModuleType {
    &ModuleType::JsDynamic
  }

  fn source_types(&self, _module_graph: &ModuleGraph) -> &[SourceType] {
    &[SourceType::JavaScript, SourceType::Expose]
  }

  fn source(&self) -> Option<&BoxSource> {
    None
  }

  fn readable_identifier(&self, _context: &Context) -> Cow<'_, str> {
    "container entry".into()
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
    let mut dependencies: Vec<BoxDependency> = vec![];
    for (name, options) in &self.exposes {
      let mut block = AsyncDependenciesBlock::new(
        self.identifier,
        None,
        Some(name),
        options
          .import
          .iter()
          .map(|request| {
            Box::new(ContainerExposedDependency::new(
              name.clone(),
              request.clone(),
            )) as Box<dyn Dependency>
          })
          .collect(),
        None,
      );
      block.set_group_options(GroupOptions::ChunkGroup(
        ChunkGroupOptions::default().name_optional(options.name.clone()),
      ));
      blocks.push(Box::new(block));
    }
    dependencies.push(Box::new(StaticExportsDependency::new(
      StaticExportsSpec::Array(vec!["get".into(), "init".into()]),
      false,
    )));

    Ok(BuildResult {
      dependencies,
      blocks,
      ..Default::default()
    })
  }

  // #[tracing::instrument("ContainerEntryModule::code_generation", skip_all, fields(identifier = ?self.identifier()))]
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
      .insert(RuntimeGlobals::CURRENT_REMOTE_GET_SCOPE);
    let module_map = ExposeModuleMap::new(compilation, self, &mut runtime_template);
    let module_map_str = module_map.render(&mut runtime_template);
    let source = if self.enhanced {
      let define_property_getters =
        runtime_template.render_runtime_globals(&RuntimeGlobals::DEFINE_PROPERTY_GETTERS);
      let get_container = format!(
        "{}.getContainer",
        runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE)
      );
      let init_container = format!(
        "{}.initContainer",
        runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE)
      );

      format!(
        r#"
{}({}, {{
	get: {},
	init: {}
}});"#,
        define_property_getters,
        runtime_template.render_exports_argument(ExportsArgument::Exports),
        runtime_template.returning_function(&get_container, ""),
        runtime_template.returning_function(&init_container, ""),
      )
    } else {
      format!(
        r#"
var moduleMap = {module_map_str};
var get = function(module, getScope) {{
  {current_remote_get_scope} = getScope;
  getScope = (
    {has_own_property}(moduleMap, module)
      ? moduleMap[module]()
      : Promise.resolve().then({get_scope_reject})
  );
  {current_remote_get_scope} = undefined;
  return getScope;
}}
var init = function(shareScope, initScope) {{
  if (!{share_scope_map}) return;
  var name = {share_scope};
  var oldScope = {share_scope_map}[name];
  if(oldScope && oldScope !== shareScope) throw new Error("Container initialization failed as it has already been initialized with a different share scope");
  {share_scope_map}[name] = shareScope;
  return {initialize_sharing}(name, initScope);
}}
{define_property_getters}({exports}, {{
	get: {export_get},
	init: {export_init}
}});"#,
        exports = runtime_template.render_exports_argument(ExportsArgument::Exports),
        current_remote_get_scope =
          runtime_template.render_runtime_globals(&RuntimeGlobals::CURRENT_REMOTE_GET_SCOPE),
        has_own_property =
          runtime_template.render_runtime_globals(&RuntimeGlobals::HAS_OWN_PROPERTY),
        share_scope_map = runtime_template.render_runtime_globals(&RuntimeGlobals::SHARE_SCOPE_MAP),
        share_scope = json_stringify(&self.share_scope),
        initialize_sharing =
          runtime_template.render_runtime_globals(&RuntimeGlobals::INITIALIZE_SHARING),
        define_property_getters =
          runtime_template.render_runtime_globals(&RuntimeGlobals::DEFINE_PROPERTY_GETTERS),
        get_scope_reject = runtime_template.basic_function(
          "",
          r#"throw new Error('Module "' + module + '" does not exist in container.');"#
        ),
        export_get = runtime_template.returning_function("get", ""),
        export_init = runtime_template.returning_function("init", ""),
      )
    };
    code_generation_result =
      code_generation_result.with_javascript(RawStringSource::from(source).boxed());
    code_generation_result.add(SourceType::Expose, RawStringSource::from_static("").boxed());
    if self.enhanced {
      code_generation_result
        .data
        .insert(CodeGenerationDataExpose {
          module_map,
          share_scope: self.share_scope.clone(),
        });
    }
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

impl_empty_diagnosable_trait!(ContainerEntryModule);

#[derive(Debug, Clone)]
pub struct ExposeModuleMap(Vec<(String, String)>);

impl ExposeModuleMap {
  pub fn new(
    compilation: &Compilation,
    container_entry_module: &ContainerEntryModule,
    runtime_template: &mut ModuleCodegenRuntimeTemplate,
  ) -> Self {
    let mut module_map = vec![];
    let module_graph = compilation.get_module_graph();
    for block_id in container_entry_module.get_blocks() {
      let block = module_graph
        .block_by_id(block_id)
        .expect("should have block");
      let modules_iter = block.get_dependencies().iter().map(|dependency_id| {
        let dep = module_graph.dependency_by_id(dependency_id);
        let dep = dep
          .downcast_ref::<ContainerExposedDependency>()
          .expect("dependencies of ContainerEntryModule should be ContainerExposedDependency");
        let name = dep.exposed_name.as_str();
        let module = module_graph.get_module_by_dependency_id(dependency_id);
        let user_request = dep.user_request();
        (name, module, user_request, dependency_id)
      });
      let name = modules_iter.clone().next().expect("should have item").0;
      let str = if modules_iter
        .clone()
        .any(|(_, module, _, _)| module.is_none())
      {
        runtime_template.throw_missing_module_error_block(
          &modules_iter
            .map(|(_, _, request, _)| request)
            .collect::<Vec<&str>>()
            .join(", "),
        )
      } else {
        let block_promise = runtime_template.block_promise(Some(block_id), compilation, "");
        let modules = modules_iter
          .map(|(_, _, request, dependency_id)| {
            runtime_template.module_raw(compilation, dependency_id, request, false)
          })
          .collect::<Vec<_>>()
          .join(", ");
        let module_raw = runtime_template
          .returning_function(&runtime_template.returning_function(&modules, ""), "");
        format!("return {block_promise}.then({module_raw});")
      };
      module_map.push((name.to_string(), str));
    }
    Self(module_map)
  }

  pub fn render(&self, runtime_template: &mut ModuleCodegenRuntimeTemplate) -> String {
    let module_map = self
      .0
      .iter()
      .map(|(name, factory)| {
        format!(
          "{}: {},",
          json_stringify(name),
          runtime_template.basic_function("", factory)
        )
      })
      .collect::<Vec<_>>()
      .join("\n");
    format!(
      r#"{{
  {module_map}
}}"#
    )
  }
}

#[derive(Debug, Clone)]
pub struct CodeGenerationDataExpose {
  pub module_map: ExposeModuleMap,
  pub share_scope: String,
}
