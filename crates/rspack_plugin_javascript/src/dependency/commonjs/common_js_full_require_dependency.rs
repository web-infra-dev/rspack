use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsPreset, AsVec},
};
use rspack_core::{
  AsContextDependency, Dependency, DependencyCategory, DependencyCodeGeneration, DependencyId,
  DependencyLocation, DependencyRange, DependencyTemplate, DependencyTemplateType, DependencyType,
  ExportsInfoArtifact, ModuleDependency, ModuleGraph, ModuleGraphCacheArtifact, ReferencedExport,
  RuntimeGlobals, RuntimeSpec, TemplateContext, TemplateReplaceSource, UsedName, property_access,
  should_reference_call_receiver, to_normal_comment,
};

use crate::Atom;

#[cacheable]
#[derive(Debug)]
pub struct CommonJsFullRequireDependency {
  id: DependencyId,
  request: String,
  #[cacheable(with=AsVec<AsPreset>)]
  names: Vec<Atom>,
  range: DependencyRange,
  is_call: bool,
  namespace_object_as_context: bool,
  optional: bool,
  asi_safe: bool,
  loc: Option<DependencyLocation>,
}

impl CommonJsFullRequireDependency {
  #[allow(clippy::too_many_arguments)]
  #[allow(clippy::fn_params_excessive_bools)]
  pub fn new(
    request: String,
    names: Vec<Atom>,
    range: DependencyRange,
    loc: Option<DependencyLocation>,
    is_call: bool,
    namespace_object_as_context: bool,
    optional: bool,
    asi_safe: bool,
  ) -> Self {
    Self {
      id: DependencyId::new(),
      request,
      names,
      range,
      is_call,
      namespace_object_as_context,
      optional,
      asi_safe,
      loc,
    }
  }

  pub fn names(&self) -> &[Atom] {
    &self.names
  }

  pub fn range(&self) -> DependencyRange {
    self.range
  }

  pub fn asi_safe(&self) -> bool {
    self.asi_safe
  }
}

#[cacheable_dyn]
impl Dependency for CommonJsFullRequireDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CommonJS
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CjsFullRequire
  }

  fn loc(&self) -> Option<DependencyLocation> {
    self.loc.clone()
  }

  fn range(&self) -> Option<DependencyRange> {
    Some(self.range)
  }

  fn get_referenced_exports(
    &self,
    module_graph: &ModuleGraph,
    module_graph_cache: &ModuleGraphCacheArtifact,
    exports_info_artifact: &ExportsInfoArtifact,
    _runtime: Option<&RuntimeSpec>,
  ) -> Vec<ReferencedExport> {
    let mut names = self.names.as_slice();
    if self.is_call {
      let module = module_graph
        .get_module_by_dependency_id(&self.id)
        .expect("should have module");
      let exports_type = module.get_exports_type(
        module_graph,
        module_graph_cache,
        exports_info_artifact,
        false,
      );
      if self.namespace_object_as_context
        || should_reference_call_receiver(
          names,
          exports_type,
          exports_info_artifact,
          exports_info_artifact.get_exports_info_data(&module.identifier()),
        )
      {
        names = &names[..names.len().saturating_sub(1)];
      }
    }
    vec![ReferencedExport::from(names)]
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }
}

#[cacheable_dyn]
impl ModuleDependency for CommonJsFullRequireDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn get_optional(&self) -> bool {
    self.optional
  }
}

#[cacheable_dyn]
impl DependencyCodeGeneration for CommonJsFullRequireDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(CommonJsFullRequireDependencyTemplate::template_type())
  }
}

impl AsContextDependency for CommonJsFullRequireDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct CommonJsFullRequireDependencyTemplate;

impl CommonJsFullRequireDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::CjsFullRequire)
  }
}

impl DependencyTemplate for CommonJsFullRequireDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<CommonJsFullRequireDependency>()
      .expect("CommonJsFullRequireDependencyTemplate should only be used for CommonJsFullRequireDependency");

    let TemplateContext {
      compilation,
      runtime,
      runtime_template,
      ..
    } = code_generatable_context;
    let module_graph = compilation.get_module_graph();

    let require_expr = if let Some(imported_module) =
      module_graph.module_graph_module_by_dependency_id(&dep.id)
      && let Some(used) = {
        let exports_info = compilation
          .exports_info_artifact
          .get_exports_info_data(&imported_module.module_identifier);
        exports_info.get_used_name(&compilation.exports_info_artifact, *runtime, &dep.names)
      } {
      let mut require_expr = match used {
        UsedName::Normal(used) => {
          format!(
            "{}({}){}{}",
            runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE),
            runtime_template.module_id(compilation, &dep.id, &dep.request, false),
            to_normal_comment(&property_access(&dep.names, 0)),
            property_access(used, 0)
          )
        }
        UsedName::Inlined(inlined) => inlined.render(&to_normal_comment(&format!(
          "inlined export {}",
          property_access(&dep.names, 0)
        ))),
      };
      if dep.asi_safe {
        require_expr = format!("({require_expr})");
      }
      require_expr
    } else {
      format!(
        r#"{}({})"#,
        runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE),
        runtime_template.module_id(compilation, &dep.id, &dep.request, false)
      )
    };

    source.replace(dep.range.start, dep.range.end, require_expr, None);
  }
}
