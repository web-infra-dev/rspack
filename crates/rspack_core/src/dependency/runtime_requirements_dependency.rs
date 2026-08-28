use std::fmt::{Display, Formatter};

use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_hash::{RspackHash, RspackHasher};

use crate::{
  CodeGenerationDataItem, Compilation, DependencyCodeGeneration, DependencyRange,
  DependencyTemplate, DependencyTemplateType, RuntimeGlobals, RuntimeGlobalsRenderMode,
  RuntimeSpec, TemplateContext, TemplateReplaceSource,
};

#[cacheable]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeRequirementsDependencyWriteOperation {
  Assign,
  Add,
  LogicalAnd,
  LogicalOr,
  NullishCoalescing,
}

impl RspackHash for RuntimeRequirementsDependencyWriteOperation {
  fn hash(&self, state: &mut RspackHasher) {
    (*self as u8).hash(state);
  }
}

#[cacheable]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeRequirementsDependencyWriteInfo {
  pub value_range: DependencyRange,
  pub assignment_range: DependencyRange,
  pub operation: RuntimeRequirementsDependencyWriteOperation,
}

impl RspackHash for RuntimeRequirementsDependencyWriteInfo {
  fn hash(&self, state: &mut RspackHasher) {
    self.value_range.hash(state);
    self.assignment_range.hash(state);
    self.operation.hash(state);
  }
}

#[cacheable]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum RuntimeRequirementsDependencyMode {
  #[default]
  Normal,
  Call,
  AddOnly,
  Write(Option<RuntimeRequirementsDependencyWriteInfo>),
  UnsupportedRequireProperty,
}

impl RspackHash for RuntimeRequirementsDependencyMode {
  fn hash(&self, state: &mut RspackHasher) {
    self.as_str().hash(state);
    if let RuntimeRequirementsDependencyMode::Write(write_info) = self {
      write_info.hash(state);
    }
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
      RuntimeRequirementsDependencyMode::Write(_) => "write",
      RuntimeRequirementsDependencyMode::UnsupportedRequireProperty => {
        "unsupported-require-property"
      }
    }
  }
}

#[cacheable]
#[derive(Debug)]
pub struct RuntimeRequirementsDependency {
  pub range: DependencyRange,
  pub runtime_requirements: RuntimeGlobals,
  pub mode: RuntimeRequirementsDependencyMode,
}

impl RspackHash for RuntimeRequirementsDependency {
  fn hash(&self, state: &mut RspackHasher) {
    "runtime_requirements".hash(state);
    self.runtime_requirements.hash(state);
    match &self.mode {
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
      RuntimeRequirementsDependencyMode::Write(_)
      | RuntimeRequirementsDependencyMode::UnsupportedRequireProperty => {
        "range".hash(state);
        self.range.hash(state);
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
      mode: RuntimeRequirementsDependencyMode::Write(None),
    }
  }
  pub fn write_assignment(
    range: DependencyRange,
    value_range: DependencyRange,
    assignment_range: DependencyRange,
    operation: RuntimeRequirementsDependencyWriteOperation,
    runtime_requirements: RuntimeGlobals,
  ) -> Self {
    Self {
      range,
      runtime_requirements,
      mode: RuntimeRequirementsDependencyMode::Write(Some(
        RuntimeRequirementsDependencyWriteInfo {
          value_range,
          assignment_range,
          operation,
        },
      )),
    }
  }
}

#[cacheable]
#[derive(Debug, Default, Clone)]
pub struct CodeGenerationRuntimeRequirementsWrite {
  pub runtime_requirements: RuntimeGlobals,
}

#[cacheable_dyn]
impl CodeGenerationDataItem for CodeGenerationRuntimeRequirementsWrite {}

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

    if matches!(&dep.mode, RuntimeRequirementsDependencyMode::AddOnly) {
      code_generatable_context
        .runtime_template
        .runtime_requirements_mut()
        .insert(dep.runtime_requirements);
      return;
    }

    if matches!(
      &dep.mode,
      RuntimeRequirementsDependencyMode::UnsupportedRequireProperty
    ) {
      source.replace(dep.range.start, dep.range.end, "undefined".into(), None);
      return;
    }

    if let RuntimeRequirementsDependencyMode::Write(write_info) = &dep.mode {
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
      if code_generatable_context.runtime_template.render_mode()
        == RuntimeGlobalsRenderMode::RspackExport
        && let Some(setter) = dep.runtime_requirements.to_rspack_export_setter_name()
      {
        let Some(write_info) = write_info else {
          let content = code_generatable_context
            .runtime_template
            .render_runtime_globals(&dep.runtime_requirements);
          source.replace(dep.range.start, dep.range.end, content, None);
          return;
        };
        let runtime_global = code_generatable_context
          .runtime_template
          .render_runtime_globals(&dep.runtime_requirements);
        let prefix = match write_info.operation {
          RuntimeRequirementsDependencyWriteOperation::Assign => format!("{setter}("),
          RuntimeRequirementsDependencyWriteOperation::Add => {
            format!("{setter}({runtime_global} + ")
          }
          RuntimeRequirementsDependencyWriteOperation::LogicalAnd => {
            format!("{runtime_global} && {setter}(")
          }
          RuntimeRequirementsDependencyWriteOperation::LogicalOr => {
            format!("{runtime_global} || {setter}(")
          }
          RuntimeRequirementsDependencyWriteOperation::NullishCoalescing => {
            format!("{runtime_global} ?? {setter}(")
          }
        };
        // ParenExpr nodes are removed before dependency creation. Rebuild one pair to preserve
        // RHS grouping while consuming the original trailing delimiters.
        let has_parenthesized_value = write_info.value_range.end < write_info.assignment_range.end;
        let (prefix, suffix) = if has_parenthesized_value {
          (format!("{prefix}("), "))")
        } else {
          (prefix, ")")
        };
        source.replace(dep.range.start, write_info.value_range.start, prefix, None);
        source.replace(
          write_info.value_range.end,
          write_info.assignment_range.end,
          suffix.to_string(),
          None,
        );
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

    if matches!(&dep.mode, RuntimeRequirementsDependencyMode::Call) {
      content = format!("{content}()");
    }

    source.replace(dep.range.start, dep.range.end, content, None);
  }
}
