use rspack_cacheable::cacheable;
use rspack_core::{
  DependencyCodeGeneration, DependencyTemplate, DependencyTemplateType, DependencyType,
  ModuleDependency, TemplateContext, TemplateReplaceSource,
};
use rspack_plugin_javascript::dependency::{URLDependency, URLDependencyTemplate};

#[cacheable]
#[derive(Debug, Default)]
pub struct RstestUrlDependencyTemplate {
  /// List of extensions to preserve (e.g., [".wasm", ".node"])
  preserve_extensions: Vec<String>,
}

impl RstestUrlDependencyTemplate {
  pub fn new(preserve_extensions: Vec<String>) -> Self {
    Self {
      preserve_extensions,
    }
  }

  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::NewUrl)
  }
}

impl DependencyTemplate for RstestUrlDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<URLDependency>()
      .expect("RstestUrlDependencyTemplate should be used for URLDependency");

    // Strip query string and fragment from request path before checking extension
    let request = dep.request();
    let request_path = request.split(&['?', '#'][..]).next().unwrap_or(request);

    let should_preserve = request_path.rsplit('.').next().is_some_and(|ext| {
      self.preserve_extensions.iter().any(|preserve_ext| {
        // Support both ".ext" and "ext" formats
        let preserve_ext = preserve_ext.trim_start_matches('.');
        ext.eq_ignore_ascii_case(preserve_ext)
      })
    });

    if should_preserve {
      return;
    }

    URLDependencyTemplate::default().render(dep, source, code_generatable_context);
  }
}
