use std::{path::Path, sync::Arc};

use once_cell::sync::Lazy;
use rspack_core::{contextify, ModuleType, ReactOptions};
use swc_core::common::{comments::SingleThreadedComments, Mark, SourceMap};
use swc_core::ecma::ast::{CallExpr, Callee, Expr, Module, Program};
use swc_core::ecma::transforms::react::RefreshOptions;
use swc_core::ecma::transforms::react::{react as swc_react, Options};
use swc_core::ecma::visit::{Fold, Visit, VisitWith};

use crate::ast::parse_js_code;

pub fn react<'a>(
  top_level_mark: Mark,
  comments: Option<&'a SingleThreadedComments>,
  cm: &Arc<SourceMap>,
  options: &ReactOptions,
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
      // use_builtins: options.use_builtins,
      // use_spread: options.use_spread,
      ..Default::default()
    },
    top_level_mark,
  )
}

pub fn fold_react_refresh(context: &Path, uri: &str) -> impl Fold {
  ReactHmrFolder {
    id: contextify(context, uri),
  }
}

pub struct FoundReactRefreshVisitor {
  pub is_refresh_boundary: bool,
}

impl Visit for FoundReactRefreshVisitor {
  fn visit_call_expr(&mut self, call_expr: &CallExpr) {
    if let Callee::Expr(expr) = &call_expr.callee {
      if let Expr::Ident(ident) = &**expr {
        if "$RefreshReg$".eq(&ident.sym) {
          self.is_refresh_boundary = true;
        }
      }
    }
  }
}

// __webpack_require__.$ReactRefreshRuntime$ is injected by the react-refresh additional entry
static HMR_HEADER: &str = r#"var RefreshRuntime = __webpack_require__.m.$ReactRefreshRuntime$;
var prevRefreshReg;
var prevRefreshSig;
prevRefreshReg = globalThis.$RefreshReg$;
prevRefreshSig = globalThis.$RefreshSig$;
globalThis.$RefreshReg$ = (type, id) => {
  RefreshRuntime.register(type, "__SOURCE__" + "_" + id);
};
globalThis.$RefreshSig$ = RefreshRuntime.createSignatureFunctionForTransform;"#;

static HMR_FOOTER: &str = r#"var RefreshRuntime = __webpack_require__.m.$ReactRefreshRuntime$;
globalThis.$RefreshReg$ = prevRefreshReg;
globalThis.$RefreshSig$ = prevRefreshSig;
module.hot.accept();
RefreshRuntime.queueUpdate();
"#;

static HMR_FOOTER_AST: Lazy<Program> =
  Lazy::new(|| parse_js_code(HMR_FOOTER.to_string(), &ModuleType::Js).expect("TODO:"));

pub struct ReactHmrFolder {
  pub id: String,
}

impl Fold for ReactHmrFolder {
  fn fold_module(&mut self, mut module: Module) -> Module {
    let mut f = FoundReactRefreshVisitor {
      is_refresh_boundary: false,
    };

    module.visit_with(&mut f);
    if !f.is_refresh_boundary {
      return module;
    }
    // TODO: cache the ast
    let hmr_header_ast = parse_js_code(
      HMR_HEADER.replace("__SOURCE__", self.id.as_str()),
      &ModuleType::Js,
    )
    .expect("TODO:");

    let mut body = vec![];
    body.append(&mut match hmr_header_ast {
      Program::Module(m) => m.body,
      _ => vec![],
    });
    body.append(&mut module.body);
    if let Some(m) = HMR_FOOTER_AST.as_module() {
      body.append(&mut m.body.clone());
    }

    Module { body, ..module }
  }
}
