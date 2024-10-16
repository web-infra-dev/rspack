use std::hash::Hash;
use std::path::PathBuf;
use std::sync::LazyLock;

use rspack_collections::{Identifiable, Identifier};
use rspack_core::rspack_sources::Source;
use rspack_core::{
  impl_module_meta_info, impl_source_map_config, module_update_hash,
  AsyncDependenciesBlockIdentifier, BuildContext, BuildInfo, BuildMeta, BuildResult,
  CodeGenerationResult, Compilation, CompilerOptions, ConcatenationScope, DependenciesBlock,
  DependencyId, DependencyType, FactoryMeta, Module, ModuleFactory, ModuleFactoryCreateData,
  ModuleFactoryResult, RuntimeSpec, SourceType,
};
use rspack_error::Result;
use rspack_error::{impl_empty_diagnosable_trait, Diagnostic};
use rspack_hash::{RspackHash, RspackHashDigest};
use rspack_util::ext::DynHash;
use rspack_util::itoa;
use rustc_hash::FxHashSet;

use crate::css_dependency::CssDependency;
use crate::plugin::{MODULE_TYPE, SOURCE_TYPE};

pub(crate) static DEPENDENCY_TYPE: LazyLock<DependencyType> =
  LazyLock::new(|| DependencyType::Custom("mini-extract-dep"));

#[impl_source_map_config]
#[derive(Debug)]
pub(crate) struct CssModule {
  pub(crate) identifier: String,
  pub(crate) content: String,
  pub(crate) _context: String,
  pub(crate) media: Option<String>,
  pub(crate) supports: Option<String>,
  pub(crate) source_map: Option<String>,
  pub(crate) layer: Option<String>,
  pub(crate) identifier_index: u32,

  factory_meta: Option<FactoryMeta>,
  build_info: Option<BuildInfo>,
  build_meta: Option<BuildMeta>,

  blocks: Vec<AsyncDependenciesBlockIdentifier>,
  dependencies: Vec<DependencyId>,

  identifier__: Identifier,
  cacheable: bool,
  file_dependencies: FxHashSet<PathBuf>,
  context_dependencies: FxHashSet<PathBuf>,
  missing_dependencies: FxHashSet<PathBuf>,
  build_dependencies: FxHashSet<PathBuf>,
}

impl CssModule {
  pub fn new(dep: CssDependency) -> Self {
    let identifier__ = format!(
      "css|{}|{}|{}|{}|{}}}",
      dep.identifier,
      itoa!(dep.identifier_index),
      dep.layer.as_deref().unwrap_or_default(),
      dep.supports.as_deref().unwrap_or_default(),
      dep.media.as_deref().unwrap_or_default(),
    )
    .into();

    Self {
      identifier: dep.identifier,
      content: dep.content,
      layer: dep.layer.clone(),
      _context: dep.context,
      media: dep.media,
      supports: dep.supports,
      source_map: dep.source_map,
      identifier_index: dep.identifier_index,
      blocks: vec![],
      dependencies: vec![],
      factory_meta: None,
      build_info: None,
      build_meta: None,
      source_map_kind: rspack_util::source_map::SourceMapKind::empty(),
      identifier__,
      cacheable: dep.cacheable,
      file_dependencies: dep.file_dependencies,
      context_dependencies: dep.context_dependencies,
      missing_dependencies: dep.missing_dependencies,
      build_dependencies: dep.build_dependencies,
    }
  }

  fn compute_hash(&self, options: &CompilerOptions) -> RspackHashDigest {
    let mut hasher = RspackHash::from(&options.output);

    self.content.hash(&mut hasher);
    if let Some(layer) = &self.layer {
      layer.hash(&mut hasher);
    }
    self.supports.hash(&mut hasher);
    self.media.hash(&mut hasher);
    self.source_map.hash(&mut hasher);

    hasher.digest(&options.output.hash_digest)
  }
}

#[async_trait::async_trait]
impl Module for CssModule {
  impl_module_meta_info!();

  fn readable_identifier(&self, context: &rspack_core::Context) -> std::borrow::Cow<str> {
    std::borrow::Cow::Owned(format!(
      "css {}{}{}{}{}",
      context.shorten(&self.identifier),
      if self.identifier_index > 0 {
        format!("({})", itoa!(self.identifier_index))
      } else {
        "".into()
      },
      if let Some(layer) = &self.layer {
        format!(" (layer {})", layer)
      } else {
        "".into()
      },
      if let Some(supports) = &self.supports
        && !supports.is_empty()
      {
        format!(" (supports {})", supports)
      } else {
        "".into()
      },
      if let Some(media) = &self.media
        && !media.is_empty()
      {
        format!(" (media {})", media)
      } else {
        "".into()
      }
    ))
  }

  fn name_for_condition(&self) -> Option<Box<str>> {
    self
      .identifier
      .split('!')
      .last()
      .map(|resource| resource.split('?').next().unwrap_or(resource).into())
  }

  fn size(&self, _source_type: Option<&SourceType>, _compilation: &Compilation) -> f64 {
    self.content.len() as f64
  }

  fn original_source(&self) -> Option<&dyn Source> {
    None
  }

  fn module_type(&self) -> &rspack_core::ModuleType {
    &MODULE_TYPE
  }

  fn source_types(&self) -> &[SourceType] {
    &*SOURCE_TYPE
  }

  fn need_id(&self) -> bool {
    false
  }

  async fn build(
    &mut self,
    build_context: BuildContext<'_>,
    _compilation: Option<&Compilation>,
  ) -> Result<BuildResult> {
    Ok(BuildResult {
      build_info: BuildInfo {
        hash: Some(self.compute_hash(build_context.compiler_options)),
        cacheable: self.cacheable,
        file_dependencies: self.file_dependencies.clone(),
        context_dependencies: self.context_dependencies.clone(),
        missing_dependencies: self.missing_dependencies.clone(),
        build_dependencies: self.build_dependencies.clone(),
        ..Default::default()
      },
      ..Default::default()
    })
  }

  // #[tracing::instrument("ExtractCssModule::code_generation", skip_all, fields(identifier = ?self.identifier()))]
  fn code_generation(
    &self,
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
    _concatenation_scope: Option<ConcatenationScope>,
  ) -> Result<CodeGenerationResult> {
    Ok(CodeGenerationResult::default())
  }

  fn get_diagnostics(&self) -> Vec<Diagnostic> {
    vec![]
  }

  fn update_hash(
    &self,
    hasher: &mut dyn std::hash::Hasher,
    compilation: &Compilation,
    runtime: Option<&RuntimeSpec>,
  ) -> Result<()> {
    module_update_hash(self, hasher, compilation, runtime);
    self
      .build_info
      .as_ref()
      .expect("should update_hash after build")
      .hash
      .dyn_hash(hasher);
    Ok(())
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

    Ok(ModuleFactoryResult::new_with_module(Box::new(
      CssModule::new(css_dep.clone()),
    )))
  }
}

impl_empty_diagnosable_trait!(CssModule);
impl_empty_diagnosable_trait!(CssModuleFactory);
