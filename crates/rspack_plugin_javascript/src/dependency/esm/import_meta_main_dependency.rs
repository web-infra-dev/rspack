use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  CodeGenerationModuleReferenceKind, DependencyCodeGeneration, DependencyTemplate,
  DependencyTemplateType, TemplateContext, TemplateReplaceSource,
};

/// Records the current module's `import.meta.main` value as a typed modern-
/// module relocation. The actual source range is replaced by a ConstDependency
/// so this also works when `import.meta` is expanded for destructuring.
#[cacheable]
#[derive(Debug, Clone)]
pub struct ImportMetaMainDependency;

#[cacheable_dyn]
impl DependencyCodeGeneration for ImportMetaMainDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(ImportMetaMainDependencyTemplate::template_type())
  }
}

#[cacheable]
#[derive(Debug, Default)]
pub struct ImportMetaMainDependencyTemplate;

impl ImportMetaMainDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Custom("ImportMetaMainDependency")
  }
}

impl DependencyTemplate for ImportMetaMainDependencyTemplate {
  fn render(
    &self,
    _dep: &dyn DependencyCodeGeneration,
    _source: &mut TemplateReplaceSource,
    context: &mut TemplateContext,
  ) {
    context.create_module_relocation_for_module(
      context.module.identifier(),
      CodeGenerationModuleReferenceKind::EntryValue,
    );
  }
}
