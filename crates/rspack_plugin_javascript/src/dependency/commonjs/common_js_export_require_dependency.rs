use rspack_core::{
  AsContextDependency, AsModuleDependency, Dependency, DependencyCategory, DependencyId,
  DependencyTemplate, DependencyType, ExportsOfExportsSpec, ExportsSpec, ModuleGraph,
  TemplateContext, TemplateReplaceSource,
};
use swc_core::atoms::Atom;

use super::ExportsBase;

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct CommonJsExportRequireDependency {
  id: DependencyId,
  range: (u32, u32),
  value_range: Option<(u32, u32)>,
  base: ExportsBase,
  names: Vec<Atom>,
  ids: Vec<Atom>,
  require_dep: Option<DependencyId>,
}

impl CommonJsExportRequireDependency {
  pub fn new(
    range: (u32, u32),
    value_range: Option<(u32, u32)>,
    base: ExportsBase,
    names: Vec<Atom>,
    require_dep: Option<DependencyId>,
  ) -> Self {
    Self {
      id: DependencyId::new(),
      range,
      value_range,
      base,
      names,
      // TODO: bailout rest opt for now
      ids: vec![],
      require_dep,
    }
  }
}

impl Dependency for CommonJsExportRequireDependency {
  fn dependency_debug_name(&self) -> &'static str {
    "CommonJsExportRequireDependency"
  }

  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CommonJS
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CjsExports
  }

  fn get_exports(&self, mg: &ModuleGraph) -> Option<ExportsSpec> {
    let related_require_dep = self.require_dep?;
    let con = mg.connection_by_dependency(&related_require_dep)?;
    Some(ExportsSpec {
      exports: ExportsOfExportsSpec::True,
      can_mangle: Some(false),
      from: if self.ids.is_empty() {
        Some(*con)
      } else {
        None
      },
      dependencies: Some(vec![con.module_identifier]),
      ..Default::default()
    })
  }
}

impl AsModuleDependency for CommonJsExportRequireDependency {}

impl DependencyTemplate for CommonJsExportRequireDependency {
  fn apply(
    &self,
    _source: &mut TemplateReplaceSource,
    _code_generatable_context: &mut TemplateContext,
  ) {
    // TODO:
  }
}

impl AsContextDependency for CommonJsExportRequireDependency {}
