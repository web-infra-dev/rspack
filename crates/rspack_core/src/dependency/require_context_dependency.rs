use rspack_error::Result;
use swc_core::{
  common::DUMMY_SP,
  ecma::{
    ast::CallExpr,
    utils::{quote_ident, ExprFactory},
  },
};

use crate::{
  create_javascript_visitor, runtime_globals, CodeGeneratable, CodeGeneratableResult,
  ContextOptions, Dependency, DependencyId, ErrorSpan, JsAstPath, ModuleDependency,
  ModuleIdentifier,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RequireContextDependency {
  pub id: Option<DependencyId>,
  pub parent_module_identifier: Option<ModuleIdentifier>,
  pub options: ContextOptions,
  span: Option<ErrorSpan>,
  #[allow(unused)]
  pub ast_path: JsAstPath,
}

impl RequireContextDependency {
  pub fn new(options: ContextOptions, span: Option<ErrorSpan>, ast_path: JsAstPath) -> Self {
    Self {
      parent_module_identifier: None,
      options,
      span,
      ast_path,
      id: None,
    }
  }
}

impl Dependency for RequireContextDependency {
  fn id(&self) -> Option<DependencyId> {
    self.id
  }
  fn set_id(&mut self, id: Option<DependencyId>) {
    self.id = id;
  }
  fn category(&self) -> &crate::DependencyCategory {
    &crate::DependencyCategory::CommonJS
  }

  fn dependency_type(&self) -> &crate::DependencyType {
    &crate::DependencyType::CommonJSRequireContext
  }

  fn parent_module_identifier(&self) -> Option<&ModuleIdentifier> {
    self.parent_module_identifier.as_ref()
  }

  fn set_parent_module_identifier(&mut self, module_identifier: Option<ModuleIdentifier>) {
    self.parent_module_identifier = module_identifier;
  }
}

impl ModuleDependency for RequireContextDependency {
  fn request(&self) -> &str {
    &self.options.request
  }

  fn user_request(&self) -> &str {
    &self.options.request
  }

  fn span(&self) -> Option<&crate::ErrorSpan> {
    None
  }

  fn options(&self) -> Option<&ContextOptions> {
    Some(&self.options)
  }
}

impl CodeGeneratable for RequireContextDependency {
  fn generate(&self, context: &mut crate::CodeGeneratableContext) -> Result<CodeGeneratableResult> {
    let compilation = context.compilation;
    let mut code_gen = CodeGeneratableResult::default();
    if let Some(id) = self.id() {
      if let Some(module_id) = compilation
        .module_graph
        .module_graph_module_by_dependency_id(&id)
        .map(|m| m.id(&compilation.chunk_graph).to_string())
      {
        let module_id = format!("'{module_id}'");
        code_gen.visitors.push(
          create_javascript_visitor!(exact &self.ast_path, visit_mut_call_expr(n: &mut CallExpr) {
            *n = CallExpr {
              span: DUMMY_SP,
              callee: quote_ident!(DUMMY_SP, runtime_globals::REQUIRE).as_callee(),
              args: vec![quote_ident!(DUMMY_SP, *module_id).as_arg()],
              type_args: None,
            }
          }),
        );
      }
    }

    Ok(code_gen)
  }
}
