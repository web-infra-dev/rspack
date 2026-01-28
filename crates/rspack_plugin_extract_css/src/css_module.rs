use std::hash::Hash;

use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_collections::{Identifiable, Identifier};
use rspack_core::{
  AsyncDependenciesBlockIdentifier, BuildContext, BuildInfo, BuildMeta, BuildResult,
  CodeGenerationResult, Compilation, CompilerOptions, DependenciesBlock, DependencyId, FactoryMeta,
  Module, ModuleCodeGenerationContext, ModuleExt, ModuleFactory, ModuleFactoryCreateData,
  ModuleFactoryResult, ModuleGraph, ModuleLayer, RuntimeSpec, SourceType, impl_module_meta_info,
  impl_source_map_config, module_update_hash, rspack_sources::BoxSource,
};
use rspack_error::{Result, impl_empty_diagnosable_trait};
use rspack_hash::{RspackHash, RspackHashDigest};
use rspack_util::{ext::DynHash, itoa};

use crate::{
  css_dependency::CssDependency,
  plugin::{MODULE_TYPE, SOURCE_TYPE},
};

#[impl_source_map_config]
#[cacheable]
#[derive(Debug)]
pub(crate) struct CssModule {
  pub(crate) identifier: String,
  pub(crate) content: String,
  pub(crate) _context: String,
  pub(crate) module_layer: Option<ModuleLayer>,
  pub(crate) media: Option<String>,
  pub(crate) supports: Option<String>,
  pub(crate) source_map: Option<String>,
  pub(crate) css_layer: Option<String>,
  pub(crate) identifier_index: u32,

  factory_meta: Option<FactoryMeta>,
  build_info: BuildInfo,
  build_meta: BuildMeta,

  blocks: Vec<AsyncDependenciesBlockIdentifier>,
  dependencies: Vec<DependencyId>,

  identifier__: Identifier,
}

impl CssModule {
  pub fn new(dep: CssDependency) -> Self {
    let mut identifier_index_buffer = itoa::Buffer::new();
    let identifier_index_str = identifier_index_buffer.format(dep.identifier_index);
    let identifier__ = format!(
      "css|{}|{}|{}|{}|{}}}",
      dep.identifier,
      identifier_index_str,
      dep.css_layer.as_deref().unwrap_or_default(),
      dep.supports.as_deref().unwrap_or_default(),
      dep.media.as_deref().unwrap_or_default(),
    )
    .into();

    Self {
      identifier: dep.identifier,
      content: dep.content,
      module_layer: dep.module_layer.clone(),
      css_layer: dep.css_layer.clone(),
      _context: dep.context,
      media: dep.media,
      supports: dep.supports,
      source_map: dep.source_map,
      identifier_index: dep.identifier_index,
      blocks: vec![],
      dependencies: vec![],
      factory_meta: None,
      build_info: BuildInfo {
        cacheable: dep.cacheable,
        strict: true,
        file_dependencies: dep.file_dependencies,
        context_dependencies: dep.context_dependencies,
        missing_dependencies: dep.missing_dependencies,
        build_dependencies: dep.build_dependencies,
        ..Default::default()
      },
      build_meta: Default::default(),
      source_map_kind: rspack_util::source_map::SourceMapKind::empty(),
      identifier__,
    }
  }

  fn compute_hash(&self, options: &CompilerOptions) -> RspackHashDigest {
    let mut hasher = RspackHash::from(&options.output);

    self.content.hash(&mut hasher);
    if let Some(layer) = &self.css_layer {
      layer.hash(&mut hasher);
    }
    self.supports.hash(&mut hasher);
    self.media.hash(&mut hasher);
    self.source_map.hash(&mut hasher);

    hasher.digest(&options.output.hash_digest)
  }
}

#[cacheable_dyn]
#[async_trait::async_trait]
impl Module for CssModule {
  impl_module_meta_info!();

  fn readable_identifier(&self, context: &rspack_core::Context) -> std::borrow::Cow<'_, str> {
    let index_suffix = if self.identifier_index > 0 {
      let mut index_buffer = itoa::Buffer::new();
      let index_str = index_buffer.format(self.identifier_index);
      format!("({})", index_str)
    } else {
      "".into()
    };
    std::borrow::Cow::Owned(format!(
      "css {}{}{}{}{}",
      context.shorten(&self.identifier),
      index_suffix,
      if let Some(layer) = &self.css_layer {
        format!(" (layer {layer})")
      } else {
        "".into()
      },
      if let Some(supports) = &self.supports
        && !supports.is_empty()
      {
        format!(" (supports {supports})")
      } else {
        "".into()
      },
      if let Some(media) = &self.media
        && !media.is_empty()
      {
        format!(" (media {media})")
      } else {
        "".into()
      }
    ))
  }

  fn name_for_condition(&self) -> Option<Box<str>> {
    self
      .identifier
      .split('!')
      .next_back()
      .map(|resource| resource.split('?').next().unwrap_or(resource).into())
  }

  fn size(&self, _source_type: Option<&SourceType>, _compilation: Option<&Compilation>) -> f64 {
    self.content.len() as f64
  }

  fn source(&self) -> Option<&BoxSource> {
    None
  }

  fn module_type(&self) -> &rspack_core::ModuleType {
    &MODULE_TYPE
  }

  fn source_types(&self, _module_graph: &ModuleGraph) -> &[SourceType] {
    &*SOURCE_TYPE
  }

  fn need_id(&self) -> bool {
    false
  }

  async fn build(
    &mut self,
    build_context: BuildContext,
    _compilation: Option<&Compilation>,
  ) -> Result<BuildResult> {
    self.build_info.hash = Some(self.compute_hash(&build_context.compiler_options));
    Ok(Default::default())
  }

  // #[tracing::instrument("ExtractCssModule::code_generation", skip_all, fields(identifier = ?self.identifier()))]
  async fn code_generation(
    &self,
    _code_generation_context: &mut ModuleCodeGenerationContext,
  ) -> Result<CodeGenerationResult> {
    Ok(CodeGenerationResult::default())
  }

  async fn get_runtime_hash(
    &self,
    compilation: &Compilation,
    runtime: Option<&RuntimeSpec>,
  ) -> Result<RspackHashDigest> {
    let mut hasher = RspackHash::from(&compilation.options.output);
    module_update_hash(self, &mut hasher, compilation, runtime);
    self.build_info.hash.dyn_hash(&mut hasher);
    Ok(hasher.digest(&compilation.options.output.hash_digest))
  }

  fn get_layer(&self) -> Option<&ModuleLayer> {
    self.module_layer.as_ref()
  }
}

impl Identifiable for CssModule {
  fn identifier(&self) -> rspack_collections::Identifier {
    self.identifier__
  }
}

impl DependenciesBlock for CssModule {
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

#[derive(Debug)]
pub(crate) struct CssModuleFactory;

#[async_trait::async_trait]
impl ModuleFactory for CssModuleFactory {
  async fn create(&self, data: &mut ModuleFactoryCreateData) -> Result<ModuleFactoryResult> {
    let css_dep = data.dependencies[0]
      .downcast_ref::<CssDependency>()
      .expect("unreachable");

    Ok(ModuleFactoryResult::new_with_module(
      CssModule::new(css_dep.clone()).boxed(),
    ))
  }
}

impl_empty_diagnosable_trait!(CssModule);
impl_empty_diagnosable_trait!(CssModuleFactory);
