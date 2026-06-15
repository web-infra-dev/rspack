use rspack_cacheable::cacheable;
use rspack_core::{
  DependencyCodeGeneration, DependencyTemplate, DependencyTemplateType, DependencyType,
  TemplateContext, TemplateReplaceSource,
};
use rspack_plugin_javascript::dependency::{URLDependency, URLDependencyTemplate};

#[cacheable]
#[derive(Debug, Default)]
pub struct RstestUrlDependencyTemplate {
  /// List of extensions to preserve (e.g., `[".wasm", ".node"]`)
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
  fn render_ast(
    &self,
    dep: &dyn DependencyCodeGeneration,
    _code_generatable_context: &mut TemplateContext,
  ) -> Option<Vec<(rspack_core::DependencyRange, String)>> {
    let dep = dep
      .as_any()
      .downcast_ref::<URLDependency>()
      .expect("RstestUrlDependencyTemplate should be used for URLDependency");

    if self.should_preserve(dep.request()) {
      Some(Vec::new())
    } else {
      None
    }
  }

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

    if self.should_preserve(dep.request()) {
      return;
    }

    URLDependencyTemplate::default().render(dep, source, code_generatable_context);
  }
}

impl RstestUrlDependencyTemplate {
  fn should_preserve(&self, request: &str) -> bool {
    // Strip query string and fragment from request path before checking extension
    let request_path = request.split(&['?', '#'][..]).next().unwrap_or(request);

    request_path.rsplit('.').next().is_some_and(|ext| {
      self.preserve_extensions.iter().any(|preserve_ext| {
        // Support both ".ext" and "ext" formats
        let preserve_ext = preserve_ext.trim_start_matches('.');
        ext.eq_ignore_ascii_case(preserve_ext)
      })
    })
  }
}
