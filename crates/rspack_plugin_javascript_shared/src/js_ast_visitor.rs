//! How to add a missing hook to `JsAstVisitorHook`
//! Find the missing hook at https://rustdoc.swc.rs/swc_ecma_visit/trait.Visit.html
use std::{borrow::Cow, collections::HashMap};

use swc_core::ecma::{
  ast,
  visit::{Visit, VisitWith},
};

type StaticStr = Cow<'static, str>;

/// The reason why we introduce [JsAstVisitor] instead of using `swc_core::ecma_visit::Visit` is
/// we want do finish all work in a single visit pass.
#[derive(Default)]
pub struct JsAstVisitor<'me> {
  hooks: HashMap<StaticStr, Vec<&'me mut dyn JsAstVisitorHook>>,
}

impl<'me> JsAstVisitor<'me> {
  pub fn tap(&mut self, key: impl Into<StaticStr>, hook: &'me mut dyn JsAstVisitorHook) {
    let key = key.into();
    self.hooks.entry(key).or_default().push(hook);
  }
}

/// This macro is used to help of reducing boilerplate code of adding
/// hook for various JS AST node type.
///
/// # Explanation
/// For input
/// ```ignore
/// register_hooks!(pub trait JsAstVisitorHook {
///   (visit_expr, ast::Expr),
/// });
/// ```
///
/// The macro would expand as
///
/// ```ignore
/// trait JsAstVisitorHook {
///   fn visit_expr(&mut self, node: &ast::Expr) {
///      false
///   }
/// }
///
/// impl<'me> Visit for JsAstVisitor<'me> {
///   fn visit_expr(&mut self, node: &ast::Expr) {
///     self.hooks.values_mut().for_each(|hooks| {
///       // Bailout if any hook returns `true`
///       hooks.iter_mut().any(|hook| hook.visit_expr(node));
///     });
///     node.visit_children_with(self)
///   }
/// }
/// ```
macro_rules! register_hooks {
  (
    pub trait JsAstVisitorHook {
        $( ($name:ident,  $node:ty),)*
    }
  ) => {
    pub trait JsAstVisitorHook {
      $(
        fn $name(&mut self, _node: &$node) -> bool {
          false
        }
      )*
    }

    impl<'me> Visit for JsAstVisitor<'me> {
        $(
        fn $name(&mut self, node: &$node) {
          self.hooks.values_mut().for_each(|hooks| {
            // Bailout if any hook returns `true`
            hooks.iter_mut().any(|hook| hook.$name(node));
          });
          node.visit_children_with(self)
        }
        )*
    }
  };
}

// # How to register a missing hook
// 1. Find the missing hook at https://rustdoc.swc.rs/swc_ecma_visit/trait.Visit.html
// 2. Let's take `fn visit_expr(&mut self, n: &Expr)` as an example.
// 3. Copy the name and Ast node type of the method and use a tuple to combine them.
// 4. We will get a `(visit_expr, ast::Expr)`, then add it in the following postion.
// 5. Don't forget to add the comma.
register_hooks!(pub trait JsAstVisitorHook {
    (visit_expr, ast::Expr),
    (visit_ident, ast::Ident),

});

#[test]
fn example_basic() {
  use swc_core::common::util::take::Take;

  let ast = ast::Module {
    body: vec![ast::ModuleItem::Stmt(ast::Stmt::Expr(ast::ExprStmt {
      ..ast::ExprStmt {
        span: Default::default(),
        expr: Box::new(ast::Expr::Array(ast::ArrayLit::dummy())),
      }
    }))],
    ..ast::Module::dummy()
  };

  #[derive(Default)]
  struct Hook {
    pub called: bool,
  }
  #[derive(Default)]
  struct Hook2 {
    pub called: bool,
  }
  #[derive(Default)]
  struct HookShouldBeSkipped {
    pub called: bool,
  }
  impl JsAstVisitorHook for Hook {
    fn visit_expr(&mut self, _node: &ast::Expr) -> bool {
      self.called = true;
      false
    }
  }
  impl JsAstVisitorHook for Hook2 {
    fn visit_expr(&mut self, _node: &ast::Expr) -> bool {
      self.called = true;
      true
    }
  }
  impl JsAstVisitorHook for HookShouldBeSkipped {
    fn visit_expr(&mut self, _node: &ast::Expr) -> bool {
      self.called = true;
      false
    }
  }

  let mut ast_visitor = JsAstVisitor::default();
  let mut hook = Hook::default();
  let mut hook2 = Hook2::default();
  let mut hook_should_skipped = HookShouldBeSkipped::default();
  ast_visitor.tap("test", &mut hook);
  ast_visitor.tap("test", &mut hook2);
  ast_visitor.tap("test", &mut hook_should_skipped);
  ast.visit_with(&mut ast_visitor);
  assert!(hook.called);
  assert!(hook2.called);
  assert!(!hook_should_skipped.called);
}
