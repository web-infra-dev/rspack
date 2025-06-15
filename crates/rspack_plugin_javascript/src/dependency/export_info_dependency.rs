use itertools::Itertools;
use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsPreset, AsVec},
};
use rspack_core::{
  DependencyCodeGeneration, DependencyTemplate, DependencyTemplateType, ExportInfoGetter,
  ExportProvided, ExportsInfoGetter, Inlinable, PrefetchExportsInfoMode, TemplateContext,
  TemplateReplaceSource, UsageState, UsedExports,
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
      let exports_info = module_graph
        .get_prefetched_exports_info(&module_identifier, PrefetchExportsInfoMode::AllExports);
      let used_exports = exports_info.get_used_exports(*runtime);
      return Some(match used_exports {
        UsedExports::Unknown => "null".to_owned(),
        UsedExports::UsedNamespace(value) => value.to_string(),
        UsedExports::UsedNames(exports) => {
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

    let exports_info = module_graph.get_prefetched_exports_info(
      &module_identifier,
      PrefetchExportsInfoMode::NamedNestedExports(export_name),
    );

    match prop.to_string().as_str() {
      "canMangle" => {
        let can_mangle = if let Some(export_info) =
          exports_info.get_read_only_export_info_recursive(export_name)
        {
          ExportInfoGetter::can_mangle(export_info)
        } else {
          ExportInfoGetter::can_mangle(exports_info.other_exports_info())
        };
        can_mangle.map(|v| v.to_string())
      }
      "inlinable" => {
        let inlinable = ExportInfoGetter::inlinable(
          if let Some(export_info) = exports_info.get_read_only_export_info_recursive(export_name) {
            export_info
          } else {
            exports_info.other_exports_info()
          },
        );
        Some(match inlinable {
          Inlinable::Inlined(inlined) => format!("inlined {}", inlined.render()),
          _ => "no inline".to_string(),
        })
      }
      "used" => {
        let used = ExportsInfoGetter::get_used(&exports_info, export_name, *runtime);
        Some((!matches!(used, UsageState::Unused)).to_string())
      }
      "useInfo" => {
        let used_state = ExportsInfoGetter::get_used(&exports_info, export_name, *runtime);
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
      "provideInfo" => {
        ExportsInfoGetter::is_export_provided(&exports_info, export_name).map(|provided| {
          (match provided {
            ExportProvided::Provided => "true",
            ExportProvided::NotProvided => "false",
            ExportProvided::Unknown => "null",
          })
          .to_owned()
        })
      }
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
