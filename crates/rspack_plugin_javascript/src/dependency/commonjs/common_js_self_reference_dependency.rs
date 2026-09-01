use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsPreset, AsVec},
};
use rspack_core::{
  AsContextDependency, Dependency, DependencyCategory, DependencyCodeGeneration, DependencyId,
  DependencyRange, DependencyTemplate, DependencyTemplateType, DependencyType, ExportsInfoArtifact,
  ModuleDependency, ModuleGraph, ModuleGraphCacheArtifact, ReferencedExport, RuntimeGlobals,
  RuntimeSpec, TemplateContext, TemplateReplaceSource, UsedName, property_access_with_optional,
};
use rspack_hash::{RspackHash, RspackHasher};
use swc_atoms::Atom;

use super::{ExportsBase, common_js_dependency_helpers::is_worker_entry_this};

#[cacheable]
#[derive(Debug)]
pub struct CommonJsSelfReferenceDependency {
  id: DependencyId,
  range: DependencyRange,
  base: ExportsBase,
  #[cacheable(with=AsVec<AsPreset>)]
  names: Vec<Atom>,
  names_optionals: Vec<bool>,
  is_call: bool,
}

impl CommonJsSelfReferenceDependency {
  pub fn new(
    range: DependencyRange,
    base: ExportsBase,
    names: Vec<Atom>,
    names_optionals: Vec<bool>,
    is_call: bool,
  ) -> Self {
    Self {
      id: DependencyId::new(),
      range,
      base,
      names,
      names_optionals,
      is_call,
    }
  }
}

#[cacheable_dyn]
impl Dependency for CommonJsSelfReferenceDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn range(&self) -> Option<DependencyRange> {
    Some(self.range)
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

  fn get_referenced_exports(
    &self,
    _module_graph: &ModuleGraph,
    _module_graph_cache: &ModuleGraphCacheArtifact,
    _exports_info_artifact: &ExportsInfoArtifact,
    _runtime: Option<&RuntimeSpec>,
  ) -> Vec<ReferencedExport> {
    if self.is_call {
      if self.names.is_empty() {
        vec![ReferencedExport::default()]
      } else {
        vec![ReferencedExport::from(&self.names[0..self.names.len() - 1])]
      }
    } else {
      vec![ReferencedExport::from(self.names.as_slice())]
    }
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }
}

#[cacheable_dyn]
impl ModuleDependency for CommonJsSelfReferenceDependency {
  fn request(&self) -> &str {
    "self"
  }
}

impl AsContextDependency for CommonJsSelfReferenceDependency {}

#[cacheable_dyn]
impl DependencyCodeGeneration for CommonJsSelfReferenceDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(CommonJsSelfReferenceDependencyTemplate::template_type())
  }

  fn update_hash(
    &self,
    hasher: &mut RspackHasher,
    compilation: &rspack_core::Compilation,
    runtime: Option<&RuntimeSpec>,
  ) {
    if !self.base.is_this() {
      return;
    }
    let module_graph = compilation.get_module_graph();
    let worker_global = module_graph
      .get_parent_module(&self.id)
      .is_some_and(|module| is_worker_entry_this(compilation, *module, runtime));
    if worker_global {
      "worker global".hash(hasher);
    } else {
      "exports".hash(hasher);
    }
  }
}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct CommonJsSelfReferenceDependencyTemplate;

impl CommonJsSelfReferenceDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::CjsSelfReference)
  }
}

impl DependencyTemplate for CommonJsSelfReferenceDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<CommonJsSelfReferenceDependency>()
      .expect("CommonJsSelfReferenceDependencyTemplate should only be used for CommonJsSelfReferenceDependency");

    let TemplateContext {
      compilation,
      module,
      runtime,
      runtime_template,
      ..
    } = code_generatable_context;
    let module_graph = compilation.get_module_graph();
    let module = module_graph
      .module_by_identifier(&module.identifier())
      .expect("should have mgm");

    let is_worker_entry =
      dep.base.is_this() && is_worker_entry_this(compilation, module.identifier(), *runtime);

    let used = if is_worker_entry || dep.names.is_empty() {
      UsedName::Normal(dep.names.clone())
    } else {
      let exports_info = compilation
        .exports_info_artifact
        .get_exports_info_data(&module.identifier());
      exports_info
        .get_used_name(&compilation.exports_info_artifact, *runtime, &dep.names)
        .unwrap_or_else(|| UsedName::Normal(dep.names.clone()))
    };

    let exports_argument = module.get_exports_argument();
    let module_argument = module.get_module_argument();

    let base = if dep.base.is_exports() {
      runtime_template.render_exports_argument(exports_argument)
    } else if dep.base.is_module_exports() {
      format!(
        "{}.exports",
        runtime_template.render_module_argument(module_argument)
      )
    } else if is_worker_entry {
      runtime_template.render_runtime_globals(&RuntimeGlobals::GLOBAL)
    } else if dep.base.is_this() {
      runtime_template.render_this_exports()
    } else {
      unreachable!();
    };

    source.replace(
      dep.range.start,
      dep.range.end,
      match used {
        UsedName::Normal(used) => format!(
          "{}{}",
          base,
          property_access_with_optional(used, &dep.names_optionals, 0)
        ),
        // Export a inlinable const from cjs is not possible for now, so self reference a inlinable
        // const is also not possible for now, but we compat it here
        UsedName::Inlined(inlined) => inlined.render(""),
      },
      None,
    )
  }
}
