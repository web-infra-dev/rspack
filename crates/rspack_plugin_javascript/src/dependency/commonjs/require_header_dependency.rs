use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, AsModuleDependency, Dependency, DependencyCodeGeneration, DependencyId,
  DependencyLocation, DependencyRange, DependencyTemplate, DependencyTemplateType, RuntimeGlobals,
  TemplateContext, TemplateReplaceSource,
};

#[cacheable]
#[derive(Debug, Clone, Copy)]
enum RequireHeaderDependencyMode {
  Replace,
  GuardPreInitialization,
  EvaluateCreateRequireArgs,
  EvaluateCreateRequireCacheArgs,
}

#[cacheable]
#[derive(Debug, Clone)]
pub struct RequireHeaderDependency {
  id: DependencyId,
  range: DependencyRange,
  loc: Option<DependencyLocation>,
  mode: RequireHeaderDependencyMode,
}

impl RequireHeaderDependency {
  pub fn new(range: DependencyRange, loc: Option<DependencyLocation>) -> Self {
    Self {
      id: DependencyId::new(),
      range,
      loc,
      mode: RequireHeaderDependencyMode::Replace,
    }
  }

  pub fn guard_pre_initialization(range: DependencyRange, loc: Option<DependencyLocation>) -> Self {
    Self {
      mode: RequireHeaderDependencyMode::GuardPreInitialization,
      ..Self::new(range, loc)
    }
  }

  pub fn evaluate_create_require_args(
    range: DependencyRange,
    loc: Option<DependencyLocation>,
  ) -> Self {
    Self {
      mode: RequireHeaderDependencyMode::EvaluateCreateRequireArgs,
      ..Self::new(range, loc)
    }
  }

  pub fn evaluate_create_require_cache_args(
    range: DependencyRange,
    loc: Option<DependencyLocation>,
  ) -> Self {
    Self {
      mode: RequireHeaderDependencyMode::EvaluateCreateRequireCacheArgs,
      ..Self::new(range, loc)
    }
  }
}

#[cacheable_dyn]
impl Dependency for RequireHeaderDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn loc(&self) -> Option<DependencyLocation> {
    self.loc.clone()
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::False
  }
}

impl AsModuleDependency for RequireHeaderDependency {}
impl AsContextDependency for RequireHeaderDependency {}

#[cacheable_dyn]
impl DependencyCodeGeneration for RequireHeaderDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(RequireHeaderDependencyTemplate::template_type())
  }
}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct RequireHeaderDependencyTemplate;

impl RequireHeaderDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Custom("RequireHeaderDependency")
  }
}

impl DependencyTemplate for RequireHeaderDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<RequireHeaderDependency>()
      .expect("RequireHeaderDependencyTemplate should only be used for RequireHeaderDependency");

    let runtime_global = match dep.mode {
      RequireHeaderDependencyMode::EvaluateCreateRequireCacheArgs => RuntimeGlobals::MODULE_CACHE,
      _ => RuntimeGlobals::REQUIRE,
    };
    let runtime = code_generatable_context
      .runtime_template
      .render_runtime_globals(&runtime_global);
    match dep.mode {
      RequireHeaderDependencyMode::Replace => {
        source.replace(dep.range.start, dep.range.end, runtime, None);
      }
      RequireHeaderDependencyMode::GuardPreInitialization => {
        source.insert_static(dep.range.start, "(", None);
        source.insert(dep.range.end, format!(", {runtime})"), None);
      }
      RequireHeaderDependencyMode::EvaluateCreateRequireArgs => {
        source.replace(
          dep.range.start,
          dep.range.end,
          format!("(function() {{ return {runtime}; }})"),
          None,
        );
      }
      RequireHeaderDependencyMode::EvaluateCreateRequireCacheArgs => {
        source.replace(
          dep.range.start,
          dep.range.end,
          format!("(function() {{ return {{ cache: {runtime} }}; }})"),
          None,
        );
      }
    }
  }
}
