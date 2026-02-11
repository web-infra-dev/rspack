use std::borrow::Cow;

use async_trait::async_trait;
use rspack_cacheable::{cacheable, cacheable_dyn, with::Unsupported};
use rspack_collections::{Identifiable, Identifier};
use rspack_core::{
  AsyncDependenciesBlock, AsyncDependenciesBlockIdentifier, BoxDependency, BoxModule, BuildContext,
  BuildInfo, BuildMeta, BuildResult, CodeGenerationResult, Compilation, Context, DependenciesBlock,
  DependencyId, ExportsType, FactoryMeta, LibIdentOptions, Module, ModuleCodeGenerationContext,
  ModuleGraph, ModuleIdentifier, ModuleType, RuntimeGlobals, RuntimeSpec, SourceType,
  impl_module_meta_info, impl_source_map_config, module_update_hash, rspack_sources::BoxSource,
};
use rspack_error::{Result, impl_empty_diagnosable_trait};
use rspack_hash::{RspackHash, RspackHashDigest};
use rspack_util::{ext::DynHash, source_map::SourceMapKind};

use super::{
  consume_shared_fallback_dependency::ConsumeSharedFallbackDependency,
  consume_shared_runtime_module::CodeGenerationDataConsumeShared,
};
use crate::{ConsumeOptions, utils::json_stringify};

#[impl_source_map_config]
#[cacheable]
#[derive(Debug)]
pub struct ConsumeSharedModule {
  #[cacheable(with=Unsupported)]
  blocks: Vec<AsyncDependenciesBlockIdentifier>,
  dependencies: Vec<DependencyId>,
  identifier: ModuleIdentifier,
  lib_ident: String,
  readable_identifier: String,
  context: Context,
  options: ConsumeOptions,
  factory_meta: Option<FactoryMeta>,
  build_info: BuildInfo,
  build_meta: BuildMeta,
}

impl ConsumeSharedModule {
  pub fn new(context: Context, options: ConsumeOptions) -> Self {
    let identifier = format!(
      "consume shared module ({}) {}@{}{}{}{}{}",
      &options.share_scope,
      &options.share_key,
      options
        .required_version
        .as_ref()
        .map_or_else(|| "*".to_string(), |v| v.to_string()),
      if options.strict_version {
        " (strict)"
      } else {
        Default::default()
      },
      if options.singleton {
        " (strict)"
      } else {
        Default::default()
      },
      options
        .import_resolved
        .as_ref()
        .map(|f| format!(" (fallback: {f})"))
        .unwrap_or_default(),
      if options.eager {
        " (eager)"
      } else {
        Default::default()
      },
    );
    Self {
      blocks: Vec::new(),
      dependencies: Vec::new(),
      identifier: ModuleIdentifier::from(identifier.as_ref()),
      lib_ident: format!(
        "webpack/sharing/consume/{}/{}{}",
        &options.share_scope,
        &options.share_key,
        options
          .import
          .as_ref()
          .map(|r| format!("/{r}"))
          .unwrap_or_default()
      ),
      readable_identifier: identifier,
      context,
      options,
      factory_meta: None,
      build_info: Default::default(),
      build_meta: Default::default(),
      source_map_kind: SourceMapKind::empty(),
    }
  }
}

impl Identifiable for ConsumeSharedModule {
  fn identifier(&self) -> Identifier {
    self.identifier
  }
}

impl DependenciesBlock for ConsumeSharedModule {
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
impl Module for ConsumeSharedModule {
  impl_module_meta_info!();

  fn size(&self, _source_type: Option<&SourceType>, _compilation: Option<&Compilation>) -> f64 {
    42.0
  }

  fn module_type(&self) -> &ModuleType {
    &ModuleType::ConsumeShared
  }

  fn source_types(&self, _module_graph: &ModuleGraph) -> &[SourceType] {
    &[SourceType::ConsumeShared]
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

  fn get_context(&self) -> Option<Box<Context>> {
    Some(Box::new(self.context.clone()))
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
    mut self: Box<Self>,
    _build_context: BuildContext,
    _: Option<&Compilation>,
  ) -> Result<BuildResult> {
    let mut blocks = vec![];
    let mut dependencies = vec![];
    if let Some(fallback) = &self.options.import {
      let dep = Box::new(ConsumeSharedFallbackDependency::new(fallback.to_owned()));
      if self.options.eager {
        dependencies.push(dep as BoxDependency);
      } else {
        let block = AsyncDependenciesBlock::new(self.identifier, None, None, vec![dep], None);
        blocks.push(Box::new(block));
      }
    }

    Ok(BuildResult {
      module: BoxModule::new(self),
      dependencies,
      blocks,
      optimization_bailouts: vec![],
    })
  }

  // #[tracing::instrument("ConsumeSharedModule::code_generation", skip_all, fields(identifier = ?self.identifier()))]
  async fn code_generation(
    &self,
    code_generation_context: &mut ModuleCodeGenerationContext,
  ) -> Result<CodeGenerationResult> {
    let ModuleCodeGenerationContext {
      compilation,
      runtime_template,
      ..
    } = code_generation_context;

    let mut code_generation_result = CodeGenerationResult::default();
    runtime_template
      .runtime_requirements_mut()
      .insert(RuntimeGlobals::SHARE_SCOPE_MAP);

    let mut function = String::from("loaders.load");
    let mut args = vec![
      json_stringify(&self.options.share_scope),
      json_stringify(&self.options.share_key),
    ];
    if let Some(version) = &self.options.required_version {
      if self.options.strict_version {
        function += "Strict";
      }
      if self.options.singleton {
        function += "Singleton";
      }
      let version = json_stringify(&version.to_string());
      args.push(format!("loaders.parseRange({version})"));
      function += "VersionCheck";
    } else if self.options.singleton {
      function += "Singleton";
    }
    let factory = self.options.import.as_ref().map(|fallback| {
      if self.options.eager {
        runtime_template.sync_module_factory(&self.get_dependencies()[0], fallback, compilation)
      } else {
        runtime_template.async_module_factory(&self.get_blocks()[0], fallback, compilation)
      }
    });
    code_generation_result
      .data
      .insert(CodeGenerationDataConsumeShared {
        share_scope: self.options.share_scope.clone(),
        share_key: self.options.share_key.clone(),
        import: self.options.import.clone(),
        required_version: self.options.required_version.clone(),
        strict_version: self.options.strict_version,
        singleton: self.options.singleton,
        eager: self.options.eager,
        fallback: factory,
        tree_shaking_mode: self.options.tree_shaking_mode.clone(),
      });
    Ok(code_generation_result)
  }

  async fn get_runtime_hash(
    &self,
    compilation: &Compilation,
    runtime: Option<&RuntimeSpec>,
  ) -> Result<RspackHashDigest> {
    let mut hasher = RspackHash::from(&compilation.options.output);
    self.options.dyn_hash(&mut hasher);
    module_update_hash(self, &mut hasher, compilation, runtime);
    Ok(hasher.digest(&compilation.options.output.hash_digest))
  }
}

impl_empty_diagnosable_trait!(ConsumeSharedModule);
