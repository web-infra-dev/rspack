mod new_url_dependency;

use rspack_core::{
  create_javascript_visitor, CodeGeneratable, CodeGeneratableContext, CodeGeneratableResult,
  Dependency, DependencyCategory, DependencyId, DependencyType, ErrorSpan, JsAstPath,
  ModuleDependency, RuntimeGlobals,
};
use swc_core::common::Spanned;
use swc_core::ecma::utils::{member_expr, quote_ident, quote_str};
use swc_core::ecma::{ast::*, atoms::JsWord};

#[derive(Debug, Clone)]
pub struct URLDependency {
  id: Option<DependencyId>,
  request: JsWord,
  span: Option<ErrorSpan>,
  #[allow(unused)]
  ast_path: JsAstPath,
}

impl URLDependency {
  pub fn new(request: JsWord, span: Option<ErrorSpan>, ast_path: JsAstPath) -> Self {
    Self {
      id: None,
      request,
      span,
      ast_path,
    }
  }
}

impl Dependency for URLDependency {
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
    &DependencyType::NewUrl
  }
}

impl ModuleDependency for URLDependency {
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
    self.request = request.into();
  }
}

impl CodeGeneratable for URLDependency {
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
        code_generatable_context
          .runtime_requirements
          .insert(RuntimeGlobals::BASE_URI);
        code_gen.visitors.push(
          create_javascript_visitor!(exact &self.ast_path, visit_mut_new_expr(n: &mut NewExpr) {
                let Some(args) = &mut n.args else { return };

                if let (Some(first), Some(second)) = (args.first(), args.get(1)) {
                  let path_span = first.span();
                  let meta_span = second.span();

                  let require_call = CallExpr {
                    span: path_span,
                    callee: Callee::Expr(quote_ident!(RuntimeGlobals::REQUIRE).into()),
                    args: vec![ExprOrSpread {
                      spread: None,
                      expr: quote_str!(&*module_id).into(),
                    }],
                    type_args: None,
                  };

                  args[0] = ExprOrSpread {
                    spread: None,
                    expr: require_call.into(),
                  };

                  args[1] = ExprOrSpread {
                    spread: None,
                    expr: member_expr!(meta_span, __webpack_require__.b),
                  };
                }
          }),
        );
      }
    }

    Ok(code_gen)
  }
}
