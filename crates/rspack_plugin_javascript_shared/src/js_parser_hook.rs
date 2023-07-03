use std::{borrow::Cow, collections::HashMap};

use rspack_core::{CodeGeneratableDependency, ModuleDependency};
use swc_core::ecma::{
  ast,
  visit::{Visit, VisitWith},
};

type StaticStr = Cow<'static, str>;

#[derive(Debug, Default)]
pub struct JsParserContext {
  pub presentational_dependencies: Vec<Box<dyn CodeGeneratableDependency>>,
  pub dependencies: Vec<Box<dyn ModuleDependency>>,
}

/// The reason why we introduce [JsParserHookDriver] instead of using plain `swc_core::ecma_visit::Visit` is
/// we want to finish all work in a single visit pass.
#[derive(Default)]
pub struct JsParserHookDriver {
  hooks: HashMap<StaticStr, Vec<Box<dyn JsParserHook>>>,
  context: JsParserContext,
}

impl JsParserHookDriver {
  pub fn register_with_key(&mut self, key: impl Into<StaticStr>, hook: Box<dyn JsParserHook>) {
    let key = key.into();
    self.hooks.entry(key).or_default().push(hook);
  }

  pub fn into_context(self) -> JsParserContext {
    self.context
  }
}

pub enum Control {
  // Skip subsequent subscribers
  Skip,
  Continue,
}

/// This macro is used to help of reducing boilerplate code of adding
/// hook for various JS AST node type.
///
/// # Explanation
/// For input
/// ```ignore
/// register_hooks!(pub trait JsParserHook {
///   (visit_expr, ast::Expr),
/// });
/// ```
///
/// The macro would expand as
///
/// ```ignore
/// trait JsParserHook {
///   fn visit_expr(&mut self, _ctx: &mut JsParserContext, _node: &ast::Expr) {
///      false
///   }
/// }
///
/// impl<'me> Visit for JsAstVisitor<'me> {
///   fn visit_expr(&mut self, node: &ast::Expr) {
///     self.hooks.values_mut().for_each(|hooks| {
///       // Bailout if any hook returns `true`
///       hooks.iter_mut().any(|hook| hook.visit_expr(&self.context, node));
///     });
///     node.visit_children_with(self)
///   }
/// }
/// ```
macro_rules! register_hooks {
  (
    pub trait JsParserHook {
        $( ($name:ident,  $node:ty),)*
    }
  ) => {
    pub trait JsParserHook {
      $(
        fn $name(&mut self, _ctx: &mut JsParserContext,_node: &$node) -> Control {
          Control::Continue
        }
      )*
    }

    impl Visit for JsParserHookDriver {
        $(
        fn $name(&mut self, node: &$node) {
          self.hooks.values_mut().for_each(|hooks| {
            // Bailout if any hook returns `true`
            hooks.iter_mut().any(|hook| matches!(hook.$name(&mut self.context, node), Control::Skip));
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
// 4. We will get a `(visit_expr, ast::Expr)`, then add it in the following position.
// 5. Don't forget to add the comma.
register_hooks!(pub trait JsParserHook {
    (visit_expr, ast::Expr),
    (visit_ident, ast::Ident),
    (visit_call_expr, ast::CallExpr),
});

#[test]
fn example_basic() {
  use std::sync::{atomic::AtomicUsize, Arc};

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
    pub count: Arc<AtomicUsize>,
  }

  impl JsParserHook for Hook {
    fn visit_expr(&mut self, _ctx: &mut JsParserContext, _node: &ast::Expr) -> Control {
      let prev = self.count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
      if prev < 1 {
        Control::Continue
      } else {
        Control::Skip
      }
    }
  }

  let count = Arc::new(AtomicUsize::new(0));
  let hook1 = Hook {
    count: count.clone(),
  };
  let hook2 = Hook {
    count: count.clone(),
  };
  let hook3 = Hook {
    count: count.clone(),
  };
  let mut driver = JsParserHookDriver::default();
  driver.register_with_key("test", Box::new(hook1));
  driver.register_with_key("test", Box::new(hook2));
  driver.register_with_key("test", Box::new(hook3));
  ast.visit_with(&mut driver);
  assert_eq!(count.load(std::sync::atomic::Ordering::SeqCst), 2);
}
