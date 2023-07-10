use std::sync::Arc;

use rspack_core::ReactOptions;
use swc_core::common::DUMMY_SP;
use swc_core::common::{comments::SingleThreadedComments, Mark, SourceMap};
use swc_core::ecma::ast::{
  BinExpr, BinaryOp, BlockStmt, CallExpr, Callee, Expr, ExprOrSpread, ExprStmt, FnDecl, FnExpr,
  Function, Ident, MemberExpr, ModuleItem, Program, Stmt,
};
use swc_core::ecma::transforms::react::RefreshOptions;
use swc_core::ecma::transforms::react::{react as swc_react, Options};
use swc_core::ecma::utils::{member_expr, quote_ident, quote_str};
use swc_core::ecma::visit::{noop_visit_type, Fold, Visit, VisitWith};

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

pub fn fold_react_refresh(unresolved_mark: Mark) -> impl Fold {
  ReactHmrFolder { unresolved_mark }
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

// $ReactRefreshRuntime$ is injected by provide
fn create_react_refresh_runtime_stmts(unresolved_mark: Mark) -> Vec<Stmt> {
  fn create_react_refresh_runtime_ident(unresolved_mark: Mark) -> Box<Expr> {
    Box::new(Expr::Ident(Ident {
      span: DUMMY_SP.apply_mark(unresolved_mark),
      sym: "$ReactRefreshRuntime$".into(),
      optional: false,
    }))
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
          stmts: vec![Stmt::Expr(ExprStmt {
            span: DUMMY_SP,
            expr: CallExpr {
              span: DUMMY_SP,
              callee: Callee::Expr(
                MemberExpr {
                  span: DUMMY_SP,
                  obj: create_react_refresh_runtime_ident(unresolved_mark),
                  prop: quote_ident!("register").into(),
                }
                .into(),
              ),
              args: vec![
                ExprOrSpread {
                  spread: None,
                  expr: quote_ident!("type").into(),
                },
                ExprOrSpread {
                  spread: None,
                  expr: BinExpr {
                    span: DUMMY_SP,
                    op: BinaryOp::Add,
                    left: BinExpr {
                      span: DUMMY_SP,
                      op: BinaryOp::Add,
                      left: member_expr!(DUMMY_SP, __webpack_module__),
                      right: quote_str!("_").into(),
                    }
                    .into(),
                    right: quote_ident!("id").into(),
                  }
                  .into(),
                },
              ],
              type_args: None,
            }
            .into(),
          })],
        }),
        is_generator: false,
        is_async: false,
        type_params: None,
        return_type: None,
      }),
    }
    .into(),
    ExprStmt {
      span: DUMMY_SP,
      expr: CallExpr {
        span: DUMMY_SP,
        callee: Callee::Expr(
          MemberExpr {
            span: DUMMY_SP,
            obj: CallExpr {
              span: DUMMY_SP,
              // See https://github.com/web-infra-dev/rspack/pull/2714 why we have a promise here
              callee: member_expr!(DUMMY_SP, Promise.resolve).into(),
              args: vec![],
              type_args: None,
            }
            .into(),
            prop: quote_ident!("then").into(),
          }
          .into(),
        ),
        args: vec![ExprOrSpread {
          spread: None,
          expr: FnExpr {
            ident: None,
            function: Box::new(Function {
              params: Vec::new(),
              decorators: Vec::new(),
              span: DUMMY_SP,
              body: Some(BlockStmt {
                span: DUMMY_SP,
                stmts: vec![Stmt::Expr(ExprStmt {
                  span: DUMMY_SP,
                  expr: CallExpr {
                    span: DUMMY_SP,
                    callee: Callee::Expr(
                      MemberExpr {
                        span: DUMMY_SP,
                        obj: create_react_refresh_runtime_ident(unresolved_mark),
                        prop: quote_ident!("refresh").into(),
                      }
                      .into(),
                    ),
                    args: vec![
                      ExprOrSpread {
                        spread: None,
                        expr: member_expr!(DUMMY_SP, __webpack_module__.id),
                      },
                      ExprOrSpread {
                        spread: None,
                        expr: member_expr!(DUMMY_SP, __webpack_module__.hot),
                      },
                    ],
                    type_args: None,
                  }
                  .into(),
                })],
              }),
              is_generator: false,
              is_async: false,
              type_params: None,
              return_type: None,
            }),
          }
          .into(),
        }],
        type_args: None,
      }
      .into(),
    }
    .into(),
  ]
}

pub struct ReactHmrFolder {
  unresolved_mark: Mark,
}

impl Fold for ReactHmrFolder {
  fn fold_program(&mut self, mut program: Program) -> Program {
    let mut f = ReactRefreshUsageFinder::default();

    program.visit_with(&mut f);
    if !f.is_founded {
      return program;
    }

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
