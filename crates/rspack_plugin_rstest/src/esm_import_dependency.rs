use rspack_cacheable::cacheable;
use rspack_core::{
  Dependency, DependencyCodeGeneration, DependencyTemplate, DependencyTemplateType, DependencyType,
  ImportAttributes, InitFragment, InitFragmentExt, InitFragmentKey, InitFragmentStage,
  ModuleDependency, RuntimeCondition, TemplateContext, TemplateReplaceSource,
};
use rspack_plugin_javascript::dependency::{
  ESMImportSideEffectDependency, ESMImportSideEffectDependencyTemplate,
  ESMImportSpecifierDependency, ESMImportSpecifierDependencyTemplate,
};

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct RstestESMImportSideEffectDependencyTemplate;

impl RstestESMImportSideEffectDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::EsmImport)
  }
}

impl DependencyTemplate for RstestESMImportSideEffectDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<ESMImportSideEffectDependency>()
      .expect("RstestESMImportSideEffectDependencyTemplate should only be used for ESMImportSideEffectDependency");
    let before_len = code_generatable_context.init_fragments.len();
    ESMImportSideEffectDependencyTemplate::default().render(dep, source, code_generatable_context);
    if has_import_actual_attr(dep.get_attributes()) {
      hoist_new_esm_import_fragments(
        code_generatable_context,
        before_len,
        Some(dep.request()),
        false,
      );
    }
  }
}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct RstestESMImportSpecifierDependencyTemplate;

impl RstestESMImportSpecifierDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::EsmImportSpecifier)
  }
}

impl DependencyTemplate for RstestESMImportSpecifierDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<ESMImportSpecifierDependency>()
      .expect("RstestESMImportSpecifierDependencyTemplate should only be used for ESMImportSpecifierDependency");

    let before_len = code_generatable_context.init_fragments.len();

    ESMImportSpecifierDependencyTemplate::default().render(dep, source, code_generatable_context);

    if !has_import_actual_attr(dep.get_attributes()) {
      return;
    }

    hoist_new_esm_import_fragments(
      code_generatable_context,
      before_len,
      Some(dep.request()),
      true,
    );
  }
}

fn hoist_new_esm_import_fragments(
  code_generatable_context: &mut TemplateContext,
  before_len: usize,
  request: Option<&str>,
  with_comment: bool,
) {
  let mut overrides = Vec::new();
  for fragment in &code_generatable_context.init_fragments[before_len..] {
    if fragment.stage() != InitFragmentStage::StageESMImports {
      continue;
    }
    let key = fragment.key().clone();
    let InitFragmentKey::ESMImport(_) = &key else {
      continue;
    };

    let cloned: Box<dyn InitFragment<_>> = fragment.clone();
    let Ok(conditional_fragment) = cloned
      .into_any()
      .downcast::<rspack_core::ConditionalInitFragment>()
    else {
      continue;
    };

    let content = if with_comment {
      format!(
        "// [Rstest importActual hoist] \"{}\"\n{}",
        request.unwrap_or_default(),
        conditional_fragment.content()
      )
    } else {
      conditional_fragment.content().to_string()
    };

    let runtime_condition = conditional_fragment.runtime_condition().clone();
    let override_fragment = rspack_core::ConditionalInitFragment::new(
      content,
      InitFragmentStage::StageESMImports,
      -1,
      key,
      None,
      runtime_condition,
    );
    overrides.push(override_fragment.boxed());
  }
  code_generatable_context.init_fragments.extend(overrides);
}

fn has_import_actual_attr(attributes: Option<&ImportAttributes>) -> bool {
  attributes
    .and_then(|attrs| attrs.get("rstest"))
    .is_some_and(|value| value == "importActual")
}
