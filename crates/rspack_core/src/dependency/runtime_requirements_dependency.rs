use std::fmt::{Display, Formatter};

use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_hash::{RspackHash, RspackHasher};

use crate::{
  Compilation, DependencyCodeGeneration, DependencyRange, DependencyTemplate,
  DependencyTemplateType, RuntimeGlobals, RuntimeSpec, TemplateContext, TemplateReplaceSource,
};

#[cacheable]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum RuntimeRequirementsDependencyMode {
  #[default]
  Normal,
  Call,
  AddOnly,
  Write,
  WriteOnly,
  UnsupportedRequireProperty,
}

impl RspackHash for RuntimeRequirementsDependencyMode {
  fn hash(&self, state: &mut RspackHasher) {
    self.as_str().hash(state);
  }
}

impl Display for RuntimeRequirementsDependencyMode {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.write_str(self.as_str())
  }
}

impl RuntimeRequirementsDependencyMode {
  fn as_str(&self) -> &'static str {
    match self {
      RuntimeRequirementsDependencyMode::Normal => "normal",
      RuntimeRequirementsDependencyMode::Call => "call",
      RuntimeRequirementsDependencyMode::AddOnly => "add-only",
      RuntimeRequirementsDependencyMode::Write => "write",
      RuntimeRequirementsDependencyMode::WriteOnly => "write-only",
      RuntimeRequirementsDependencyMode::UnsupportedRequireProperty => {
        "unsupported-require-property"
      }
    }
  }
}

#[cacheable]
#[derive(Debug, Clone)]
pub struct RuntimeRequirementsDependency {
  pub range: DependencyRange,
  pub runtime_requirements: RuntimeGlobals,
  pub mode: RuntimeRequirementsDependencyMode,
}

impl RspackHash for RuntimeRequirementsDependency {
  fn hash(&self, state: &mut RspackHasher) {
    "runtime_requirements".hash(state);
    self.runtime_requirements.hash(state);
    match self.mode {
      RuntimeRequirementsDependencyMode::Normal => {
        "range".hash(state);
        self.range.hash(state);
      }
      RuntimeRequirementsDependencyMode::Call => {
        "range".hash(state);
        self.range.hash(state);
        "mode".hash(state);
        self.mode.hash(state);
      }
      RuntimeRequirementsDependencyMode::Write
      | RuntimeRequirementsDependencyMode::UnsupportedRequireProperty => {
        "range".hash(state);
        self.range.hash(state);
        "mode".hash(state);
        self.mode.hash(state);
      }
      RuntimeRequirementsDependencyMode::WriteOnly => {
        "mode".hash(state);
        self.mode.hash(state);
      }
      RuntimeRequirementsDependencyMode::AddOnly => {}
    }
  }
}

#[cacheable_dyn]
impl DependencyCodeGeneration for RuntimeRequirementsDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(RuntimeRequirementsDependencyTemplate::template_type())
  }

  fn update_hash(
    &self,
    hasher: &mut RspackHasher,
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) {
    RspackHash::hash(self, hasher);
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
