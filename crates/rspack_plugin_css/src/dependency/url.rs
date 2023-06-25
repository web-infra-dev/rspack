use rspack_core::{
  CodeGeneratableContext, CodeGeneratableDependency, CodeGeneratableSource,
  CodeGenerationDataFilename, CodeGenerationDataUrl, Compilation, Dependency, DependencyCategory,
  DependencyId, DependencyType, ErrorSpan, ModuleDependency, ModuleIdentifier, PublicPath,
};

use crate::utils::AUTO_PUBLIC_PATH_PLACEHOLDER;

#[derive(Debug, Clone)]
pub struct CssUrlDependency {
  id: Option<DependencyId>,
  request: String,
  span: Option<ErrorSpan>,
  start: u32,
  end: u32,
}

impl CssUrlDependency {
  pub fn new(request: String, span: Option<ErrorSpan>, start: u32, end: u32) -> Self {
    Self {
      request,
      span,
      start,
      end,
      id: None,
    }
  }

  fn get_target_url(
    &self,
    identifier: &ModuleIdentifier,
    compilation: &Compilation,
  ) -> Option<String> {
    let code_gen_result = compilation
      .code_generation_results
      .module_generation_result_map
      .get(identifier);
    if let Some(code_gen_result) = code_gen_result {
      if let Some(url) = code_gen_result.data.get::<CodeGenerationDataUrl>() {
        Some(url.inner().to_string())
      } else if let Some(filename) = code_gen_result.data.get::<CodeGenerationDataFilename>() {
        let filename = filename.inner();
        let public_path = match &compilation.options.output.public_path {
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
  fn id(&self) -> Option<DependencyId> {
    self.id
  }
  fn set_id(&mut self, id: Option<DependencyId>) {
    self.id = id;
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Url
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CssUrl
  }
}

impl ModuleDependency for CssUrlDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn span(&self) -> Option<&ErrorSpan> {
    self.span.as_ref()
  }

  fn set_request(&mut self, request: String) {
    self.request = request;
  }

  fn as_code_generatable_dependency(&self) -> Option<Box<&dyn CodeGeneratableDependency>> {
    Some(Box::new(self))
  }
}

impl CodeGeneratableDependency for CssUrlDependency {
  fn apply(
    &self,
    source: &mut CodeGeneratableSource,
    code_generatable_context: &mut CodeGeneratableContext,
  ) {
    let CodeGeneratableContext { compilation, .. } = code_generatable_context;
    if let Some(id) = self.id()
      && let Some(mgm) = compilation
        .module_graph
        .module_graph_module_by_dependency_id(&id)
      && let Some(target_url) = self.get_target_url(&mgm.module_identifier, compilation)
    {
      source.replace(self.start, self.end, format!("url({target_url})").as_str(), None);
    }
  }
}
