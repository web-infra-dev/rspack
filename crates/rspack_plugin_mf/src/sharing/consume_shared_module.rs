use std::{borrow::Cow, hash::Hasher};

use async_trait::async_trait;
use rspack_cacheable::{cacheable, cacheable_dyn, with::Unsupported};
use rspack_collections::{Identifiable, Identifier};
use rspack_core::{
  async_module_factory, impl_module_meta_info, impl_source_map_config, module_update_hash,
  rspack_sources::BoxSource, sync_module_factory, AsyncDependenciesBlock,
  AsyncDependenciesBlockIdentifier, BoxDependency, BuildContext, BuildInfo, BuildMeta, BuildResult,
  CodeGenerationResult, Compilation, ConcatenationScope, Context, DependenciesBlock, DependencyId,
  DependencyType, FactoryMeta, LibIdentOptions, Module, ModuleGraph, ModuleIdentifier, ModuleType,
  RuntimeGlobals, RuntimeSpec, SourceType,
};
use rspack_error::{error, impl_empty_diagnosable_trait, Result};
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
  pub fn new(context: Context, options: ConsumeOptions) -> Result<Self> {
    // Validate configuration
    if options.share_key.is_empty() {
      return Err(error!("share_key cannot be empty"));
    }

    if options.share_scope.is_empty() {
      return Err(error!("share_scope cannot be empty"));
    }

    let version_str = options
      .required_version
      .as_ref()
      .map(|v| v.to_string())
      .unwrap_or_else(|| "*".to_string());

    let identifier = format!(
      "consume shared module ({}) {}@{}{}{}{}{}",
      &options.share_scope,
      &options.share_key,
      version_str,
      if options.strict_version {
        " (strict)"
      } else {
        ""
      },
      if options.singleton {
        " (singleton)"
      } else {
        ""
      },
      options
        .import_resolved
        .as_ref()
        .map(|f| format!(" (fallback: {f})"))
        .unwrap_or_default(),
      if options.eager { " (eager)" } else { "" },
    );

    let lib_ident = format!(
      "webpack/sharing/consume/{}/{}{}",
      &options.share_scope,
      &options.share_key,
      options
        .import
        .as_ref()
        .map(|r| format!("/{r}"))
        .unwrap_or_default()
    );

    Ok(Self {
      blocks: Vec::new(),
      dependencies: Vec::new(),
      identifier: ModuleIdentifier::from(identifier.as_ref()),
      lib_ident,
      readable_identifier: identifier,
      context,
      options,
      factory_meta: None,
      build_info: Default::default(),
      build_meta: Default::default(),
      source_map_kind: SourceMapKind::empty(),
    })
  }

  /// Finds the fallback module identifier with enhanced validation
  pub fn find_fallback_module_id(
    &self,
    module_graph: &ModuleGraph,
  ) -> Result<Option<ModuleIdentifier>> {
    for dep_id in self.get_dependencies() {
      let dep = module_graph.dependency_by_id(dep_id).ok_or_else(|| {
        error!(
          "Failed to resolve fallback module: {}. Dependency not found for ID: {dep_id:?}",
          self.identifier
        )
      })?;

      if matches!(dep.dependency_type(), DependencyType::ConsumeSharedFallback) {
        if let Some(fallback_id) = module_graph.module_identifier_by_dependency_id(dep_id) {
          // Validate that the fallback module actually exists
          if module_graph.module_by_identifier(fallback_id).is_some() {
            return Ok(Some(*fallback_id));
          } else {
            return Err(error!("Failed to resolve fallback module: {}. Fallback module identifier exists but module not found in graph", fallback_id));
          }
        }
      }
    }
    Ok(None)
  }

  /// Validates the consume shared configuration
  pub fn validate_configuration(&self) -> Result<()> {
    if self.options.strict_version && self.options.required_version.is_none() {
      return Err(error!(
        "Invalid consume shared configuration: strict_version requires required_version to be set"
      ));
    }

    if self.options.singleton && self.options.eager && self.options.import.is_some() {
      return Err(error!("Invalid consume shared configuration: singleton eager mode with fallback is not recommended"));
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
    _compilation: Option<&Compilation>,
  ) -> Result<BuildResult> {
    // Validate configuration during build
    self.validate_configuration()?;

    let mut blocks = vec![];
    let mut dependencies = vec![];

    if let Some(fallback) = &self.options.import {
      // Validate fallback request
      if fallback.trim().is_empty() {
        return Err(error!(
          "Invalid consume shared configuration: fallback import cannot be empty or whitespace"
        ));
      }

      let dep = Box::new(ConsumeSharedFallbackDependency::new(fallback.to_owned()));

      if self.options.eager {
        dependencies.push(dep as BoxDependency);
      } else {
        let block = AsyncDependenciesBlock::new(self.identifier, None, None, vec![dep], None);
        blocks.push(Box::new(block));
      }

      // Note: singleton with fallback is not recommended for performance
    }

    Ok(BuildResult {
      dependencies,
      blocks,
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
      .insert(RuntimeGlobals::SHARE_SCOPE_MAP);

    let mut function = String::from("loaders.load");
    let mut args = vec![
      json_stringify(&self.options.share_scope),
      json_stringify(&self.options.share_key),
    ];

    // Build function name based on options
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

    // Handle factory creation with error handling
    let factory = self
      .options
      .import
      .as_ref()
      .map(|fallback| {
        if self.options.eager {
          // Validate dependencies exist
          let deps = self.get_dependencies();
          if deps.is_empty() {
            return Err(error!("Module factory creation failed for {}", fallback));
          }
          Ok(sync_module_factory(
            &deps[0],
            fallback,
            compilation,
            &mut code_generation_result.runtime_requirements,
          ))
        } else {
          // Validate blocks exist
          let blocks = self.get_blocks();
          if blocks.is_empty() {
            return Err(error!("Module factory creation failed for {}", fallback));
          }
          Ok(async_module_factory(
            &blocks[0],
            fallback,
            compilation,
            &mut code_generation_result.runtime_requirements,
          ))
        }
      })
      .transpose()?;

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

    // Add additional context to hash for better cache invalidation
    hasher.write(self.options.share_scope.as_bytes());
    hasher.write(self.options.share_key.as_bytes());
    if let Some(version) = &self.options.required_version {
      hasher.write(version.to_string().as_bytes());
    }
    hasher.write(&[
      self.options.strict_version as u8,
      self.options.singleton as u8,
      self.options.eager as u8,
    ]);

    self.options.dyn_hash(&mut hasher);
    module_update_hash(self as &dyn Module, &mut hasher, compilation, runtime);
    Ok(hasher.digest(&compilation.options.output.hash_digest))
  }

  fn get_consume_shared_key(&self) -> Option<String> {
    Some(self.options.share_key.clone())
  }
}

impl_empty_diagnosable_trait!(ConsumeSharedModule);
