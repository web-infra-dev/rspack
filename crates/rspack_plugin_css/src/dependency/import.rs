use std::fmt::Display;

use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, Compilation, Dependency, DependencyCategory, DependencyId, DependencyRange,
  DependencyTemplate, DependencyType, FactorizeInfo, ModuleDependency, RuntimeSpec,
  TemplateContext, TemplateReplaceSource,
};

#[cacheable]
#[derive(Debug, Clone)]
pub struct CssImportDependency {
  id: DependencyId,
  request: String,
  range: DependencyRange,
  media: Option<String>,
  supports: Option<String>,
  layer: Option<CssLayer>,
  factorize_info: FactorizeInfo,
}

#[cacheable]
#[derive(Debug, Clone)]
pub enum CssLayer {
  Anonymous,
  Named(String),
}

impl CssImportDependency {
  pub fn new(
    request: String,
    range: DependencyRange,
    media: Option<String>,
    supports: Option<String>,
    layer: Option<CssLayer>,
  ) -> Self {
    Self {
      id: DependencyId::new(),
      request,
      range,
      media,
      supports,
      layer,
      factorize_info: Default::default(),
    }
  }

  pub fn media(&self) -> Option<&str> {
    self.media.as_deref()
  }

  pub fn supports(&self) -> Option<&str> {
    self.supports.as_deref()
  }

  pub fn layer(&self) -> Option<&CssLayer> {
    self.layer.as_ref()
  }
}

#[cacheable_dyn]
impl Dependency for CssImportDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CssImport
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CssImport
  }

  fn range(&self) -> Option<&DependencyRange> {
    Some(&self.range)
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }
}

#[cacheable_dyn]
impl ModuleDependency for CssImportDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn set_request(&mut self, request: String) {
    self.request = request;
  }

  fn factorize_info(&self) -> &FactorizeInfo {
    &self.factorize_info
  }

  fn factorize_info_mut(&mut self) -> &mut FactorizeInfo {
    &mut self.factorize_info
  }
}

#[derive(Clone)]
pub struct CssMedia(pub String);

#[derive(Clone)]
pub struct CssSupports(pub String);

impl Display for CssMedia {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.0.fmt(f)
  }
}

impl Display for CssSupports {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.0.fmt(f)
  }
}

#[cacheable_dyn]
impl DependencyTemplate for CssImportDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    _code_generatable_context: &mut TemplateContext,
  ) {
    source.replace(self.range.start, self.range.end, "", None);
  }

  fn dependency_id(&self) -> Option<DependencyId> {
    Some(self.id)
  }

  fn update_hash(
    &self,
    _hasher: &mut dyn std::hash::Hasher,
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) {
  }
}

impl AsContextDependency for CssImportDependency {}
