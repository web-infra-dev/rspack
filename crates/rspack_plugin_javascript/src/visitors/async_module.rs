use std::collections::LinkedList;

use swc_core::common::util::take::Take;
use swc_core::common::DUMMY_SP;
use swc_core::ecma::ast::{
  ArrayLit, ArrayPat, AssignExpr, AssignOp, AwaitExpr, BindingIdent, BlockStmt, CallExpr, Callee,
  CatchClause, CondExpr, Decl, Expr, ExprOrSpread, ExprStmt, FnExpr, Function, Ident, MemberExpr,
  MemberProp, Module, Param, ParenExpr, Pat, PatOrExpr, Script, Stmt, TryStmt, VarDecl,
};
use swc_core::ecma::ast::{VarDeclKind, VarDeclarator};
use swc_core::ecma::atoms::JsWord;
use swc_core::ecma::{
  ast::ModuleItem,
  visit::{as_folder, noop_visit_mut_type, Fold, VisitMut},
};

// AwaitDependenciesInitFragment
pub fn build_await_dependencies<'a>(promises: LinkedList<bool>) -> impl Fold + 'a {
  as_folder(AwaitDependenciesVisitor { promises })
}

// AwaitDependenciesInitFragment
pub fn build_async_module<'a>() -> impl Fold + 'a {
  as_folder(AsyncModuleVisitor)
}

struct AwaitDependenciesVisitor {
  promises: LinkedList<bool>,
}

impl VisitMut for AwaitDependenciesVisitor {
  noop_visit_mut_type!();

  fn visit_mut_module_items(&mut self, items: &mut Vec<ModuleItem>) {
    let mut args = vec![];
    let mut elems = vec![];

    let last_import = items
      .iter()
      .enumerate()
      .skip_while(|(_, item)| !matches!(item, ModuleItem::Stmt(Stmt::Decl(Decl::Var(_)))))
      .take_while(|(_, item)| matches!(item, ModuleItem::Stmt(Stmt::Decl(Decl::Var(_)))))
      .map(|(i, item)| {
        if let Some(is_async) = self.promises.pop_front() && is_async {
          if let ModuleItem::Stmt(Stmt::Decl(Decl::Var(var))) = item {
            // Safety, after swc(variable hoisting) & treeshaking(remove unused import)
            let decl = unsafe { var.decls.get_unchecked(0) };
            let var_name = decl.name.as_ident().expect("should be ok").sym.to_string();
            args.push(make_arg(&var_name));
            elems.push(make_elem(&var_name));
          }
        }
        i
      })
      .last()
      .map(|i| i + 1);
    if let Some(index) = last_import {
      let item = vec![make_var(args), make_stmt(elems)];
      items.splice(index..index, item);
    }
  }
}

struct AsyncModuleVisitor;

impl VisitMut for AsyncModuleVisitor {
  fn visit_mut_module(&mut self, m: &mut Module) {
    let body: Vec<Stmt> = m
      .body
      .take()
      .into_iter()
      .filter_map(|i| {
        // should not have ModuleItem::ModuleDecl after build_module
        if let ModuleItem::Stmt(s) = i {
          Some(s)
        } else {
          None
        }
      })
      .collect();
    m.body = create_async_module_ast(body)
      .into_iter()
      .map(|i| ModuleItem::Stmt(i))
      .collect();
  }

  fn visit_mut_script(&mut self, s: &mut Script) {
    s.body = create_async_module_ast(s.body.take());
  }
}

fn make_arg(arg: &str) -> Option<ExprOrSpread> {
  Some(ExprOrSpread {
    spread: None,
    expr: Box::new(Expr::Ident(Ident {
      span: DUMMY_SP,
      sym: JsWord::from(arg),
      optional: false,
    })),
  })
}

fn make_elem(elem: &str) -> Option<Pat> {
  Some(Pat::Ident(BindingIdent {
    id: Ident {
      span: DUMMY_SP,
      sym: JsWord::from(elem),
      optional: false,
    },
    type_ann: None,
  }))
}

fn make_var(elems: Vec<Option<ExprOrSpread>>) -> ModuleItem {
  let call_expr = CallExpr {
    span: DUMMY_SP,
    callee: Callee::Expr(Box::new(Expr::Ident(Ident {
      span: DUMMY_SP,
      sym: JsWord::from("__webpack_handle_async_dependencies__"),
      optional: false,
    }))),
    args: vec![ExprOrSpread {
      spread: None,
      expr: Box::from(Expr::Array(ArrayLit {
        span: DUMMY_SP,
        elems,
      })),
    }],
    type_args: None,
  };

  let decl = VarDeclarator {
    span: DUMMY_SP,
    name: Pat::Ident(BindingIdent {
      id: Ident {
        span: DUMMY_SP,
        sym: JsWord::from("__webpack_async_dependencies__"),
        optional: false,
      },
      type_ann: None,
    }),
    init: Some(Box::from(call_expr)),
    definite: false,
  };

  ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::from(VarDecl {
    span: DUMMY_SP,
    kind: VarDeclKind::Var,
    declare: false,
    decls: vec![decl],
  }))))
}

fn make_stmt(elems: Vec<Option<Pat>>) -> ModuleItem {
  let assign = AssignExpr {
    span: DUMMY_SP,
    op: AssignOp::Assign,
    left: PatOrExpr::Pat(Box::from(ArrayPat {
      span: DUMMY_SP,
      elems,
      optional: false,
      type_ann: None,
    })),
    right: Box::new(Expr::Cond(CondExpr {
      span: DUMMY_SP,
      test: Box::new(Expr::Member(MemberExpr {
        span: DUMMY_SP,
        obj: Box::new(Expr::Ident(Ident {
          span: DUMMY_SP,
          sym: JsWord::from("__webpack_async_dependencies__"),
          optional: false,
        })),
        prop: MemberProp::Ident(Ident {
          span: DUMMY_SP,
          sym: JsWord::from("then"),
          optional: false,
        }),
      })),
      cons: Box::new(Expr::Call(CallExpr {
        span: DUMMY_SP,
        callee: Callee::Expr(Box::from(Expr::Await(AwaitExpr {
          span: DUMMY_SP,
          arg: Box::new(Expr::Ident(Ident {
            span: DUMMY_SP,
            sym: JsWord::from("__webpack_async_dependencies__"),
            optional: false,
          })),
        }))),
        args: vec![],
        type_args: None,
      })),
      alt: Box::new(Expr::Ident(Ident {
        span: DUMMY_SP,
        sym: JsWord::from("__webpack_async_dependencies__"),
        optional: false,
      })),
    })),
  };

  ModuleItem::Stmt(Stmt::Expr(ExprStmt {
    span: DUMMY_SP,
    expr: Box::new(Expr::Paren(ParenExpr {
      span: DUMMY_SP,
      expr: Box::from(Expr::Assign(assign)),
    })),
  }))
}

// __webpack_require__.a(module, async function(__webpack_handle_async_dependencies__, __webpack_async_result__) { try {
// ${body}
// __webpack_async_result__();
// } catch(e) { __webpack_async_result__(e); } });
fn create_async_module_ast(mut body: Vec<Stmt>) -> Vec<Stmt> {
  body.push(Stmt::Expr(ExprStmt {
    span: DUMMY_SP,
    expr: Box::new(Expr::Call(CallExpr {
      span: DUMMY_SP,
      callee: Callee::Expr(Box::new(Expr::Ident(Ident {
        span: DUMMY_SP,
        sym: JsWord::from("__webpack_async_result__"),
        optional: false,
      }))),
      args: vec![],
      type_args: None,
    })),
  }));
  vec![Stmt::Expr(ExprStmt {
    span: DUMMY_SP,
    expr: Box::new(Expr::Call(CallExpr {
      span: DUMMY_SP,
      callee: Callee::Expr(Box::new(Expr::Member(MemberExpr {
        span: DUMMY_SP,
        obj: Box::new(Expr::Ident(Ident {
          span: DUMMY_SP,
          sym: JsWord::from("__webpack_require__"),
          optional: false,
        })),
        prop: MemberProp::Ident(Ident {
          span: DUMMY_SP,
          sym: JsWord::from("a"),
          optional: false,
        }),
      }))),
      args: vec![
        ExprOrSpread {
          spread: None,
          expr: Box::new(Expr::Ident(Ident {
            span: DUMMY_SP,
            sym: JsWord::from("module"),
            optional: false,
          })),
        },
        ExprOrSpread {
          spread: None,
          expr: Box::new(Expr::Fn(FnExpr {
            ident: None,
            function: Box::new(Function {
              params: vec![
                Param {
                  span: DUMMY_SP,
                  decorators: vec![],
                  pat: Pat::Ident(BindingIdent {
                    id: Ident {
                      span: DUMMY_SP,
                      sym: JsWord::from("__webpack_handle_async_dependencies__"),
                      optional: false,
                    },
                    type_ann: None,
                  }),
                },
                Param {
                  span: DUMMY_SP,
                  decorators: vec![],
                  pat: Pat::Ident(BindingIdent {
                    id: Ident {
                      span: DUMMY_SP,
                      sym: JsWord::from("__webpack_async_result__"),
                      optional: false,
                    },
                    type_ann: None,
                  }),
                },
              ],
              decorators: vec![],
              span: DUMMY_SP,
              body: Some(BlockStmt {
                span: DUMMY_SP,
                stmts: vec![Stmt::Try(Box::new(TryStmt {
                  span: DUMMY_SP,
                  block: BlockStmt {
                    span: DUMMY_SP,
                    stmts: body,
                  },
                  handler: Some(CatchClause {
                    span: DUMMY_SP,
                    param: Some(Pat::Ident(BindingIdent {
                      id: Ident {
                        span: DUMMY_SP,
                        sym: JsWord::from("e"),
                        optional: false,
                      },
                      type_ann: None,
                    })),
                    body: BlockStmt {
                      span: DUMMY_SP,
                      stmts: vec![Stmt::Expr(ExprStmt {
                        span: DUMMY_SP,
                        expr: Box::new(Expr::Call(CallExpr {
                          span: DUMMY_SP,
                          callee: Callee::Expr(Box::new(Expr::Ident(Ident {
                            span: DUMMY_SP,
                            sym: JsWord::from("__webpack_async_result__"),
                            optional: false,
                          }))),
                          args: vec![ExprOrSpread {
                            spread: None,
                            expr: Box::new(Expr::Ident(Ident {
                              span: DUMMY_SP,
                              sym: JsWord::from("e"),
                              optional: false,
                            })),
                          }],
                          type_args: None,
                        })),
                      })],
                    },
                  }),
                  finalizer: None,
                }))],
              }),
              is_generator: false,
              is_async: true,
              type_params: None,
              return_type: None,
            }),
          })),
        },
      ],
      type_args: None,
    })),
  })]
}
