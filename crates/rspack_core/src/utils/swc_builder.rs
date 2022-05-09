use ast::{CallExpr, Callee, Expr, ExprOrSpread, Ident, Lit};
use swc_atoms::js_word;

pub fn is_dynamic_import(e: &mut CallExpr) -> bool {
  if let Callee::Import(_) = e.callee {
    true
  } else {
    false
  }
}
pub fn dynamic_import_with_literal(e: &mut CallExpr) -> Option<String> {
  if is_dynamic_import(e) {
    if let Some(ExprOrSpread {
      spread: None,
      expr: box Expr::Lit(Lit::Str(str)),
    }) = e.args.first()
    {
      return Some(str.value.to_string());
    }
  }
  return None;
}
pub fn is_require(e: &mut CallExpr) -> bool {
  if let Callee::Expr(box Expr::Ident(Ident {
    sym: js_word!("require"),
    ..
  })) = &e.callee
  {
    true
  } else {
    false
  }
}

#[cfg(test)]
mod swc_builder_test {
  use ast::{CallExpr, Ident, Lit};
  use swc_common::DUMMY_SP;
  use swc_ecma_utils::ExprFactory;
  use swc_ecma_visit::{VisitMut, VisitMutWith};

  use crate::{
    swc_builder::{dynamic_import_with_literal, is_dynamic_import, is_require},
    test_runner::compile,
  };

  #[test]
  fn dynamic_require() {
    let (mut ast, _) = compile(
      r#"
      const x = import('./a');
      const y = require('./b');
    "#
      .into(),
      None,
    );
    #[derive(Debug)]
    struct CheckVisitor {
      dynamic_called: usize,
      require_called: usize,
      string_literal_called: usize,
    }
    impl CheckVisitor {
      fn new() -> Self {
        Self {
          dynamic_called: 0,
          require_called: 0,
          string_literal_called: 0,
        }
      }
    }

    impl VisitMut for CheckVisitor {
      fn visit_mut_call_expr(&mut self, node: &mut CallExpr) {
        if is_dynamic_import(node) {
          self.dynamic_called += 1;
        }
        if is_require(node) {
          self.require_called += 1;
        }
        if let Some(str) = dynamic_import_with_literal(node) {
          self.string_literal_called += 1;
        }
      }
    }

    struct TransformVisitor {}
    impl TransformVisitor {
      fn new() -> TransformVisitor {
        TransformVisitor {}
      }
    }
    impl VisitMut for TransformVisitor {
      fn visit_mut_call_expr(&mut self, node: &mut CallExpr) {
        if let Some(str) = dynamic_import_with_literal(node) {
          let callee = Ident::new("require".into(), DUMMY_SP).as_callee();
          let arg = Lit::Str(str.into()).as_arg();
          node.callee = callee;
          node.args = vec![arg];
        }
      }
    }
    let mut check_visitor = CheckVisitor::new();
    let mut transform_visitor = TransformVisitor::new();
    ast.visit_mut_with(&mut check_visitor);

    assert_eq!(check_visitor.dynamic_called, 1);
    assert_eq!(check_visitor.require_called, 1);
    assert_eq!(check_visitor.string_literal_called, 1);
    ast.visit_mut_with(&mut transform_visitor);
    let (_, code) = compile(Default::default(), Some(ast));
    assert_eq!(
      code.code,
      "const x = require(\"./a\");\nconst y = require('./b');\n"
    );
  }
}
