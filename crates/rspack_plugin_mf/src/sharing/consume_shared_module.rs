use std::borrow::Cow;

use async_trait::async_trait;
use rspack_cacheable::{cacheable, cacheable_dyn, with::Unsupported};
use rspack_collections::{Identifiable, Identifier};
use rspack_core::{
  async_module_factory, impl_module_meta_info, impl_source_map_config, module_update_hash,
  rspack_sources::BoxSource, sync_module_factory, AsyncDependenciesBlock,
  AsyncDependenciesBlockIdentifier, BoxDependency, BuildContext, BuildInfo, BuildMeta, BuildResult,
  CodeGenerationResult, Compilation, ConcatenationScope, Context, DependenciesBlock, DependencyId,
  DependencyType, ExportsInfoGetter, FactoryMeta, LibIdentOptions, Module, ModuleGraph,
  ModuleIdentifier, ModuleType, PrefetchExportsInfoMode, ProvidedExports, RuntimeGlobals,
  RuntimeSpec, SourceType,
};
use rspack_error::{impl_empty_diagnosable_trait, Result};
use rspack_hash::{RspackHash, RspackHashDigest};
use rspack_util::{ext::DynHash, source_map::SourceMapKind};

use super::{
  consume_shared_fallback_dependency::ConsumeSharedFallbackDependency,
  consume_shared_runtime_module::CodeGenerationDataConsumeShared,
};
use crate::{utils::json_stringify, ConsumeOptions};

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
        .map(|v| v.to_string())
        .unwrap_or_else(|| "*".to_string()),
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

  pub fn get_share_key(&self) -> &str {
    &self.options.share_key
  }

  /// Copies metadata from the fallback module to make this ConsumeSharedModule act as a true proxy
  pub fn copy_metadata_from_fallback(&mut self, module_graph: &mut ModuleGraph) -> Result<()> {
    if let Some(fallback_id) = self.find_fallback_module_id(module_graph) {
      // Copy build meta from fallback module
      if let Some(fallback_module) = module_graph.module_by_identifier(&fallback_id) {
        // Copy build meta information
        self.build_meta = fallback_module.build_meta().clone();
        self.build_info = fallback_module.build_info().clone();

        // Copy export information from fallback module to ConsumeShared module
        self.copy_exports_from_fallback(module_graph, &fallback_id)?;
      }
    }
    Ok(())
  }

  /// Finds the fallback module identifier for this ConsumeShared module
  pub fn find_fallback_module_id(&self, module_graph: &ModuleGraph) -> Option<ModuleIdentifier> {
    // Look through dependencies to find the fallback
    for dep_id in self.get_dependencies() {
      if let Some(dep) = module_graph.dependency_by_id(dep_id) {
        if matches!(dep.dependency_type(), DependencyType::ConsumeSharedFallback) {
          if let Some(fallback_id) = module_graph.module_identifier_by_dependency_id(dep_id) {
            return Some(*fallback_id);
          }
        }
      }
    }
    None
  }

  /// Copies export information from fallback module to ConsumeShared module
  fn copy_exports_from_fallback(
    &self,
    module_graph: &mut ModuleGraph,
    fallback_id: &ModuleIdentifier,
  ) -> Result<()> {
    let consume_shared_id = self.identifier();

    // Get exports info for both modules
    let fallback_exports_info = module_graph.get_exports_info(fallback_id);
    let consume_shared_exports_info = module_graph.get_exports_info(&consume_shared_id);

    // Get the fallback module's provided exports using prefetched mode
    let prefetched_fallback = ExportsInfoGetter::prefetch(
      &fallback_exports_info,
      module_graph,
      PrefetchExportsInfoMode::Full,
    );

    let fallback_provided = prefetched_fallback.get_provided_exports();

    // Copy the provided exports to the ConsumeShared module
    match fallback_provided {
      ProvidedExports::ProvidedNames(export_names) => {
        // Copy each specific export from fallback to ConsumeShared
        for export_name in export_names {
          // Get or create export info for this export in the ConsumeShared module
          let consume_shared_export_info =
            consume_shared_exports_info.get_export_info(module_graph, &export_name);
          let fallback_export_info =
            fallback_exports_info.get_export_info(module_graph, &export_name);

          // Copy the provided status
          if let Some(provided) = fallback_export_info.as_data(module_graph).provided() {
            consume_shared_export_info
              .as_data_mut(module_graph)
              .set_provided(Some(provided));
          }

          // Copy can_mangle_provide status
          if let Some(can_mangle) = fallback_export_info
            .as_data(module_graph)
            .can_mangle_provide()
          {
            consume_shared_export_info
              .as_data_mut(module_graph)
              .set_can_mangle_provide(Some(can_mangle));
          }

          // Copy exports_info if it exists (for nested exports)
          if let Some(nested_exports_info) =
            fallback_export_info.as_data(module_graph).exports_info()
          {
            consume_shared_export_info
              .as_data_mut(module_graph)
              .set_exports_info(Some(nested_exports_info));
          }
        }

        // Mark the ConsumeShared module as having complete provide info
        consume_shared_exports_info.set_has_provide_info(module_graph);

        // Set the "other exports" to not provided (since we copied all specific exports)
        consume_shared_exports_info.set_unknown_exports_provided(
          module_graph,
          false, // not provided
          None,  // no exclude exports
          None,  // no can_mangle
          None,  // no terminal_binding
          None,  // no target_key
        );
      }
      ProvidedExports::ProvidedAll => {
        // If fallback provides all exports, mark ConsumeShared the same way
        consume_shared_exports_info.set_unknown_exports_provided(
          module_graph,
          true, // provided
          None, // no exclude exports
          None, // no can_mangle
          None, // no terminal_binding
          None, // no target_key
        );
        consume_shared_exports_info.set_has_provide_info(module_graph);
      }
      ProvidedExports::Unknown => {
        // Keep unknown status - don't copy anything
      }
    }

    Ok(())
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

  fn readable_identifier(&self, _context: &Context) -> Cow<str> {
    self.readable_identifier.as_str().into()
  }

  fn lib_ident(&self, _options: LibIdentOptions) -> Option<Cow<str>> {
    Some(self.lib_ident.as_str().into())
  }

  fn get_context(&self) -> Option<Box<Context>> {
    Some(Box::new(self.context.clone()))
  }

  async fn build(
    &mut self,
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
      dependencies,
      blocks,
      ..Default::default()
    })
  }

  // #[tracing::instrument("ConsumeSharedModule::code_generation", skip_all, fields(identifier = ?self.identifier()))]
  async fn code_generation(
    &self,
    compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
    _: Option<ConcatenationScope>,
  ) -> Result<CodeGenerationResult> {
    let mut code_generation_result = CodeGenerationResult::default();
    code_generation_result
      .runtime_requirements
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
        sync_module_factory(
          &self.get_dependencies()[0],
          fallback,
          compilation,
          &mut code_generation_result.runtime_requirements,
        )
      } else {
        async_module_factory(
          &self.get_blocks()[0],
          fallback,
          compilation,
          &mut code_generation_result.runtime_requirements,
        )
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
    module_update_hash(self as &dyn Module, &mut hasher, compilation, runtime);
    Ok(hasher.digest(&compilation.options.output.hash_digest))
  }

  /// Get the share_key for ConsumeShared modules.
  /// Returns Some(share_key) for ConsumeShared modules, None for others.
  fn get_consume_shared_key(&self) -> Option<String> {
    Some(self.options.share_key.clone())
  }
}

impl_empty_diagnosable_trait!(ConsumeSharedModule);
