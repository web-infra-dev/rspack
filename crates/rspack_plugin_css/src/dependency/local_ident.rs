use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, AsModuleDependency, Compilation, Dependency, DependencyCategory,
  DependencyCodeGeneration, DependencyId, DependencyTemplate, DependencyTemplateType,
  DependencyType, ExportNameOrSpec, ExportSpec, ExportsOfExportsSpec, ExportsSpec, RuntimeSpec,
  TemplateContext, TemplateReplaceSource,
};
use rspack_util::ext::DynHash;

use crate::utils::escape_css;

#[cacheable]
#[derive(Debug, Clone)]
pub struct CssLocalIdentDependency {
  id: DependencyId,
  local_ident: String,
  convention_names: Vec<String>,
  start: u32,
  end: u32,
}

impl CssLocalIdentDependency {
  pub fn new(local_ident: String, convention_names: Vec<String>, start: u32, end: u32) -> Self {
    Self {
      id: DependencyId::new(),
      local_ident,
      convention_names,
      start,
      end,
    }
  }
}

#[cacheable_dyn]
impl Dependency for CssLocalIdentDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CssLocalIdent
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CssLocalIdent
  }

  fn get_exports(
    &self,
    _mg: &rspack_core::ModuleGraph,
    _module_graph_cache: &rspack_core::ModuleGraphCacheArtifact,
  ) -> Option<ExportsSpec> {
    Some(ExportsSpec {
      exports: ExportsOfExportsSpec::Names(
        self
          .convention_names
          .iter()
          .map(|name| {
            ExportNameOrSpec::ExportSpec(ExportSpec {
              name: name.as_str().into(),
              can_mangle: Some(true),
              ..Default::default()
            })
          })
          .collect(),
      ),
      ..Default::default()
    })
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::False
  }
}

#[cacheable_dyn]
impl DependencyCodeGeneration for CssLocalIdentDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(CssLocalIdentDependencyTemplate::template_type())
  }

  fn update_hash(
    &self,
    hasher: &mut dyn std::hash::Hasher,
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) {
    self.local_ident.dyn_hash(hasher);
  }
}

impl AsContextDependency for CssLocalIdentDependency {}
impl AsModuleDependency for CssLocalIdentDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct CssLocalIdentDependencyTemplate;

impl CssLocalIdentDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::CssLocalIdent)
  }
}

impl DependencyTemplate for CssLocalIdentDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    _code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<CssLocalIdentDependency>()
      .expect("CssLocalIdentDependencyTemplate should be used for CssLocalIdentDependency");

    source.replace(dep.start, dep.end, &escape_css(&dep.local_ident), None);
  }
}
