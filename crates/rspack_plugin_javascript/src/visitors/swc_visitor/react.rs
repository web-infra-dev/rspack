use std::sync::Arc;

use once_cell::sync::Lazy;
use rspack_core::{ModuleType, ReactOptions};
use swc_core::common::{comments::SingleThreadedComments, Mark, SourceMap};
use swc_core::ecma::ast::{CallExpr, Callee, Expr, Ident, Module, ModuleItem, Program};
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
static HMR_HEADER: &str = r#"var RefreshRuntime = __webpack_modules__.$ReactRefreshRuntime$;
var $RefreshReg$ = function (type, id) {
  RefreshRuntime.register(type, __webpack_module__.id + "_" + id);
}
var $RefreshSig$ = RefreshRuntime.createSignatureFunctionForTransform;"#;

// See https://github.com/web-infra-dev/rspack/pull/2714 why we have a promise here
static HMR_FOOTER: &str = r#"Promise.resolve().then(function(){ 
  __webpack_modules__.$ReactRefreshRuntime$.refresh(__webpack_module__.id, module.hot);
})"#;

static HMR_HEADER_AST: Lazy<Program> =
  Lazy::new(|| parse_js_code(HMR_HEADER.to_string(), &ModuleType::Js).expect("TODO:"));

static HMR_FOOTER_AST: Lazy<Program> =
  Lazy::new(|| parse_js_code(HMR_FOOTER.to_string(), &ModuleType::Js).expect("TODO:"));

pub struct ReactHmrFolder {}

impl Fold for ReactHmrFolder {
  fn fold_module(&mut self, mut module: Module) -> Module {
    let mut f = ReactRefreshUsageFinder::default();

    module.visit_with(&mut f);
    if !f.is_founded {
      return module;
    }

    let mut body = vec![];
    if let Some(m) = HMR_HEADER_AST.as_module() {
      body.append(&mut m.body.clone());
    }
    body.append(&mut module.body);
    if let Some(m) = HMR_FOOTER_AST.as_module() {
      body.append(&mut m.body.clone());
    }

    Module { body, ..module }
  }
}
