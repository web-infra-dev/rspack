use std::{borrow::Cow, sync::Arc};

use async_trait::async_trait;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_collections::{Identifiable, Identifier};
use rspack_core::{
  AsyncDependenciesBlock, AsyncDependenciesBlockIdentifier, BoxDependency, BoxModule, BuildContext,
  BuildInfo, BuildMeta, BuildMetaExportsType, BuildResult, CodeGenerationResult, Compilation,
  Context, DependenciesBlock, Dependency, DependencyId, DependencyRange, FactoryMeta, ImportPhase,
  LibIdentOptions, Module, ModuleCodeGenerationContext, ModuleDependency, ModuleGraph,
  ModuleIdentifier, ModuleLayer, ModuleType, ReferencedSpecifier, RuntimeSpec, SourceType,
  contextify, impl_module_meta_info, impl_source_map_config, module_update_hash,
  rspack_sources::{BoxSource, RawStringSource, SourceExt},
  to_comment,
};
use rspack_error::{Result, impl_empty_diagnosable_trait};
use rspack_hash::{RspackHash, RspackHashDigest};
use rspack_plugin_javascript::dependency::ImportEagerDependency;
use rspack_util::{fx_hash::FxIndexSet, source_map::SourceMapKind};
use rustc_hash::FxHashSet;
use swc_core::ecma::atoms::Atom;

use crate::{
  client_reference_dependency::ClientReferenceDependency, constants::LAYERS_NAMES,
  plugin_state::ClientModuleImport,
};

#[impl_source_map_config]
#[cacheable]
#[derive(Debug)]
pub struct RscEntryModule {
  blocks: Vec<AsyncDependenciesBlockIdentifier>,
  dependencies: Vec<DependencyId>,
  identifier: ModuleIdentifier,
  lib_ident: String,
  client_modules: Vec<ClientModuleImport>,
  name: Arc<str>,
  /// When true, client modules are loaded eagerly (not as code-split points).
  is_server_side_rendering: bool,
  factory_meta: Option<FactoryMeta>,
  build_info: BuildInfo,
  build_meta: BuildMeta,
  layer: Option<ModuleLayer>,
}

impl RscEntryModule {
  pub fn new(
    name: Arc<str>,
    client_modules: Vec<ClientModuleImport>,
    is_server_side_rendering: bool,
  ) -> Self {
    let lib_ident = format!("rspack/rsc-entry?name={}", &name);
    let identifier = ModuleIdentifier::from(format!(
      "rsc entry ({}) [{}]",
      name,
      client_modules
        .iter()
        .map(|m| m.request.as_str())
        .collect::<Vec<_>>()
        .join(", ")
    ));
    let layer = if is_server_side_rendering {
      Some(LAYERS_NAMES.server_side_rendering.to_string())
    } else {
      None
    };

    Self {
      blocks: Vec::new(),
      dependencies: Vec::new(),
      identifier,
      lib_ident,
      client_modules,
      name,
      is_server_side_rendering,
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
      layer,
    }
  }
}

impl Identifiable for RscEntryModule {
  fn identifier(&self) -> Identifier {
    self.identifier
  }
}

impl DependenciesBlock for RscEntryModule {
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
impl Module for RscEntryModule {
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
    format!("rsc client entry {}", self.name).into()
  }

  fn lib_ident(&self, _options: LibIdentOptions) -> Option<Cow<'_, str>> {
    Some(self.lib_ident.as_str().into())
  }

  fn get_layer(&self) -> Option<&ModuleLayer> {
    self.layer.as_ref()
  }

  async fn build(
    mut self: Box<Self>,
    _build_context: BuildContext,
    _: Option<&Compilation>,
  ) -> Result<BuildResult> {
    if self.is_server_side_rendering {
      // Eager: no code-split points; use ImportEagerDependency (CSS filtering done at call site).
      let mut dependencies: Vec<BoxDependency> = vec![];
      let modules: Vec<(String, FxIndexSet<Atom>)> = self
        .client_modules
        .iter()
        .map(|m| (m.request.clone(), m.ids.clone()))
        .collect();
      for (request, ids) in &modules {
        let referenced_specifiers = if ids.is_empty() || ids.iter().any(|id| id == "*") {
          None
        } else {
          Some(
            ids
              .iter()
              .map(|id| ReferencedSpecifier::new(vec![Atom::from(id.as_str())]))
              .collect::<Vec<_>>(),
          )
        };
        let dep = ImportEagerDependency::new(
          Atom::from(request.as_str()),
          DependencyRange { start: 0, end: 0 },
          referenced_specifiers,
          None,
          ImportPhase::Evaluation,
        );
        self.add_dependency_id(*dep.id());
        dependencies.push(Box::new(dep));
      }
      Ok(BuildResult {
        module: BoxModule::new(self),
        dependencies,
        blocks: vec![],
        optimization_bailouts: vec![],
        parser_and_generator: None,
      })
    } else {
      // Non-eager: code-split points; use AsyncDependenciesBlock + ClientReferenceDependency.
      let mut blocks = vec![];
      let dependencies: Vec<BoxDependency> = vec![];

      for client_module in &self.client_modules {
        let dep = ClientReferenceDependency::new(
          client_module.request.clone(),
          client_module.ids.clone(),
          self.is_server_side_rendering,
        );
        let block = AsyncDependenciesBlock::new(
          self.identifier,
          None,
          None,
          vec![Box::new(dep) as Box<dyn Dependency>],
          Some(client_module.request.clone()),
        );
        blocks.push(Box::new(block));
      }

      Ok(BuildResult {
        module: BoxModule::new(self),
        dependencies,
        blocks,
        optimization_bailouts: vec![],
        parser_and_generator: None,
      })
    }
  }

  async fn code_generation(
    &self,
    code_generation_context: &mut ModuleCodeGenerationContext,
  ) -> Result<CodeGenerationResult> {
    let ModuleCodeGenerationContext { compilation, .. } = code_generation_context;

    let mut code_generation_result = CodeGenerationResult::default();
    let module_graph = compilation.get_module_graph();

    if self.is_server_side_rendering {
      let mut comments = Vec::new();
      for dep_id in self.get_dependencies() {
        let dependency = module_graph.dependency_by_id(dep_id);
        let dep = dependency
          .downcast_ref::<ImportEagerDependency>()
          .unwrap_or_else(|| {
            panic!(
              "Expected dependency of eager RscEntryModule to be ImportEagerDependency, got {:?}",
              dependency.dependency_type()
            )
          });
        let comment = to_comment(&contextify(
          compilation.options.context.as_path(),
          dep.request(),
        ));
        comments.push(comment);
      }
      let source = comments.join("\n");
      code_generation_result =
        code_generation_result.with_javascript(RawStringSource::from(source).boxed());
    } else {
      let mut comments = Vec::new();
      for block_id in self.get_blocks() {
        let block = module_graph
          .block_by_id(block_id)
          .expect("should have block");

        for dependency_id in block.get_dependencies() {
          let dependency = module_graph.dependency_by_id(dependency_id);
          let request = dependency
            .downcast_ref::<ClientReferenceDependency>()
            .unwrap_or_else(|| {
              panic!(
                "Expected dependency of RscEntryModule to be ClientReferenceDependency, got {:?}",
                dependency.dependency_type()
              )
            })
            .user_request();

          let comment = to_comment(&contextify(compilation.options.context.as_path(), request));
          comments.push(comment);
        }
      }
      let source = comments.join("\n");
      code_generation_result =
        code_generation_result.with_javascript(RawStringSource::from(source).boxed());
    }
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

impl_empty_diagnosable_trait!(RscEntryModule);
