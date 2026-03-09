use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsPreset, AsVec},
};
use rspack_core::{
  AsContextDependency, Dependency, DependencyCategory, DependencyCodeGeneration, DependencyId,
  DependencyLocation, DependencyRange, DependencyTemplate, DependencyTemplateType, DependencyType,
  ExportsInfoArtifact, ExportsInfoGetter, ExportsType, ExtendedReferencedExport, FactorizeInfo,
  GetUsedNameParam, ModuleDependency, ModuleGraph, ModuleGraphCacheArtifact,
  PrefetchExportsInfoMode, RuntimeGlobals, RuntimeSpec, TemplateContext, TemplateReplaceSource,
  UsedName, create_exports_object_referenced, get_exports_type, property_access, to_normal_comment,
};
use swc_core::atoms::Atom;

#[cacheable]
#[derive(Debug, Clone)]
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
  factorize_info: FactorizeInfo,
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
      factorize_info: Default::default(),
    }
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
  ) -> Vec<ExtendedReferencedExport> {
    let mut namespace_object_as_context = self.namespace_object_as_context;
    let parent_module = module_graph
      .get_parent_module(&self.id)
      .expect("should have parent module");
    let exports_type = get_exports_type(
      module_graph,
      module_graph_cache,
      exports_info_artifact,
      &self.id,
      parent_module,
    );

    // Force enable namespace object as context for DefaultOnly and DefaultWithNamed
    // because it's more common in cjs and json
    if matches!(
      exports_type,
      ExportsType::DefaultOnly | ExportsType::DefaultWithNamed
    ) {
      namespace_object_as_context = true;
    }

    if namespace_object_as_context && self.is_call {
      if self.names.is_empty() {
        return create_exports_object_referenced();
      }
      return vec![ExtendedReferencedExport::Array(
        self.names[0..self.names.len().saturating_sub(1)].to_vec(),
      )];
    }
    vec![ExtendedReferencedExport::Array(self.names.clone())]
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

  fn factorize_info(&self) -> &FactorizeInfo {
    &self.factorize_info
  }

  fn factorize_info_mut(&mut self) -> &mut FactorizeInfo {
    &mut self.factorize_info
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
      && let used = {
        if dep.names.is_empty() {
          let exports_info_used = compilation
            .exports_info_artifact
            .get_prefetched_exports_info_used(&imported_module.module_identifier, *runtime);
          ExportsInfoGetter::get_used_name(
            GetUsedNameParam::WithoutNames(&exports_info_used),
            *runtime,
            &dep.names,
          )
        } else {
          let exports_info = compilation
            .exports_info_artifact
            .get_prefetched_exports_info(
              &imported_module.module_identifier,
              PrefetchExportsInfoMode::Nested(&dep.names),
            );
          ExportsInfoGetter::get_used_name(
            GetUsedNameParam::WithNames(&exports_info),
            *runtime,
            &dep.names,
          )
        }
      }
      && let Some(used) = used
    {
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

    source.replace(dep.range.start, dep.range.end, &require_expr, None);
  }
}
