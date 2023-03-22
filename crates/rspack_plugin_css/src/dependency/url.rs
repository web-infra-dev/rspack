use rspack_core::{
  create_css_visitor, CodeGeneratable, CodeGeneratableContext, CodeGeneratableResult, Compilation,
  CssAstPath, Dependency, DependencyCategory, DependencyId, DependencyType, ErrorSpan,
  ModuleDependency, ModuleIdentifier,
};
use swc_core::css::ast::UrlValue;

#[derive(Debug, Eq, Clone)]
pub struct CssUrlDependency {
  id: Option<DependencyId>,
  parent_module_identifier: Option<ModuleIdentifier>,
  request: String,
  span: Option<ErrorSpan>,
  #[allow(unused)]
  ast_path: CssAstPath,
}

// Do not edit this, as it is used to uniquely identify the dependency.
impl PartialEq for CssUrlDependency {
  fn eq(&self, other: &Self) -> bool {
    self.parent_module_identifier == other.parent_module_identifier && self.request == other.request
  }
}

// Do not edit this, as it is used to uniquely identify the dependency.
impl std::hash::Hash for CssUrlDependency {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.parent_module_identifier.hash(state);
    self.request.hash(state);
  }
}

impl CssUrlDependency {
  pub fn new(request: String, span: Option<ErrorSpan>, ast_path: CssAstPath) -> Self {
    Self {
      parent_module_identifier: None,
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
      if let Some(url) = code_gen_result.data.get("url") {
        Some(url.to_string())
      } else if let Some(filename) = code_gen_result.data.get("filename") {
        let public_path = compilation
          .options
          .output
          .public_path
          .render(compilation, filename);
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
  fn parent_module_identifier(&self) -> Option<&ModuleIdentifier> {
    self.parent_module_identifier.as_ref()
  }

  fn set_parent_module_identifier(&mut self, identifier: Option<ModuleIdentifier>) {
    self.parent_module_identifier = identifier;
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
