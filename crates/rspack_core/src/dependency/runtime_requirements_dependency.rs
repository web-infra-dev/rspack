use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_util::ext::DynHash;

use crate::{
  Compilation, DependencyCodeGeneration, DependencyRange, DependencyTemplate,
  DependencyTemplateType, RuntimeGlobals, RuntimeSpec, TemplateContext, TemplateReplaceSource,
};

#[cacheable]
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub enum RuntimeRequirementsDependencyMode {
  #[default]
  Normal,
  Call,
  AddOnly,
  Write,
  WriteOnly,
  UnsupportedRequireProperty,
}

#[cacheable]
#[derive(Debug, Clone)]
pub struct RuntimeRequirementsDependency {
  pub range: DependencyRange,
  pub runtime_requirements: RuntimeGlobals,
  pub mode: RuntimeRequirementsDependencyMode,
}

#[cacheable_dyn]
impl DependencyCodeGeneration for RuntimeRequirementsDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(RuntimeRequirementsDependencyTemplate::template_type())
  }

  fn update_hash(
    &self,
    hasher: &mut dyn std::hash::Hasher,
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) {
    self.range.dyn_hash(hasher);
    self.runtime_requirements.dyn_hash(hasher);
    self.mode.dyn_hash(hasher);
  }
}

impl RuntimeRequirementsDependency {
  pub fn new(range: DependencyRange, runtime_requirements: RuntimeGlobals) -> Self {
    Self {
      range,
      runtime_requirements,
      mode: RuntimeRequirementsDependencyMode::Normal,
    }
  }
  pub fn call(range: DependencyRange, runtime_requirements: RuntimeGlobals) -> Self {
    Self {
      range,
      runtime_requirements,
      mode: RuntimeRequirementsDependencyMode::Call,
    }
  }
  pub fn add_only(runtime_requirements: RuntimeGlobals) -> Self {
    Self {
      range: DependencyRange::default(),
      runtime_requirements,
      mode: RuntimeRequirementsDependencyMode::AddOnly,
    }
  }
  pub fn write(range: DependencyRange, runtime_requirements: RuntimeGlobals) -> Self {
    Self {
      range,
      runtime_requirements,
      mode: RuntimeRequirementsDependencyMode::Write,
    }
  }
  pub fn write_only(runtime_requirements: RuntimeGlobals) -> Self {
    Self {
      range: DependencyRange::default(),
      runtime_requirements,
      mode: RuntimeRequirementsDependencyMode::WriteOnly,
    }
  }
  pub fn unsupported_require_property(
    range: DependencyRange,
    runtime_requirements: RuntimeGlobals,
  ) -> Self {
    Self {
      range,
      runtime_requirements,
      mode: RuntimeRequirementsDependencyMode::UnsupportedRequireProperty,
    }
  }
}

#[derive(Debug, Default, Clone)]
pub struct CodeGenerationRuntimeRequirementsWrite {
  pub runtime_requirements: RuntimeGlobals,
}

impl CodeGenerationRuntimeRequirementsWrite {
  pub fn insert(&mut self, runtime_requirements: RuntimeGlobals) {
    self.runtime_requirements.insert(runtime_requirements);
  }
}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct RuntimeRequirementsDependencyTemplate;

impl RuntimeRequirementsDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Custom("RuntimeRequirementsDependency")
  }
}

impl DependencyTemplate for RuntimeRequirementsDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<RuntimeRequirementsDependency>()
      .expect(
        "RuntimeRequirementsDependencyTemplate should be used for RuntimeRequirementsDependency",
      );

    if matches!(dep.mode, RuntimeRequirementsDependencyMode::AddOnly) {
      code_generatable_context
        .runtime_template
        .runtime_requirements_mut()
        .insert(dep.runtime_requirements);
      return;
    }

    if matches!(
      dep.mode,
      RuntimeRequirementsDependencyMode::UnsupportedRequireProperty
    ) {
      source.replace(dep.range.start, dep.range.end, "undefined".into(), None);
      return;
    }

    if matches!(
      dep.mode,
      RuntimeRequirementsDependencyMode::Write | RuntimeRequirementsDependencyMode::WriteOnly
    ) {
      code_generatable_context
        .runtime_template
        .runtime_requirements_mut()
        .insert(dep.runtime_requirements);
      if code_generatable_context
        .data
        .get::<CodeGenerationRuntimeRequirementsWrite>()
        .is_none()
      {
        code_generatable_context
          .data
          .insert(CodeGenerationRuntimeRequirementsWrite::default());
      }
      code_generatable_context
        .data
        .get_mut::<CodeGenerationRuntimeRequirementsWrite>()
        .expect("should have runtime requirements write metadata")
        .insert(dep.runtime_requirements);
      if matches!(dep.mode, RuntimeRequirementsDependencyMode::WriteOnly) {
        return;
      }
      let content = code_generatable_context
        .runtime_template
        .render_runtime_globals(&dep.runtime_requirements);
      source.replace(dep.range.start, dep.range.end, content, None);
      return;
    }

    let mut content = code_generatable_context
      .runtime_template
      .render_runtime_globals(&dep.runtime_requirements);

    if matches!(dep.mode, RuntimeRequirementsDependencyMode::Call) {
      content = format!("{content}()");
    }

    source.replace(dep.range.start, dep.range.end, content, None);
  }
}
