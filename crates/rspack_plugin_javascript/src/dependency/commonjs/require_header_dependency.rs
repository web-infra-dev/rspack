use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, AsModuleDependency, Dependency, DependencyCodeGeneration, DependencyId,
  DependencyLocation, DependencyRange, DependencyTemplate, DependencyTemplateType, RuntimeGlobals,
  TemplateContext, TemplateReplaceSource,
};

#[cacheable]
#[derive(Debug, Clone, Copy)]
enum RequireHeaderMode {
  Direct,
  Conditional,
  Compatibility,
}

#[cacheable]
#[derive(Debug, Clone)]
pub struct RequireHeaderDependency {
  id: DependencyId,
  range: DependencyRange,
  loc: Option<DependencyLocation>,
  mode: RequireHeaderMode,
}

impl RequireHeaderDependency {
  pub fn new(range: DependencyRange, loc: Option<DependencyLocation>) -> Self {
    Self {
      id: DependencyId::new(),
      range,
      loc,
      mode: RequireHeaderMode::Direct,
    }
  }

  pub fn conditional(range: DependencyRange, loc: Option<DependencyLocation>) -> Self {
    Self {
      mode: RequireHeaderMode::Conditional,
      ..Self::new(range, loc)
    }
  }

  pub fn compatibility(range: DependencyRange, loc: Option<DependencyLocation>) -> Self {
    Self {
      mode: RequireHeaderMode::Compatibility,
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

    let content = if code_generatable_context
      .runtime_template
      .supports_module_relocations()
    {
      match dep.mode {
        // The module dependency replaces the complete call span.
        RequireHeaderMode::Direct => return,
        // Multiple conditional dependency markers share one call span. A
        // constructible identity retains call syntax (including trailing
        // commas) and `new require(value)` semantics without a named binding.
        RequireHeaderMode::Conditional => {
          source.replace(
            dep.range.start,
            dep.range.end,
            "(function(value) { return value; })".into(),
            None,
          );
          return;
        }
        // Opaque `new require(expression)` cannot be converted to a context
        // module. Keep its callable shape, but never fall back to a module-id
        // dispatcher.
        RequireHeaderMode::Compatibility => code_generatable_context
          .runtime_template
          .render_compatibility_require(),
      }
    } else {
      code_generatable_context
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::REQUIRE)
    };
    source.replace(dep.range.start, dep.range.end, content, None);
  }
}
