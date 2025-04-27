use itertools::Itertools;
use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsPreset, AsVec},
};
use rspack_core::{
  DependencyCodeGeneration, DependencyTemplate, DependencyTemplateType, ExportProvided,
  TemplateContext, TemplateReplaceSource, UsageState, UsedExports, UsedName,
};
use swc_core::ecma::atoms::Atom;

#[cacheable]
#[derive(Debug, Clone)]
pub struct ExportInfoDependency {
  start: u32,
  end: u32,
  #[cacheable(with=AsVec<AsPreset>)]
  export_name: Vec<Atom>,
  #[cacheable(with=AsPreset)]
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

#[cacheable_dyn]
impl DependencyCodeGeneration for ExportInfoDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(ExportInfoDependencyTemplate::template_type())
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
        let can_mangle = if let Some(export_info) =
          exports_info.get_read_only_export_info_recursive(&module_graph, export_name)
        {
          export_info.can_mangle(&module_graph)
        } else {
          exports_info
            .other_exports_info(&module_graph)
            .can_mangle(&module_graph)
        };
        can_mangle.map(|v| v.to_string())
      }
      "used" => {
        let used =
          exports_info.get_used(&module_graph, UsedName::Vec(export_name.clone()), *runtime);
        Some((!matches!(used, UsageState::Unused)).to_string())
      }
      "useInfo" => {
        let used_state =
          exports_info.get_used(&module_graph, UsedName::Vec(export_name.clone()), *runtime);
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
        .is_export_provided(&module_graph, export_name)
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

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct ExportInfoDependencyTemplate;

impl ExportInfoDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Custom("ExportInfoDependency")
  }
}

impl DependencyTemplate for ExportInfoDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<ExportInfoDependency>()
      .expect("ExportInfoDependencyTemplate should be used for ExportInfoDependency");

    let value = dep.get_property(code_generatable_context);
    source.replace(
      dep.start,
      dep.end,
      value.unwrap_or("undefined".to_owned()).as_str(),
      None,
    );
  }
}
