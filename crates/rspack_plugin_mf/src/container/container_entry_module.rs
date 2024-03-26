use std::{borrow::Cow, hash::Hash};

use async_trait::async_trait;
use rspack_core::{
  block_promise, impl_build_info_meta, impl_source_map_config, module_raw, returning_function,
  rspack_sources::{RawSource, Source, SourceExt},
  throw_missing_module_error_block, AsyncDependenciesBlock, AsyncDependenciesBlockIdentifier,
  BoxDependency, BuildContext, BuildInfo, BuildMeta, BuildMetaExportsType, BuildResult,
  ChunkGroupOptions, CodeGenerationResult, Compilation, ConcatenationScope, Context,
  DependenciesBlock, Dependency, DependencyId, GroupOptions, LibIdentOptions, Module,
  ModuleDependency, ModuleIdentifier, ModuleType, RuntimeGlobals, RuntimeSpec, SourceType,
  StaticExportsDependency, StaticExportsSpec,
};
use rspack_error::{impl_empty_diagnosable_trait, Diagnostic, Result};
use rspack_hash::RspackHash;
use rspack_identifier::{Identifiable, Identifier};
use rspack_util::source_map::SourceMapKind;

use super::{
  container_exposed_dependency::ContainerExposedDependency, container_plugin::ExposeOptions,
  expose_runtime_module::CodeGenerationDataExpose,
};
use crate::utils::json_stringify;

#[impl_source_map_config]
#[derive(Debug)]
pub struct ContainerEntryModule {
  blocks: Vec<AsyncDependenciesBlockIdentifier>,
  dependencies: Vec<DependencyId>,
  identifier: ModuleIdentifier,
  lib_ident: String,
  exposes: Vec<(String, ExposeOptions)>,
  share_scope: String,
  build_info: Option<BuildInfo>,
  build_meta: Option<BuildMeta>,
}

impl ContainerEntryModule {
  pub fn new(name: String, exposes: Vec<(String, ExposeOptions)>, share_scope: String) -> Self {
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
      build_info: None,
      build_meta: None,
      source_map_kind: SourceMapKind::None,
    }
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

  fn get_dependencies(&self) -> &[DependencyId] {
    &self.dependencies
  }
}

#[async_trait]
impl Module for ContainerEntryModule {
  impl_build_info_meta!();

  fn size(&self, _source_type: &SourceType) -> f64 {
    42.0
  }

  fn module_type(&self) -> &ModuleType {
    &ModuleType::JsDynamic
  }

  fn source_types(&self) -> &[SourceType] {
    &[SourceType::JavaScript, SourceType::Expose]
  }

  fn original_source(&self) -> Option<&dyn Source> {
    None
  }

  fn readable_identifier(&self, _context: &Context) -> Cow<str> {
    "container entry".into()
  }

  fn lib_ident(&self, _options: LibIdentOptions) -> Option<Cow<str>> {
    Some(self.lib_ident.as_str().into())
  }
  fn get_diagnostics(&self) -> Vec<Diagnostic> {
    vec![]
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
      );
      block.set_group_options(GroupOptions::ChunkGroup(
        ChunkGroupOptions::default().name_optional(options.name.clone()),
      ));
      blocks.push(block);
    }
    dependencies.push(Box::new(StaticExportsDependency::new(
      StaticExportsSpec::Array(vec!["get".into(), "init".into()]),
      false,
    )));

    Ok(BuildResult {
      build_info: BuildInfo {
        hash: Some(hash),
        strict: true,
        ..Default::default()
      },
      build_meta: BuildMeta {
        exports_type: BuildMetaExportsType::Namespace,
        ..Default::default()
      },
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
      .insert(RuntimeGlobals::DEFINE_PROPERTY_GETTERS);
    code_generation_result
      .runtime_requirements
      .insert(RuntimeGlobals::EXPORTS);
    code_generation_result
      .runtime_requirements
      .insert(RuntimeGlobals::REQUIRE);
    code_generation_result
      .runtime_requirements
      .insert(RuntimeGlobals::CURRENT_REMOTE_GET_SCOPE);
    let mut module_map = vec![];
    let module_graph = compilation.get_module_graph();
    for block_id in self.get_blocks() {
      let block = module_graph
        .block_by_id(block_id)
        .expect("should have block");
      let modules_iter = block.get_dependencies().iter().map(|dependency_id| {
        let dep = module_graph
          .dependency_by_id(dependency_id)
          .expect("should have dependency");
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
        throw_missing_module_error_block(
          &modules_iter
            .map(|(_, _, request, _)| request)
            .collect::<Vec<&str>>()
            .join(", "),
        )
      } else {
        let block_promise = block_promise(
          Some(block_id),
          &mut code_generation_result.runtime_requirements,
          compilation,
        );
        let module_raw = returning_function(
          &returning_function(
            &modules_iter
              .map(|(_, _, request, dependency_id)| {
                module_raw(
                  compilation,
                  &mut code_generation_result.runtime_requirements,
                  dependency_id,
                  request,
                  false,
                )
              })
              .collect::<Vec<_>>()
              .join(", "),
            "",
          ),
          "",
        );
        format!("return {}.then({});", block_promise, module_raw)
      };
      module_map.push((name.to_string(), str));
    }
    let source = format!(
      r#"
{}(exports, {{
	get: () => (__webpack_require__.getContainer),
	init: () => (__webpack_require__.initContainer)
}});"#,
      RuntimeGlobals::DEFINE_PROPERTY_GETTERS,
    );
    code_generation_result =
      code_generation_result.with_javascript(RawSource::from(source).boxed());
    code_generation_result.add(SourceType::Expose, RawSource::from("").boxed());
    code_generation_result
      .data
      .insert(CodeGenerationDataExpose {
        module_map,
        share_scope: self.share_scope.clone(),
      });
    Ok(code_generation_result)
  }
}

impl_empty_diagnosable_trait!(ContainerEntryModule);

impl Hash for ContainerEntryModule {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    "__rspack_internal__ContainerEntryModule".hash(state);
    self.identifier().hash(state);
  }
}

impl PartialEq for ContainerEntryModule {
  fn eq(&self, other: &Self) -> bool {
    self.identifier() == other.identifier()
  }
}

impl Eq for ContainerEntryModule {}
