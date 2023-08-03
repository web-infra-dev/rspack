use rspack_core::{DependencyTemplate, TemplateContext, TemplateReplaceSource, UsageState};
use swc_core::ecma::atoms::JsWord;

#[derive(Debug, Clone)]
pub struct ExportInfoApiDependency {
  start: u32,
  end: u32,
  // id: DependencyId,
  export_name: Vec<JsWord>,
  property: JsWord,
  // TODO: runtime_requirements
}

impl ExportInfoApiDependency {
  pub fn new(start: u32, end: u32, export_name: Vec<JsWord>, property: JsWord) -> Self {
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
}

impl ExportInfoApiDependency {
  fn get_property(&self, context: &TemplateContext) -> Option<UsageState> {
    let TemplateContext {
      compilation,
      module,
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
            .module_graph
            .module_graph_module_by_identifier(&id)?;
          let info = mgm.exports.exports.get(export_name)?;
          Some(info.usage_state)
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
