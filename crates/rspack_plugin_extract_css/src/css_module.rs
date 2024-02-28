use std::hash::Hash;
use std::path::PathBuf;

use once_cell::sync::Lazy;
use rspack_core::rspack_sources::Source;
use rspack_core::{
  impl_build_info_meta, impl_source_map_config, AsyncDependenciesBlockIdentifier, BuildContext,
  BuildInfo, BuildMeta, BuildResult, CodeGenerationResult, Compilation, CompilerOptions,
  ConcatenationScope, DependenciesBlock, DependencyId, DependencyType, Module, ModuleFactory,
  ModuleFactoryCreateData, ModuleFactoryResult, RuntimeSpec, SourceType,
};
use rspack_error::Result;
use rspack_error::{impl_empty_diagnosable_trait, Diagnostic};
use rspack_hash::{RspackHash, RspackHashDigest};
use rspack_identifier::{Identifiable, Identifier};
use rustc_hash::FxHashSet;

use crate::css_dependency::CssDependency;
use crate::plugin::{MODULE_TYPE, SOURCE_TYPE};

pub(crate) static DEPENDENCY_TYPE: Lazy<DependencyType> =
  Lazy::new(|| DependencyType::Custom("mini-extract-dep".into()));

#[impl_source_map_config]
#[derive(Debug)]
pub(crate) struct CssModule {
  pub(crate) identifier: String,
  pub(crate) content: String,
  pub(crate) context: String,
  pub(crate) media: String,
  pub(crate) supports: String,
  pub(crate) source_map: String,
  pub(crate) identifier_index: u32,

  pub build_info: Option<BuildInfo>,
  pub build_meta: Option<BuildMeta>,

  blocks: Vec<AsyncDependenciesBlockIdentifier>,
  dependencies: Vec<DependencyId>,

  identifier__: Identifier,
  filepath: PathBuf,
}

impl Hash for CssModule {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.identifier.hash(state);
  }
}

impl PartialEq for CssModule {
  fn eq(&self, other: &Self) -> bool {
    self.identifier == other.identifier
  }
}

impl Eq for CssModule {}

impl CssModule {
  pub fn new(dep: CssDependency) -> Self {
    let identifier__ = format!(
      "css|{}|{}|{}|{}}}",
      dep.identifier, dep.identifier_index, dep.supports, dep.media,
    )
    .into();

    Self {
      identifier: dep.identifier,
      content: dep.content,
      context: dep.context,
      media: dep.media,
      supports: dep.supports,
      source_map: dep.source_map,
      identifier_index: dep.identifier_index,
      blocks: vec![],
      dependencies: vec![],
      build_info: None,
      build_meta: None,
      source_map_kind: rspack_util::source_map::SourceMapKind::None,
      identifier__,
      filepath: dep.filepath,
    }
  }

  fn compute_hash(&self, options: &CompilerOptions) -> RspackHashDigest {
    let mut hasher = RspackHash::from(&options.output);

    self.content.hash(&mut hasher);
    self.supports.hash(&mut hasher);
    self.media.hash(&mut hasher);
    self.context.hash(&mut hasher);

    hasher.digest(&options.output.hash_digest)
  }
}

#[async_trait::async_trait]
impl Module for CssModule {
  impl_build_info_meta!();

  fn readable_identifier(&self, context: &rspack_core::Context) -> std::borrow::Cow<str> {
    std::borrow::Cow::Owned(format!(
      "css {}{}{}{}",
      context.shorten(&self.identifier),
      if self.identifier_index > 0 {
        format!("({})", self.identifier_index)
      } else {
        "".into()
      },
      if self.supports.is_empty() {
        "".into()
      } else {
        format!(" (supports {})", self.supports)
      },
      if self.media.is_empty() {
        "".into()
      } else {
        format!(" (media {})", self.media)
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

  fn size(&self, _source_type: &SourceType) -> f64 {
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

  async fn build(
    &mut self,
    build_context: BuildContext<'_>,
    _compilation: Option<&Compilation>,
  ) -> Result<BuildResult> {
    let mut file_deps = FxHashSet::default();
    file_deps.insert(self.filepath.clone());

    Ok(BuildResult {
      build_info: BuildInfo {
        hash: Some(self.compute_hash(build_context.compiler_options)),
        file_dependencies: file_deps,
        ..Default::default()
      },
      ..Default::default()
    })
  }

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
}

impl Identifiable for CssModule {
  fn identifier(&self) -> rspack_identifier::Identifier {
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
    let css_dep = data
      .dependency
      .downcast_ref::<CssDependency>()
      .expect("unreachable");

    Ok(ModuleFactoryResult::new_with_module(Box::new(
      CssModule::new(css_dep.clone()),
    )))
  }
}

impl_empty_diagnosable_trait!(CssModule);
impl_empty_diagnosable_trait!(CssModuleFactory);
