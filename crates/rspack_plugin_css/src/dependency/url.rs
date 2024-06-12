use rspack_core::{
  AsContextDependency, CodeGenerationDataFilename, CodeGenerationDataUrl, Compilation, Dependency,
  DependencyCategory, DependencyId, DependencyTemplate, DependencyType, ErrorSpan,
  ModuleDependency, ModuleIdentifier, PublicPath, TemplateContext, TemplateReplaceSource,
};

use crate::utils::{css_escape_string, AUTO_PUBLIC_PATH_PLACEHOLDER};

#[derive(Debug, Clone)]
pub struct CssUrlDependency {
  id: DependencyId,
  request: String,
  span: Option<ErrorSpan>,
  start: u32,
  end: u32,
  replace_function: bool,
}

impl CssUrlDependency {
  pub fn new(
    request: String,
    span: Option<ErrorSpan>,
    start: u32,
    end: u32,
    replace_function: bool,
  ) -> Self {
    Self {
      request,
      span,
      start,
      end,
      id: DependencyId::new(),
      replace_function,
    }
  }

  fn get_target_url(
    &self,
    identifier: &ModuleIdentifier,
    compilation: &Compilation,
  ) -> Option<String> {
    // TODO: how to handle if module related to multi runtime codegen
    let code_gen_result = compilation.code_generation_results.get_one(identifier);
    if let Some(code_gen_result) = code_gen_result {
      if let Some(url) = code_gen_result.data.get::<CodeGenerationDataUrl>() {
        Some(url.inner().to_string())
      } else if let Some(data) = code_gen_result.data.get::<CodeGenerationDataFilename>() {
        let filename = data.filename();
        let public_path = match data.public_path() {
          PublicPath::String(p) => p,
          PublicPath::Auto => AUTO_PUBLIC_PATH_PLACEHOLDER,
        };
        Some(format!("{public_path}{filename}"))
      } else {
        None
      }
    } else {
      Some("data:,".to_string())
    }
  }
}

impl Dependency for CssUrlDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Url
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CssUrl
  }

  fn span(&self) -> Option<ErrorSpan> {
    self.span
  }
}

impl ModuleDependency for CssUrlDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn set_request(&mut self, request: String) {
    self.request = request;
  }
}

impl DependencyTemplate for CssUrlDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let TemplateContext { compilation, .. } = code_generatable_context;
    if let Some(mgm) = compilation
      .get_module_graph()
      .module_graph_module_by_dependency_id(self.id())
      && let Some(target_url) = self.get_target_url(&mgm.module_identifier, compilation)
    {
      let target_url = css_escape_string(&target_url);
      let content = if self.replace_function {
        format!("url({target_url})")
      } else {
        target_url
      };
      source.replace(self.start, self.end, &content, None);
    }
  }

  fn dependency_id(&self) -> Option<DependencyId> {
    Some(self.id)
  }
}

impl AsContextDependency for CssUrlDependency {}
