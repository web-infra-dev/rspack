use rspack_core::{
  create_css_visitor, CodeGeneratable, CodeGeneratableContext, CodeGeneratableResult,
  CodeGenerationDataFilename, CodeGenerationDataUrl, Compilation, CssAstPath, Dependency,
  DependencyCategory, DependencyId, DependencyType, ErrorSpan, ModuleDependency, ModuleIdentifier,
  PublicPath,
};
use swc_core::css::ast::UrlValue;

use crate::utils::AUTO_PUBLIC_PATH_PLACEHOLDER;

#[derive(Debug, Clone)]
pub struct CssUrlDependency {
  id: Option<DependencyId>,
  request: String,
  span: Option<ErrorSpan>,
  #[allow(unused)]
  ast_path: CssAstPath,
}

impl CssUrlDependency {
  pub fn new(request: String, span: Option<ErrorSpan>, ast_path: CssAstPath) -> Self {
    Self {
      request,
      span,
      ast_path,
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
}

impl CodeGeneratable for CssUrlDependency {
  fn generate(
    &self,
    code_generatable_context: &mut CodeGeneratableContext,
  ) -> rspack_error::Result<CodeGeneratableResult> {
    let CodeGeneratableContext { compilation, .. } = code_generatable_context;
    let mut code_gen = CodeGeneratableResult::default();

    if let Some(id) = self.id()
      && let Some(mgm) = compilation
        .module_graph
        .module_graph_module_by_dependency_id(&id)
      && let Some(target_url) = self.get_target_url(&mgm.module_identifier, compilation)
    {
      code_gen.visitors.push(
        create_css_visitor!(exact &self.ast_path, visit_mut_url(url: &mut Url) {
          match url.value {
            Some(box UrlValue::Str(ref mut s)) => {
              s.raw = None;
              s.value = target_url.clone().into();
            }
            Some(box UrlValue::Raw(ref mut s)) => {
              s.raw = None;
              s.value = target_url.clone().into();
            }
            None => {}
          }
        }),
      );
    }

    Ok(code_gen)
  }
}
