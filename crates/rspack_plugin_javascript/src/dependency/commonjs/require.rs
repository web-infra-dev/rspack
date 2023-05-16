use rspack_core::{
  create_javascript_visitor, CodeGeneratable, CodeGeneratableContext, CodeGeneratableResult,
  Dependency, DependencyCategory, DependencyId, DependencyType, ErrorSpan, JsAstPath,
  ModuleDependency,
};
use swc_core::ecma::{
  ast::*,
  atoms::{Atom, JsWord},
};

#[derive(Debug, Clone)]
pub struct CommonJSRequireDependency {
  id: Option<DependencyId>,
  request: JsWord,
  optional: bool,
  span: Option<ErrorSpan>,
  ast_path: JsAstPath,
}

impl CommonJSRequireDependency {
  pub fn new(
    request: JsWord,
    span: Option<ErrorSpan>,
    ast_path: JsAstPath,
    optional: bool,
  ) -> Self {
    Self {
      id: None,
      request,
      optional,
      span,
      ast_path,
    }
  }
}

impl Dependency for CommonJSRequireDependency {
  fn id(&self) -> Option<DependencyId> {
    self.id
  }
  fn set_id(&mut self, id: Option<DependencyId>) {
    self.id = id;
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CommonJS
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CjsRequire
  }
}

impl ModuleDependency for CommonJSRequireDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn span(&self) -> Option<&ErrorSpan> {
    self.span.as_ref()
  }

  fn get_optional(&self) -> bool {
    self.optional
  }
}

impl CodeGeneratable for CommonJSRequireDependency {
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
          create_javascript_visitor!(exact &self.ast_path, visit_mut_call_expr(n: &mut CallExpr) {
            if let Callee::Expr(box Expr::Ident(_ident)) = &mut n.callee {
              if let Some(ExprOrSpread {
                spread: None,
                expr: box Expr::Lit(Lit::Str(str)),
              }) = n.args.first_mut()
              {
                str.value = JsWord::from(&*module_id);
                str.raw = Some(Atom::from(format!("\"{module_id}\"")));
              }else if let Some(ExprOrSpread{
                spread:None,
                expr: box Expr::Tpl(tpl)
              }) = n.args.first_mut() && tpl.exprs.is_empty() {

                let s = tpl.quasis.first_mut().expect("should have one quasis");
                s.raw = Atom::from(module_id.as_str());


              }
            }
          }),
        );
      }
    }

    Ok(code_gen)
  }
}
