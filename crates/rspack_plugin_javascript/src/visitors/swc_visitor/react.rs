use std::sync::Arc;

use once_cell::sync::Lazy;
use rspack_core::{ModuleType, ReactOptions};
use swc_core::common::{comments::SingleThreadedComments, Mark, SourceMap};
use swc_core::ecma::ast::{CallExpr, Callee, Expr, Ident, ModuleItem, Program, Script};
use swc_core::ecma::transforms::react::RefreshOptions;
use swc_core::ecma::transforms::react::{react as swc_react, Options};
use swc_core::ecma::visit::{noop_visit_type, Fold, Visit, VisitWith};

use crate::ast::parse_js_code;

pub fn react<'a>(
  top_level_mark: Mark,
  comments: Option<&'a SingleThreadedComments>,
  cm: &Arc<SourceMap>,
  options: &ReactOptions,
  unresolved_mark: Mark,
) -> impl Fold + 'a {
  swc_react(
    cm.clone(),
    comments,
    Options {
      refresh: options.refresh.and_then(|dev| {
        if dev {
          Some(RefreshOptions::default())
        } else {
          None
        }
      }),
      runtime: options.runtime,
      import_source: options.import_source.clone(),
      pragma: options.pragma.clone(),
      pragma_frag: options.pragma_frag.clone(),
      throw_if_namespace: options.throw_if_namespace,
      development: options.development,
      ..Default::default()
    },
    top_level_mark,
    unresolved_mark,
  )
}

pub fn fold_react_refresh() -> impl Fold {
  ReactHmrFolder {}
}

#[derive(Default)]
struct ReactRefreshUsageFinder {
  pub is_founded: bool,
}

impl Visit for ReactRefreshUsageFinder {
  noop_visit_type!();

  fn visit_module_items(&mut self, items: &[ModuleItem]) {
    for item in items {
      item.visit_children_with(self);
      if self.is_founded {
        return;
      }
    }
  }

  fn visit_call_expr(&mut self, call_expr: &CallExpr) {
    if self.is_founded {
      return;
    }

    self.is_founded = matches!(call_expr, CallExpr {
      callee: Callee::Expr(box Expr::Ident(Ident { sym, .. })),
      ..
    } if sym == "$RefreshReg$" || sym == "$RefreshSig$");

    if self.is_founded {
      return;
    }

    call_expr.visit_children_with(self);
  }
}

// __webpack_require__.$ReactRefreshRuntime$ is injected by the react-refresh additional entry
// See https://github.com/web-infra-dev/rspack/pull/2714 why we have a promise here
static RUNTIME_CODE: &str = r#"
function $RefreshReg$(type, id) {
  __webpack_modules__.$ReactRefreshRuntime$.register(type, __webpack_module__.id+ "_" + id);
}
Promise.resolve().then(function(){ 
  __webpack_modules__.$ReactRefreshRuntime$.refresh(__webpack_module__.id, module.hot);
})
"#;

static RUNTIME_CODE_AST: Lazy<Script> = Lazy::new(|| {
  parse_js_code(RUNTIME_CODE.to_string(), &ModuleType::Js)
    .expect("TODO:")
    .expect_script()
});

pub struct ReactHmrFolder {}

impl Fold for ReactHmrFolder {
  fn fold_program(&mut self, mut program: Program) -> Program {
    let mut f = ReactRefreshUsageFinder::default();

    program.visit_with(&mut f);
    if !f.is_founded {
      return program;
    }

    let rumtime_stmts = RUNTIME_CODE_AST.body.clone();

    match program {
      Program::Module(ref mut m) => m
        .body
        .extend(rumtime_stmts.into_iter().map(ModuleItem::Stmt)),
      Program::Script(ref mut s) => s.body.extend(rumtime_stmts),
    };

    program
  }
}
