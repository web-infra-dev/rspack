use rspack_error::Result;
use swc_core::{
  common::DUMMY_SP,
  ecma::{
    ast::{CallExpr, Expr},
    utils::{quote_ident, ExprFactory},
  },
};

use crate::{
  create_javascript_visitor, runtime_globals, CodeGeneratable, CodeGeneratableResult,
  ContextOptions, Dependency, ErrorSpan, JsAstPath, ModuleDependency, ModuleDependencyExt,
  ModuleIdentifier,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImportContextDependency {
  pub parent_module_identifier: Option<ModuleIdentifier>,
  pub options: ContextOptions,
  span: Option<ErrorSpan>,
  #[allow(unused)]
  pub ast_path: JsAstPath,
}

impl ImportContextDependency {
  pub fn new(options: ContextOptions, span: Option<ErrorSpan>, ast_path: JsAstPath) -> Self {
    Self {
      parent_module_identifier: None,
      options,
      span,
      ast_path,
    }
  }
}

impl Dependency for ImportContextDependency {
  fn category(&self) -> &crate::DependencyCategory {
    &crate::DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &crate::DependencyType {
    &crate::DependencyType::ImportContext
  }

  fn parent_module_identifier(&self) -> Option<&ModuleIdentifier> {
    self.parent_module_identifier.as_ref()
  }

  fn set_parent_module_identifier(&mut self, module_identifier: Option<ModuleIdentifier>) {
    self.parent_module_identifier = module_identifier;
  }
}

impl ModuleDependency for ImportContextDependency {
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

impl CodeGeneratable for ImportContextDependency {
  fn generate(&self, context: &mut crate::CodeGeneratableContext) -> Result<CodeGeneratableResult> {
    let compilation = context.compilation;
    let mut code_gen = CodeGeneratableResult::default();
    let referenced_module = self.referencing_module_graph_module(&compilation.module_graph);
    if let Some(referenced_module) = referenced_module {
      let module_id = format!("'{}'", referenced_module.id(&compilation.chunk_graph));
      code_gen.visitors.push(
        create_javascript_visitor!(exact &self.ast_path, visit_mut_call_expr(n: &mut CallExpr) {
          n.callee = Expr::Call(CallExpr {
            span: DUMMY_SP,
            callee: quote_ident!(DUMMY_SP, runtime_globals::REQUIRE).as_callee(),
            args: vec![quote_ident!(DUMMY_SP, *module_id).as_arg()],
            type_args: None,
          }).as_callee();
        }),
      );
    }

    Ok(code_gen)
  }
}
