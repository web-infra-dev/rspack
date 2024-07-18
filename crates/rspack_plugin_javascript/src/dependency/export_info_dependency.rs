use itertools::Itertools;
use rspack_core::{
  AsDependency, DependencyTemplate, ExportProvided, TemplateContext, TemplateReplaceSource,
  UsageState, UsedExports, UsedName,
};
use swc_core::ecma::atoms::Atom;

#[derive(Debug, Clone)]
pub struct ExportInfoDependency {
  start: u32,
  end: u32,
  export_name: Vec<Atom>,
  property: Atom,
}

impl ExportInfoDependency {
  pub fn new(start: u32, end: u32, export_name: Vec<Atom>, property: Atom) -> Self {
    Self {
      start,
      end,
      export_name,
      property,
    }
  }
}

impl DependencyTemplate for ExportInfoDependency {
  fn apply(&self, source: &mut TemplateReplaceSource, context: &mut TemplateContext) {
    let value = self.get_property(context);
    source.replace(
      self.start,
      self.end,
      value.unwrap_or("undefined".to_owned()).as_str(),
      None,
    );
  }

  fn dependency_id(&self) -> Option<rspack_core::DependencyId> {
    None
  }
}

impl ExportInfoDependency {
  fn get_property(&self, context: &TemplateContext) -> Option<String> {
    let TemplateContext {
      compilation,
      module,
      runtime,
      ..
    } = context;
    let export_name = &self.export_name;
    let prop = &self.property;
    let module_graph = compilation.get_module_graph();
    let module_identifier = module.identifier();

    if export_name.is_empty() && prop == "usedExports" {
      let used_exports = module_graph
        .get_exports_info(&module_identifier)
        .get_used_exports(&module_graph, *runtime);
      return Some(match used_exports {
        UsedExports::Null => "null".to_owned(),
        UsedExports::Bool(value) => value.to_string(),
        UsedExports::Vec(exports) => {
          format!(
            r#"[{}]"#,
            exports
              .iter()
              .map(|x| format!(r#""{x}""#))
              .collect_vec()
              .join(",")
          )
        }
      });
    }

    let exports_info = module_graph.get_exports_info(&module_identifier);

    match prop.to_string().as_str() {
      "canMangle" => {
        let can_mangle = if let Some(export_info) = exports_info
          .id
          .get_read_only_export_info_recursive(export_name, &module_graph)
        {
          export_info.can_mangle()
        } else {
          exports_info
            .other_exports_info
            .get_export_info(&module_graph)
            .can_mangle()
        };
        can_mangle.map(|v| v.to_string())
      }
      "used" => {
        let used =
          exports_info.get_used(UsedName::Vec(export_name.clone()), *runtime, &module_graph);
        Some((!matches!(used, UsageState::Unused)).to_string())
      }
      "useInfo" => {
        let used_state =
          exports_info.get_used(UsedName::Vec(export_name.clone()), *runtime, &module_graph);
        Some(
          (match used_state {
            UsageState::Used => "true",
            UsageState::OnlyPropertiesUsed => "true",
            UsageState::Unused => "false",
            UsageState::NoInfo => "undefined",
            UsageState::Unknown => "null",
          })
          .to_owned(),
        )
      }
      "provideInfo" => exports_info
        .id
        .is_export_provided(export_name, &module_graph)
        .map(|provided| {
          (match provided {
            ExportProvided::True => "true",
            ExportProvided::False => "false",
            ExportProvided::Null => "null",
          })
          .to_owned()
        }),
      _ => None,
    }
  }
}
impl AsDependency for ExportInfoDependency {}
