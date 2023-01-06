use swc_core::ecma::atoms::JsWord;
// use swc_core::ecma::{ast::*, atoms::Atom};

use crate::{
  // create_javascript_visitor,
  dependency::{
    CodeGeneratable, CodeGeneratableContext, CodeGeneratableResult, Dependency, DependencyCategory,
    ModuleDependency,
  },
  DependencyType,
  ErrorSpan,
  JsAstPath,
  ModuleIdentifier,
};

#[derive(Debug, Eq, Clone)]
pub struct EsmImportDependency {
  parent_module_identifier: Option<ModuleIdentifier>,
  request: JsWord,
  // user_request: String,
  category: &'static DependencyCategory,
  dependency_type: &'static DependencyType,

  span: Option<ErrorSpan>,
  #[allow(unused)]
  ast_path: JsAstPath,
}

// Do not edit this, as it is used to uniquely identify the dependency.
impl PartialEq for EsmImportDependency {
  fn eq(&self, other: &Self) -> bool {
    self.parent_module_identifier == other.parent_module_identifier
      && self.request == other.request
      && self.category == other.category
      && self.dependency_type == other.dependency_type
  }
}

// Do not edit this, as it is used to uniquely identify the dependency.
impl std::hash::Hash for EsmImportDependency {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.parent_module_identifier.hash(state);
    self.request.hash(state);
    self.category.hash(state);
    self.dependency_type.hash(state);
  }
}

impl EsmImportDependency {
  pub fn new(request: JsWord, span: Option<ErrorSpan>, ast_path: JsAstPath) -> Self {
    Self {
      parent_module_identifier: None,
      request,
      // user_request,
      category: &DependencyCategory::Esm,
      dependency_type: &DependencyType::EsmImport,
      span,
      ast_path,
    }
  }
}

impl Dependency for EsmImportDependency {
  fn parent_module_identifier(&self) -> Option<&ModuleIdentifier> {
    self.parent_module_identifier.as_ref()
  }

  fn set_parent_module_identifier(&mut self, module_identifier: Option<ModuleIdentifier>) {
    self.parent_module_identifier = module_identifier;
  }

  fn category(&self) -> &DependencyCategory {
    self.category
  }

  fn dependency_type(&self) -> &DependencyType {
    self.dependency_type
  }
}

impl ModuleDependency for EsmImportDependency {
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

static _SWC_HELPERS_REG: once_cell::sync::Lazy<regex::Regex> =
  once_cell::sync::Lazy::new(|| regex::Regex::new(r"@swc/helpers/lib/(\w*)\.js$").expect("TODO:"));

impl CodeGeneratable for EsmImportDependency {
  fn generate(&self, _code_generatable_context: &CodeGeneratableContext) -> CodeGeneratableResult {
    // let CodeGeneratableContext {
    //   compilation,
    //   module,
    // } = code_generatable_context;

    // let mut code_gen = CodeGeneratableResult::default();

    // let target_mgm = compilation
    //   .module_graph
    //   .module_graph_module_by_identifier(&module.identifier())
    //   .and_then(|mgm| {
    //     mgm.dependencies.iter().find_map(|dep| {
    //       if dep.request() == self.request() && dep.dependency_type() == self.dependency_type() {
    //         compilation.module_graph.module_by_dependency(dep)
    //       } else {
    //         None
    //       }
    //     })
    //   })
    //   .expect("Failed to get module graph module");

    // let _module_id = target_mgm.id(&compilation.chunk_graph).to_string();

    // let v =
    //   create_javascript_visitor!(&self.ast_path, visit_mut_module_decl(n: &mut ModuleDecl) {});

    // code_gen.visitors.push(v);

    // code_gen

    todo!()
  }
}
