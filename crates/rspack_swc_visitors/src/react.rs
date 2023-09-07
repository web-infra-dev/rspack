use std::sync::Arc;

use serde::Deserialize;
use swc_core::common::comments::Comments;
use swc_core::common::DUMMY_SP;
use swc_core::common::{Mark, SourceMap};
use swc_core::ecma::ast::{BlockStmt, FnDecl, Function, Ident, ModuleItem, Program, Stmt};
use swc_core::ecma::transforms::react::RefreshOptions;
use swc_core::ecma::transforms::react::Runtime;
use swc_core::ecma::transforms::react::{react as swc_react, Options};
use swc_core::ecma::utils::quote_ident;
use swc_core::ecma::visit::Fold;
use swc_core::quote;

#[derive(Debug, Clone, Default)]
pub struct ReactOptions {
  pub runtime: Option<Runtime>,
  pub import_source: Option<String>,
  pub pragma: Option<String>,
  pub pragma_frag: Option<String>,
  pub throw_if_namespace: Option<bool>,
  pub development: Option<bool>,
  pub use_builtins: Option<bool>,
  pub use_spread: Option<bool>,
  pub refresh: Option<bool>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RawReactOptions {
  pub runtime: Option<String>,
  pub import_source: Option<String>,
  pub pragma: Option<String>,
  pub pragma_frag: Option<String>,
  pub throw_if_namespace: Option<bool>,
  pub development: Option<bool>,
  pub use_builtins: Option<bool>,
  pub use_spread: Option<bool>,
  pub refresh: Option<bool>,
}

impl From<RawReactOptions> for ReactOptions {
  fn from(value: RawReactOptions) -> Self {
    let runtime = if let Some(runtime) = &value.runtime {
      match runtime.as_str() {
        "automatic" => Some(Runtime::Automatic),
        "classic" => Some(Runtime::Classic),
        _ => None,
      }
    } else {
      Some(Runtime::Automatic)
    };

    ReactOptions {
      runtime,
      import_source: value.import_source,
      pragma: value.pragma,
      pragma_frag: value.pragma_frag,
      throw_if_namespace: value.throw_if_namespace,
      development: value.development,
      use_builtins: value.use_builtins,
      use_spread: value.use_spread,
      refresh: value.refresh,
    }
  }
}

pub fn react<'a>(
  top_level_mark: Mark,
  comments: Option<&'a dyn Comments>,
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

pub fn fold_react_refresh(unresolved_mark: Mark) -> impl Fold {
  ReactHmrFolder { unresolved_mark }
}

// $ReactRefreshRuntime$ is injected by provide
//
// function $RefreshReg$(type, id) {
//   $ReactRefreshRuntime$.register(type, __webpack_module__.id + "_" + id);
// }
// Promise.resolve().then(function() {
//   $ReactRefreshRuntime$.refresh(__webpack_module__.id, __webpack_module__.hot);
// });
fn create_react_refresh_runtime_stmts(unresolved_mark: Mark) -> Vec<Stmt> {
  fn create_react_refresh_runtime_ident(unresolved_mark: Mark) -> Ident {
    Ident {
      span: DUMMY_SP.apply_mark(unresolved_mark),
      sym: "$ReactRefreshRuntime$".into(),
      optional: false,
    }
  }

  vec![
    FnDecl {
      ident: quote_ident!("$RefreshReg$"),
      declare: false,
      function: Box::new(Function {
        params: vec![quote_ident!("type").into(), quote_ident!("id").into()],
        decorators: Vec::new(),
        span: DUMMY_SP,
        body: Some(BlockStmt {
          span: DUMMY_SP,
          stmts: vec![quote!(
            "$runtime.register(type, __webpack_module__.id + \"_\" + id);" as Stmt,
            runtime = create_react_refresh_runtime_ident(unresolved_mark)
          )],
        }),
        is_generator: false,
        is_async: false,
        type_params: None,
        return_type: None,
      }),
    }
    .into(),
    // See https://github.com/web-infra-dev/rspack/pull/2714 why we have a promise here
    quote!("Promise.resolve().then(function() { $runtime.refresh(__webpack_module__.id, __webpack_module__.hot); });" as Stmt, runtime = create_react_refresh_runtime_ident(unresolved_mark)),
  ]
}

pub struct ReactHmrFolder {
  unresolved_mark: Mark,
}

impl Fold for ReactHmrFolder {
  fn fold_program(&mut self, mut program: Program) -> Program {
    let runtime_stmts = create_react_refresh_runtime_stmts(self.unresolved_mark);

    match program {
      Program::Module(ref mut m) => m
        .body
        .extend(runtime_stmts.into_iter().map(ModuleItem::Stmt)),
      Program::Script(ref mut s) => s.body.extend(runtime_stmts),
    };

    program
  }
}
