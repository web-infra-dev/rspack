use rspack_core::{
  module_raw, property_access, AsContextDependency, Dependency, DependencyCategory, DependencyId,
  DependencyTemplate, DependencyType, ErrorSpan, ExportsOfExportsSpec, ExportsSpec,
  ModuleDependency, ModuleGraph, RuntimeGlobals, TemplateContext, TemplateReplaceSource, UsedName,
};
use swc_core::atoms::Atom;

use super::ExportsBase;

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct CommonJsExportRequireDependency {
  id: DependencyId,
  request: String,
  optional: bool,
  span: Option<ErrorSpan>,
  range: (u32, u32),
  base: ExportsBase,
  names: Vec<Atom>,
  ids: Vec<Atom>,
  result_used: bool,
}

impl CommonJsExportRequireDependency {
  pub fn new(
    request: String,
    optional: bool,
    span: Option<ErrorSpan>,
    range: (u32, u32),
    base: ExportsBase,
    names: Vec<Atom>,
    result_used: bool,
  ) -> Self {
    Self {
      id: DependencyId::new(),
      request,
      optional,
      span,
      range,
      base,
      names,
      ids: vec![],
      result_used,
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
    &DependencyType::CjsExportRequire
  }

  fn get_exports(&self, mg: &ModuleGraph) -> Option<ExportsSpec> {
    let con = mg.connection_by_dependency(&self.id)?;
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

  fn get_ids(&self, mg: &ModuleGraph) -> Vec<Atom> {
    mg.get_dep_meta_if_existing(self.id)
      .map(|meta| meta.ids.clone())
      .unwrap_or_else(|| self.ids.clone())
  }
}

impl DependencyTemplate for CommonJsExportRequireDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let TemplateContext {
      compilation,
      module,
      runtime,
      runtime_requirements,
      ..
    } = code_generatable_context;

    let mg = &compilation.module_graph;

    let module = compilation
      .module_graph
      .module_by_identifier(&module.identifier())
      .expect("should have mgm");

    let exports_argument = module.get_exports_argument();
    let module_argument = module.get_module_argument();

    let used = mg.get_exports_info(&module.identifier()).id.get_used_name(
      mg,
      *runtime,
      UsedName::Vec(self.names.clone()),
    );

    let base = if self.base.is_exports() {
      runtime_requirements.insert(RuntimeGlobals::EXPORTS);
      exports_argument.to_string()
    } else if self.base.is_module_exports() {
      runtime_requirements.insert(RuntimeGlobals::MODULE);
      format!("{}.exports", module_argument)
    } else if self.base.is_this() {
      runtime_requirements.insert(RuntimeGlobals::THIS_AS_EXPORTS);
      "this".to_string()
    } else {
      unreachable!()
    };

    let mut require_expr = module_raw(
      compilation,
      runtime_requirements,
      &self.id,
      &self.request,
      false,
    );

    if let Some(imported_module) = mg.get_module(&self.id) {
      let ids = self.get_ids(mg);
      if let Some(used_imported) = mg
        .get_exports_info(&imported_module.identifier())
        .id
        .get_used_name(mg, *runtime, UsedName::Vec(ids))
      {
        require_expr = format!(
          "{}{}",
          require_expr,
          property_access(
            match used_imported {
              UsedName::Str(name) => vec![name].into_iter(),
              UsedName::Vec(names) => names.into_iter(),
            },
            0
          )
        )
      }
    }

    if self.base.is_expression() {
      let expr = match used {
        Some(used) => format!(
          "{base}{} = {require_expr}",
          property_access(
            match used {
              UsedName::Str(name) => vec![name].into_iter(),
              UsedName::Vec(names) => names.into_iter(),
            },
            0
          )
        ),
        None => format!("/* unused reexport */ {}", require_expr),
      };
      source.replace(self.range.0, self.range.1, expr.as_str(), None)
    } else if self.base.is_define_property() {
      panic!("TODO")
    } else {
      panic!("Unexpected type");
    }
  }

  fn dependency_id(&self) -> Option<DependencyId> {
    Some(self.id)
  }
}

impl ModuleDependency for CommonJsExportRequireDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn get_optional(&self) -> bool {
    self.optional
  }

  fn set_request(&mut self, request: String) {
    self.request = request;
  }
}
impl AsContextDependency for CommonJsExportRequireDependency {}
