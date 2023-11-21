use std::{borrow::Cow, hash::Hash};

use async_trait::async_trait;
use rspack_error::{IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};
use rspack_hash::RspackHash;
use rspack_identifier::{Identifiable, Identifier};
use rspack_sources::{RawSource, Source, SourceExt};

use super::{
  container_exposed_dependency::ContainerExposedDependency, container_plugin::ExposeOptions,
};
use crate::{
  basic_function, block_promise, module_raw, returning_function, throw_missing_module_error_block,
  AsyncDependenciesBlock, AsyncDependenciesBlockIdentifier, BuildContext, BuildInfo, BuildMeta,
  BuildMetaExportsType, BuildResult, ChunkGroupOptions, CodeGenerationResult, Compilation, Context,
  DependenciesBlock, DependencyId, GroupOptions, LibIdentOptions, Module, ModuleDependency,
  ModuleIdentifier, ModuleType, RuntimeGlobals, RuntimeSpec, SourceType,
};

#[derive(Debug)]
pub struct ContainerEntryModule {
  blocks: Vec<AsyncDependenciesBlockIdentifier>,
  dependencies: Vec<DependencyId>,
  identifier: ModuleIdentifier,
  lib_ident: String,
  exposes: Vec<(String, ExposeOptions)>,
  share_scope: String,
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
        serde_json::to_string(&exposes).expect("should able to json to_string")
      )),
      lib_ident,
      exposes,
      share_scope,
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
  fn size(&self, _source_type: &SourceType) -> f64 {
    42.0
  }

  fn module_type(&self) -> &ModuleType {
    &ModuleType::JsDynamic
  }

  fn source_types(&self) -> &[SourceType] {
    &[SourceType::JavaScript]
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

  async fn build(
    &mut self,
    build_context: BuildContext<'_>,
  ) -> Result<TWithDiagnosticArray<BuildResult>> {
    let mut hasher = RspackHash::from(&build_context.compiler_options.output);
    self.update_hash(&mut hasher);
    let hash = hasher.digest(&build_context.compiler_options.output.hash_digest);

    let mut blocks = vec![];
    for (name, options) in &self.exposes {
      let mut block = AsyncDependenciesBlock::new(self.identifier, name);
      block.set_group_options(GroupOptions::ChunkGroup(ChunkGroupOptions {
        name: options.name.clone(),
      }));
      for request in options.import.iter() {
        let dep = ContainerExposedDependency::new(name.clone(), request.clone());
        block.add_dependency(Box::new(dep));
      }
      blocks.push(block);
    }

    Ok(
      BuildResult {
        build_info: BuildInfo {
          hash: Some(hash),
          strict: true,
          ..Default::default()
        },
        build_meta: BuildMeta {
          exports_type: BuildMetaExportsType::Namespace,
          ..Default::default()
        },
        dependencies: vec![],
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
      .insert(RuntimeGlobals::DEFINE_PROPERTY_GETTERS);
    code_generation_result
      .runtime_requirements
      .insert(RuntimeGlobals::HAS_OWN_PROPERTY);
    code_generation_result
      .runtime_requirements
      .insert(RuntimeGlobals::EXPORTS);
    let mut getters = vec![];
    for block_id in self.get_blocks() {
      let block = block_id.expect_get(compilation);
      let modules_iter = block.get_dependencies().iter().map(|dependency_id| {
        let dep = compilation
          .module_graph
          .dependency_by_id(dependency_id)
          .expect("should have dependency");
        let dep = dep
          .downcast_ref::<ContainerExposedDependency>()
          .expect("dependencies of ContainerEntryModule should be ContainerExposedDependency");
        let name = dep.exposed_name.as_str();
        let module = compilation.module_graph.get_module(dependency_id);
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
      getters.push(format!(
        "{}: {}",
        serde_json::to_string(name).expect("should able to json to_string"),
        basic_function("", &str)
      ))
    }
    let source = format!(
      r#"var moduleMap = {{
  {getters}
}};
var get = function(module, getScope) {{
  {current_remote_get_scope} = getScope;
  getScope = (
		{has_own_property}(moduleMap, module)
			? moduleMap[module]()
			: Promise.resolve().then(() => {{
				throw new Error('Module "' + module + '" does not exist in container.');
			}})
	);
	{current_remote_get_scope} = undefined;
	return getScope;
}};
var init = (shareScope, initScope) => {{
	if (!{share_scope_map}) return;
	var name = {share_scope}
	var oldScope = {share_scope_map}[name];
	if(oldScope && oldScope !== shareScope) throw new Error("Container initialization failed as it has already been initialized with a different share scope");
	{share_scope_map}[name] = shareScope;
	return {initialize_sharing}(name, initScope);
}};

// This exports getters to disallow modifications
{define_property_getters}(exports, {{
	get: () => (get),
	init: () => (init)
}});"#,
      getters = getters.join(",\n"),
      current_remote_get_scope = RuntimeGlobals::CURRENT_REMOTE_GET_SCOPE,
      has_own_property = RuntimeGlobals::HAS_OWN_PROPERTY,
      share_scope_map = RuntimeGlobals::SHARE_SCOPE_MAP,
      share_scope =
        serde_json::to_string(&self.share_scope).expect("should able to json to_string"),
      initialize_sharing = RuntimeGlobals::INITIALIZE_SHARING,
      define_property_getters = RuntimeGlobals::DEFINE_PROPERTY_GETTERS,
    );
    code_generation_result =
      code_generation_result.with_javascript(RawSource::from(source).boxed());
    Ok(code_generation_result)
  }
}

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
