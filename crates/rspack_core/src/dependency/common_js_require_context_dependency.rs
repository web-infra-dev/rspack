use rspack_error::Result;
use swc_core::{
  common::DUMMY_SP,
  ecma::{
    ast::{CallExpr, Expr, MemberExpr, MemberProp, ParenExpr},
    utils::{quote_ident, quote_str, ExprFactory},
  },
};

use crate::{
  create_javascript_visitor, normalize_context, CodeGeneratable, CodeGeneratableResult,
  ContextOptions, Dependency, DependencyId, ErrorSpan, JsAstPath, ModuleDependency, RuntimeGlobals,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommonJsRequireContextDependency {
  pub id: Option<DependencyId>,
  pub options: ContextOptions,
  span: Option<ErrorSpan>,
  #[allow(unused)]
  pub ast_path: JsAstPath,
}

impl CommonJsRequireContextDependency {
  pub fn new(options: ContextOptions, span: Option<ErrorSpan>, ast_path: JsAstPath) -> Self {
    Self {
      options,
      span,
      ast_path,
      id: None,
    }
  }
}

impl Dependency for CommonJsRequireContextDependency {
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
}

impl ModuleDependency for CommonJsRequireContextDependency {
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

impl CodeGeneratable for CommonJsRequireContextDependency {
  fn generate(&self, context: &mut crate::CodeGeneratableContext) -> Result<CodeGeneratableResult> {
    let compilation = context.compilation;
    let mut code_gen = CodeGeneratableResult::default();
    if let Some(id) = self.id() {
      if let Some(module_id) = compilation
        .module_graph
        .module_graph_module_by_dependency_id(&id)
        .map(|m| m.id(&compilation.chunk_graph))
      {
        let module_id = serde_json::to_string(module_id).expect("invalid module_id");
        let context = normalize_context(&self.options.request);
        code_gen.visitors.push(
          create_javascript_visitor!(exact &self.ast_path, visit_mut_call_expr(n: &mut CallExpr) {
            n.callee = Expr::Call(CallExpr {
              span: DUMMY_SP,
              callee: quote_ident!(DUMMY_SP, RuntimeGlobals::REQUIRE).as_callee(),
              args: vec![quote_ident!(DUMMY_SP, *module_id).as_arg()],
              type_args: None,
            }).as_callee();
            if !context.is_empty() {
              let mut args = std::mem::take(&mut n.args);
              n.args = vec![Expr::Call(CallExpr {
                span: DUMMY_SP,
                callee: MemberExpr { span: DUMMY_SP, obj: Box::new(Expr::Paren(ParenExpr { span: DUMMY_SP, expr: args.remove(0).expr })), prop: MemberProp::Ident(quote_ident!("replace")) }.as_callee(),
                args: vec![quote_str!(*context).as_arg(), quote_str!("./").as_arg()],
                type_args: None,
              }).as_arg()];
            }
          }),
        );
      }
    }

    Ok(code_gen)
  }
}
