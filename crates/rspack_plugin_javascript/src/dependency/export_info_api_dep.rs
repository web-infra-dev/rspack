use rspack_core::{
  AsDependency, DependencyTemplate, TemplateContext, TemplateReplaceSource, UsageState,
};
use swc_core::ecma::atoms::Atom;

#[derive(Debug, Clone)]
pub struct ExportInfoApiDependency {
  start: u32,
  end: u32,
  // id: DependencyId,
  export_name: Vec<Atom>,
  property: Atom,
  // TODO: runtime_requirements
}

impl ExportInfoApiDependency {
  pub fn new(start: u32, end: u32, export_name: Vec<Atom>, property: Atom) -> Self {
    Self {
      start,
      end,
      // id: DependencyId::new(),
      export_name,
      property,
    }
  }
}

impl DependencyTemplate for ExportInfoApiDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let usage = matches!(
      self.get_property(code_generatable_context),
      Some(UsageState::Used)
    );
    source.replace(self.start, self.end, usage.to_string().as_ref(), None);
  }

  fn dependency_id(&self) -> Option<rspack_core::DependencyId> {
    None
  }
}

impl ExportInfoApiDependency {
  fn get_property(&self, context: &TemplateContext) -> Option<UsageState> {
    let TemplateContext {
      compilation,
      module,
      runtime,
      ..
    } = context;
    let export_name = &self.export_name;
    let prop = &self.property;
    // TODO: nested export_name, one level is enough for test
    if export_name.len() == 1 {
      let export_name = &export_name[0];
      match prop.to_string().as_str() {
        "used" => {
          let id = module.identifier();
          let mgm = compilation
            .get_module_graph()
            .module_graph_module_by_identifier(&id)?;
          let exports_info = compilation
            .get_module_graph()
            .get_exports_info_by_id(&mgm.exports);
          let info_id = exports_info.exports.get(export_name)?;
          let export_info = compilation
            .get_module_graph()
            .try_get_export_info_by_id(info_id)?;
          if compilation.options.is_new_tree_shaking() {
            Some(exports_info.get_used(
              rspack_core::UsedName::Str(export_name.clone()),
              *runtime,
              compilation.get_module_graph(),
            ))
          } else {
            Some(export_info.usage_state)
          }
        }
        _ => {
          // TODO: support other prop
          None
        }
      }
    } else {
      None
    }
  }
}
impl AsDependency for ExportInfoApiDependency {}
