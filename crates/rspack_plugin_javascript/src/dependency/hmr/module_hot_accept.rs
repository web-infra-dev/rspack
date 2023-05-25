use rspack_core::{
  create_javascript_visitor, module_id, CodeGeneratable, CodeGeneratableContext,
  CodeGeneratableResult, CodeReplaceSourceDependency, CodeReplaceSourceDependencyContext,
  CodeReplaceSourceDependencyReplaceSource, Dependency, DependencyCategory, DependencyId,
  DependencyType, ErrorSpan, JsAstPath, ModuleDependency,
};
use swc_core::ecma::atoms::{Atom, JsWord};

#[derive(Debug, Clone)]
pub struct ModuleHotAcceptDependency {
  id: Option<DependencyId>,
  request: JsWord,
  // user_request: String,
  category: &'static DependencyCategory,
  dependency_type: &'static DependencyType,

  span: Option<ErrorSpan>,
  #[allow(unused)]
  ast_path: JsAstPath,
}

impl ModuleHotAcceptDependency {
  pub fn new(request: JsWord, span: Option<ErrorSpan>, ast_path: JsAstPath) -> Self {
    Self {
      id: None,
      request,
      category: &DependencyCategory::CommonJS,
      dependency_type: &DependencyType::ModuleHotAccept,
      span,
      ast_path,
    }
  }
}

impl Dependency for ModuleHotAcceptDependency {
  fn id(&self) -> Option<DependencyId> {
    self.id
  }
  fn set_id(&mut self, id: Option<DependencyId>) {
    self.id = id;
  }

  fn category(&self) -> &DependencyCategory {
    self.category
  }

  fn dependency_type(&self) -> &DependencyType {
    self.dependency_type
  }
}

impl ModuleDependency for ModuleHotAcceptDependency {
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

impl CodeGeneratable for ModuleHotAcceptDependency {
  fn generate(
    &self,
    code_generatable_context: &mut CodeGeneratableContext,
  ) -> rspack_error::Result<CodeGeneratableResult> {
    let CodeGeneratableContext { compilation, .. } = code_generatable_context;

    let mut code_gen = CodeGeneratableResult::default();

    if let Some(id) = self.id() {
      if let Some(module_id) = compilation
        .module_graph
        .module_graph_module_by_dependency_id(&id)
        .map(|m| m.id(&compilation.chunk_graph).to_string())
      {
        code_gen.visitors.push(
          create_javascript_visitor!(exact &self.ast_path, visit_mut_str(str: &mut Str) {
            str.value = JsWord::from(&*module_id);
            str.raw = Some(Atom::from(format!("\"{module_id}\"")));
          }),
        );
      }
    }

    Ok(code_gen)
  }
}

#[derive(Debug, Clone)]
pub struct NewModuleHotAcceptDependency {
  id: Option<DependencyId>,
  request: JsWord,
  start: u32,
  end: u32,
  category: &'static DependencyCategory,
  dependency_type: &'static DependencyType,

  span: Option<ErrorSpan>,
}

impl NewModuleHotAcceptDependency {
  pub fn new(start: u32, end: u32, request: JsWord, span: Option<ErrorSpan>) -> Self {
    Self {
      id: None,
      request,
      category: &DependencyCategory::CommonJS,
      dependency_type: &DependencyType::ModuleHotAccept,
      span,
      start,
      end,
    }
  }
}

impl Dependency for NewModuleHotAcceptDependency {
  fn id(&self) -> Option<DependencyId> {
    self.id
  }
  fn set_id(&mut self, id: Option<DependencyId>) {
    self.id = id;
  }

  fn category(&self) -> &DependencyCategory {
    self.category
  }

  fn dependency_type(&self) -> &DependencyType {
    self.dependency_type
  }
}

impl ModuleDependency for NewModuleHotAcceptDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn span(&self) -> Option<&ErrorSpan> {
    self.span.as_ref()
  }

  fn as_code_replace_source_dependency(&self) -> Option<Box<dyn CodeReplaceSourceDependency>> {
    Some(Box::new(self.clone()))
  }
}

impl CodeGeneratable for NewModuleHotAcceptDependency {
  fn generate(
    &self,
    _code_generatable_context: &mut CodeGeneratableContext,
  ) -> rspack_error::Result<CodeGeneratableResult> {
    todo!()
  }
}

impl CodeReplaceSourceDependency for NewModuleHotAcceptDependency {
  fn apply(
    &self,
    source: &mut CodeReplaceSourceDependencyReplaceSource,
    code_generatable_context: &mut CodeReplaceSourceDependencyContext,
  ) {
    let id: DependencyId = self.id().expect("should have dependency id");

    source.replace(
      self.start,
      self.end,
      module_id(code_generatable_context.compilation, &id, &self.request).as_str(),
      None,
    );
  }
}
