use swc_core::ecma::ast::*;
use swc_core::ecma::visit::Visit;

#[derive(Default)]
pub struct ReactServerComponentsVisitor {
  pub directives: Vec<String>,
}

impl ReactServerComponentsVisitor {
  pub fn new() -> Self {
    Self { directives: vec![] }
  }
}

impl Visit for ReactServerComponentsVisitor {
  fn visit_expr_stmt(&mut self, n: &ExprStmt) {
    let expr = n.expr.clone();
    match *expr {
      Expr::Lit(l) => {
        if let Lit::Str(s) = l {
          if s.value.starts_with("use ") {
            self.directives.push(s.value.to_string())
          }
        }
      }
      _ => (),
    };
  }
}
