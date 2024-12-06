use std::borrow::Cow;

use async_trait::async_trait;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_collections::{Identifiable, Identifier};
use rspack_core::{
  async_module_factory, impl_module_meta_info, impl_source_map_config, rspack_sources::Source,
  sync_module_factory, AsyncDependenciesBlock, AsyncDependenciesBlockIdentifier, BoxDependency,
  BuildContext, BuildInfo, BuildMeta, BuildResult, CodeGenerationResult, Compilation, Context,
  DependenciesBlock, DependencyId, LibIdentOptions, Module, ModuleIdentifier, ModuleType,
  RuntimeGlobals, RuntimeSpec, SourceType,
};
use rspack_core::{module_update_hash, ConcatenationScope, FactoryMeta};
use rspack_error::{impl_empty_diagnosable_trait, Diagnostic, Result};
use rspack_util::ext::DynHash;
use rspack_util::source_map::SourceMapKind;

use super::{
  consume_shared_fallback_dependency::ConsumeSharedFallbackDependency,
  consume_shared_runtime_module::CodeGenerationDataConsumeShared,
};
use crate::{utils::json_stringify, ConsumeOptions};

#[impl_source_map_config]
#[cacheable]
#[derive(Debug)]
pub struct ConsumeSharedModule {
  blocks: Vec<AsyncDependenciesBlockIdentifier>,
  dependencies: Vec<DependencyId>,
  identifier: ModuleIdentifier,
  lib_ident: String,
  readable_identifier: String,
  context: Context,
  options: ConsumeOptions,
  factory_meta: Option<FactoryMeta>,
  build_info: Option<BuildInfo>,
  build_meta: Option<BuildMeta>,
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
      options
        .strict_version
        .then_some(" (strict)")
        .unwrap_or_default(),
      options.singleton.then_some(" (strict)").unwrap_or_default(),
      options
        .import_resolved
        .as_ref()
        .map(|f| format!(" (fallback: {f})"))
        .unwrap_or_default(),
      options.eager.then_some(" (eager)").unwrap_or_default(),
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
          .map(|r| format!("/{}", r))
          .unwrap_or_default()
      ),
      readable_identifier: identifier,
      context,
      options,
      factory_meta: None,
      build_info: None,
      build_meta: None,
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

  fn get_diagnostics(&self) -> Vec<Diagnostic> {
    vec![]
  }

  fn module_type(&self) -> &ModuleType {
    &ModuleType::ConsumeShared
  }

  fn source_types(&self) -> &[SourceType] {
    &[SourceType::ConsumeShared]
  }

  fn original_source(&self) -> Option<&dyn Source> {
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
      build_info: Default::default(),
      build_meta: Default::default(),
      dependencies,
      blocks,
      ..Default::default()
    })
  }

  #[tracing::instrument(name = "ConsumeSharedModule::code_generation", skip_all, fields(identifier = ?self.identifier()))]
  fn code_generation(
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
      args.push(format!("loaders.parseRange({})", version));
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

  fn update_hash(
    &self,
    hasher: &mut dyn std::hash::Hasher,
    compilation: &Compilation,
    runtime: Option<&RuntimeSpec>,
  ) -> Result<()> {
    self.options.dyn_hash(hasher);
    module_update_hash(self, hasher, compilation, runtime);
    Ok(())
  }
}

impl_empty_diagnosable_trait!(ConsumeSharedModule);
