use rspack_core::{
  property_access, AsContextDependency, Dependency, DependencyCategory, DependencyId,
  DependencyTemplate, DependencyType, ExtendedReferencedExport, ModuleDependency, ModuleGraph,
  RuntimeGlobals, RuntimeSpec, TemplateContext, TemplateReplaceSource, UsedName,
};
use swc_core::atoms::Atom;

use super::ExportsBase;

#[derive(Debug, Clone)]
pub struct CommonJsSelfReferenceDependency {
  id: DependencyId,
  range: (u32, u32),
  base: ExportsBase,
  names: Vec<Atom>,
  is_call: bool,
}

impl CommonJsSelfReferenceDependency {
  pub fn new(range: (u32, u32), base: ExportsBase, names: Vec<Atom>, is_call: bool) -> Self {
    Self {
      id: DependencyId::new(),
      range,
      base,
      names,
      is_call,
    }
  }
}

impl Dependency for CommonJsSelfReferenceDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CommonJS
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CjsSelfReference
  }

  fn resource_identifier(&self) -> Option<&str> {
    Some("self")
  }
}

impl ModuleDependency for CommonJsSelfReferenceDependency {
  fn get_referenced_exports(
    &self,
    _module_graph: &ModuleGraph,
    _runtime: Option<&RuntimeSpec>,
  ) -> Vec<ExtendedReferencedExport> {
    if self.is_call {
      if self.names.is_empty() {
        vec![ExtendedReferencedExport::Array(vec![])]
      } else {
        vec![ExtendedReferencedExport::Array(
          self.names[0..self.names.len() - 1].to_vec(),
        )]
      }
    } else {
      vec![ExtendedReferencedExport::Array(self.names.clone())]
    }
  }

  fn request(&self) -> &str {
    "self"
  }
}

impl AsContextDependency for CommonJsSelfReferenceDependency {}

impl DependencyTemplate for CommonJsSelfReferenceDependency {
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
    let module_graph = compilation.get_module_graph();
    let module = module_graph
      .module_by_identifier(&module.identifier())
      .expect("should have mgm");

    let used = if self.names.is_empty() {
      module_graph
        .get_exports_info(&module.identifier())
        .id
        .get_used_name(&module_graph, *runtime, UsedName::Vec(self.names.clone()))
        .unwrap_or_else(|| UsedName::Vec(self.names.clone()))
    } else {
      UsedName::Vec(self.names.clone())
    };

    let exports_argument = module.get_exports_argument();
    let module_argument = module.get_module_argument();

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
      unreachable!();
    };

    source.replace(
      self.range.0,
      self.range.1,
      &format!(
        "{}{}",
        base,
        property_access(
          match used {
            UsedName::Str(name) => vec![name].into_iter(),
            UsedName::Vec(names) => names.into_iter(),
          },
          0
        )
      ),
      None,
    )
  }

  fn dependency_id(&self) -> Option<DependencyId> {
    Some(self.id)
  }
}
