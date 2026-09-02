use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, Dependency, DependencyCategory, DependencyCodeGeneration, DependencyId,
  DependencyRange, DependencyTemplate, DependencyTemplateType, DependencyType, ExportsInfoArtifact,
  ModuleDependency, ReferencedExport, RuntimeSpec, TemplateContext, TemplateReplaceSource,
};
use rspack_util::atom::Atom;

use crate::{css_syntax::escape_identifier, utils::replace_css_module_id_placeholder};

#[cacheable]
#[derive(Debug, Clone)]
pub struct CssSelfReferenceLocalIdentReplacement {
  pub local_ident: String,
  pub range: DependencyRange,
}

#[cacheable]
#[derive(Debug)]
pub struct CssSelfReferenceLocalIdentDependency {
  id: DependencyId,
  names: Vec<String>,
  replaces: Vec<CssSelfReferenceLocalIdentReplacement>,
}

impl CssSelfReferenceLocalIdentDependency {
  pub fn new(names: Vec<String>, replaces: Vec<CssSelfReferenceLocalIdentReplacement>) -> Self {
    Self {
      id: DependencyId::new(),
      names,
      replaces,
    }
  }
}

#[cacheable_dyn]
impl Dependency for CssSelfReferenceLocalIdentDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn resource_identifier(&self) -> Option<&str> {
    Some("self")
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CssLocalIdent
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CssSelfReferenceLocalIdent
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::False
  }

  fn get_referenced_exports(
    &self,
    _module_graph: &rspack_core::ModuleGraph,
    _module_graph_cache: &rspack_core::ModuleGraphCacheArtifact,
    _exports_info_artifact: &ExportsInfoArtifact,
    _runtime: Option<&RuntimeSpec>,
  ) -> Vec<ReferencedExport> {
    self
      .names
      .iter()
      .map(|n| ReferencedExport::from(Atom::from(n.as_str())))
      .collect()
  }
}

#[cacheable_dyn]
impl ModuleDependency for CssSelfReferenceLocalIdentDependency {
  fn request(&self) -> &str {
    "self"
  }
}

#[cacheable_dyn]
impl DependencyCodeGeneration for CssSelfReferenceLocalIdentDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(CssSelfReferenceLocalIdentDependencyTemplate::template_type())
  }
}

impl AsContextDependency for CssSelfReferenceLocalIdentDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct CssSelfReferenceLocalIdentDependencyTemplate;

impl CssSelfReferenceLocalIdentDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::CssSelfReferenceLocalIdent)
  }
}

impl DependencyTemplate for CssSelfReferenceLocalIdentDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<CssSelfReferenceLocalIdentDependency>()
      .expect("CssSelfReferenceLocalIdentDependencyTemplate should be used for CssSelfReferenceLocalIdentDependency");

    for replace in &dep.replaces {
      let local_ident = replace_css_module_id_placeholder(
        &replace.local_ident,
        code_generatable_context.compilation,
        code_generatable_context.module,
      );
      source.replace(
        replace.range.start,
        replace.range.end,
        escape_identifier(&local_ident).into_owned(),
        None,
      );
    }
  }
}
