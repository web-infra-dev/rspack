use rspack_core::{
  module_id, property_access, to_normal_comment, ExportsType, ExtendedReferencedExport,
  ModuleGraph, RuntimeGlobals, RuntimeSpec, UsedName,
};
use rspack_core::{AsContextDependency, Dependency, DependencyCategory, DependencyLocation};
use rspack_core::{DependencyId, DependencyTemplate};
use rspack_core::{DependencyType, ErrorSpan, ModuleDependency};
use rspack_core::{TemplateContext, TemplateReplaceSource};
use swc_core::atoms::Atom;

#[derive(Debug, Clone)]
pub struct CommonJsFullRequireDependency {
  id: DependencyId,
  request: String,
  names: Vec<Atom>,
  range: DependencyLocation,
  span: Option<ErrorSpan>,
  is_call: bool,
  optional: bool,
}

impl CommonJsFullRequireDependency {
  pub fn new(
    request: String,
    names: Vec<Atom>,
    range: DependencyLocation,
    span: Option<ErrorSpan>,
    is_call: bool,
    optional: bool,
  ) -> Self {
    Self {
      id: DependencyId::new(),
      request,
      names,
      range,
      span,
      is_call,
      optional,
    }
  }
}

impl Dependency for CommonJsFullRequireDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CommonJS
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CjsRequire
  }

  fn span(&self) -> Option<ErrorSpan> {
    self.span
  }

  fn dependency_debug_name(&self) -> &'static str {
    "CommonJsFullRequireDependency"
  }
}

impl ModuleDependency for CommonJsFullRequireDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn set_request(&mut self, request: String) {
    self.request = request;
  }

  fn get_optional(&self) -> bool {
    self.optional
  }

  fn get_referenced_exports(
    &self,
    module_graph: &ModuleGraph,
    _runtime: Option<&RuntimeSpec>,
  ) -> Vec<ExtendedReferencedExport> {
    if self.is_call
      && module_graph
        .module_graph_module_by_dependency_id(&self.id)
        .and_then(|mgm| module_graph.module_by_identifier(&mgm.module_identifier))
        .map(|m| m.get_exports_type_readonly(module_graph, false))
        .is_some_and(|t| !matches!(t, ExportsType::Namespace))
    {
      if self.names.is_empty() {
        return vec![ExtendedReferencedExport::Array(vec![])];
      } else {
        return vec![ExtendedReferencedExport::Array(
          self.names[0..self.names.len() - 1].to_vec(),
        )];
      }
    }
    vec![ExtendedReferencedExport::Array(self.names.clone())]
  }
}

impl DependencyTemplate for CommonJsFullRequireDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let TemplateContext {
      compilation,
      runtime,
      runtime_requirements,
      ..
    } = code_generatable_context;

    runtime_requirements.insert(RuntimeGlobals::REQUIRE);

    let mut require_expr = format!(
      r#"{}({})"#,
      RuntimeGlobals::REQUIRE,
      module_id(compilation, &self.id, &self.request, false)
    );

    if let Some(imported_module) = compilation
      .get_module_graph()
      .module_graph_module_by_dependency_id(&self.id)
    {
      let used = compilation
        .get_module_graph()
        .get_exports_info(&imported_module.module_identifier)
        .id
        .get_used_name(
          compilation.get_module_graph(),
          *runtime,
          UsedName::Vec(self.names.clone()),
        );

      if let Some(used) = used {
        let comment = to_normal_comment(&property_access(self.names.clone(), 0));
        require_expr = format!(
          "{}{}{}",
          require_expr,
          comment,
          property_access(
            match used {
              UsedName::Str(name) => vec![name].into_iter(),
              UsedName::Vec(names) => names.into_iter(),
            },
            0
          )
        );
      }
    }

    source.replace(self.range.start(), self.range.end(), &require_expr, None);
  }

  fn dependency_id(&self) -> Option<DependencyId> {
    Some(self.id)
  }
}

impl AsContextDependency for CommonJsFullRequireDependency {}
