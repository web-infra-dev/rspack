use std::collections::HashSet;

use rspack_core::{Compilation, Dependency, DependencyCategory, Module, RuntimeGlobals};
use rustc_hash::FxHashMap as HashMap;
use swc_core::ecma::utils::{member_expr, ExprFactory};
use {
  swc_core::common::{Mark, SyntaxContext, DUMMY_SP},
  swc_core::ecma::ast::*,
  swc_core::ecma::atoms::JsWord,
  swc_core::ecma::visit::{noop_visit_mut_type, Fold, VisitMut, VisitMutWith},
};

use super::{is_import_meta_hot_accept_call, is_module_hot_accept_call};

pub struct RspackModuleFinalizer<'a> {
  pub module: &'a dyn Module,
  pub unresolved_mark: Mark,
  // pub resolved_ids: &'a HashMap<JsWord, ResolvedURI>,
  // pub entry_flag: bool,
  pub compilation: &'a Compilation,
}

impl<'a> Fold for RspackModuleFinalizer<'a> {
  fn fold_program(&mut self, mut n: Program) -> Program {
    let mut module_bindings = HashMap::default();
    // TODO: should use dependency's code generation
    n.visit_mut_with(&mut RspackModuleFormatTransformer::new(
      self.unresolved_mark,
      self.module,
      self.compilation,
      &mut module_bindings,
    ));

    let esm_dependencies = self
      .compilation
      .module_graph
      .module_graph_module_by_identifier(&self.module.identifier())
      .map(|mgm| {
        mgm
          .dependencies
          .iter()
          .filter_map(|id| {
            let dependency = self.compilation.module_graph.dependency_by_id(id);
            let module = self
              .compilation
              .module_graph
              .module_graph_module_by_dependency_id(id);
            if let (Some(dependency), Some(module)) = (dependency, module) {
              if DependencyCategory::Esm.eq(dependency.category()) {
                return Some(module.id(&self.compilation.chunk_graph).to_string());
              }
            }
            None
          })
          .collect::<HashSet<_>>()
      })
      .expect("Failed to get module graph module");

    n.visit_mut_with(&mut HmrApiRewrite {
      module_bindings: &mut module_bindings,
      esm_dependencies: &esm_dependencies,
    });

    n
  }
}

pub struct RspackModuleFormatTransformer<'a> {
  unresolved_ctxt: SyntaxContext,
  module_bindings: &'a mut HashMap<String, (JsWord, SyntaxContext, bool)>,
}

impl<'a> RspackModuleFormatTransformer<'a> {
  pub fn new(
    unresolved_mark: Mark,
    _module: &'a dyn Module,
    _compilation: &'a Compilation,
    module_bindings: &'a mut HashMap<String, (JsWord, SyntaxContext, bool)>,
  ) -> Self {
    Self {
      unresolved_ctxt: SyntaxContext::empty().apply_mark(unresolved_mark),
      module_bindings,
    }
  }
}

impl<'a> VisitMut for RspackModuleFormatTransformer<'a> {
  noop_visit_mut_type!();

  fn visit_mut_ident(&mut self, ident: &mut Ident) {
    if "require".eq(&ident.sym) && ident.span.ctxt == self.unresolved_ctxt {
      ident.sym = RuntimeGlobals::REQUIRE.into();
    }
  }

  fn visit_mut_var_decl(&mut self, var_decl: &mut VarDecl) {
    if let Some(var_declarator) = var_decl.decls.first() {
      if let Pat::Ident(BindingIdent { id: left_ident, .. }) = &var_declarator.name {
        if let Some(box Expr::Call(CallExpr {
          callee: Callee::Expr(box expr),
          args,
          ..
        })) = &var_declarator.init
        {
          // require('./xx')
          if let Expr::Ident(right_ident) = &expr {
            if "require".eq(&right_ident.sym) && right_ident.span.ctxt == self.unresolved_ctxt {
              if let Some(box Expr::Lit(Lit::Str(str))) =
                args.first().map(|first_arg| &first_arg.expr)
              {
                self.module_bindings.insert(
                  str.value.to_string(),
                  (left_ident.sym.clone(), left_ident.span.ctxt, false),
                );
              }
            }
          }
          // __webpack_require__.ir(require('./xx'))
          if let Expr::Member(MemberExpr {
            obj: box Expr::Ident(obj_ident),
            prop: MemberProp::Ident(prop_ident),
            ..
          }) = &expr
          {
            if RuntimeGlobals::REQUIRE.name().eq(&obj_ident.sym)
              && RuntimeGlobals::INTEROP_REQUIRE.name().eq(&prop_ident.sym)
            {
              if let Some(box Expr::Call(CallExpr {
                callee: Callee::Expr(box Expr::Ident(ident)),
                args,
                ..
              })) = args.first().map(|first_arg| &first_arg.expr)
              {
                if "require".eq(&ident.sym) && ident.span.ctxt == self.unresolved_ctxt {
                  if let Some(box Expr::Lit(Lit::Str(str))) =
                    args.first().map(|first_arg| &first_arg.expr)
                  {
                    self.module_bindings.insert(
                      str.value.to_string(),
                      (left_ident.sym.clone(), left_ident.span.ctxt, true),
                    );
                  }
                }
              }
            }
          }
        }
      }
    }

    var_decl.visit_mut_children_with(self);
  }
}

pub struct HmrApiRewrite<'a> {
  module_bindings: &'a HashMap<String, (JsWord, SyntaxContext, bool)>,
  esm_dependencies: &'a HashSet<String>,
}

impl<'a> HmrApiRewrite<'a> {
  fn rewrite_module_hot_accept(&mut self, n: &mut CallExpr) {
    fn create_auto_import(value: Option<&(JsWord, SyntaxContext, bool)>, str: String) -> Stmt {
      if let Some((sym, ctxt, inter_op)) = value {
        let no_inter_op_call_expr = CallExpr {
          span: DUMMY_SP,
          callee: Ident::new(RuntimeGlobals::REQUIRE.into(), DUMMY_SP).as_callee(),
          args: vec![Lit::Str(str.into()).as_arg()],
          type_args: None,
        };
        let call_expr = match *inter_op {
          true => Box::new(Expr::Call(CallExpr {
            span: DUMMY_SP,
            callee: MemberExpr {
              span: DUMMY_SP,
              obj: Box::new(Expr::Ident(Ident::new(
                RuntimeGlobals::REQUIRE.into(),
                DUMMY_SP,
              ))),
              prop: MemberProp::Ident(Ident::new(RuntimeGlobals::INTEROP_REQUIRE.into(), DUMMY_SP)),
            }
            .as_callee(),
            args: vec![no_inter_op_call_expr.as_arg()],
            type_args: None,
          })),
          false => Box::new(Expr::Call(no_inter_op_call_expr)),
        };
        AssignExpr {
          span: DUMMY_SP,
          op: op!("="),
          left: Pat::Ident(BindingIdent {
            id: Ident::new(sym.clone(), DUMMY_SP.with_ctxt(*ctxt)),
            type_ann: None,
          })
          .into(),
          right: call_expr,
        }
        .into_stmt()
      } else {
        CallExpr {
          span: DUMMY_SP,
          callee: Ident::new(RuntimeGlobals::REQUIRE.into(), DUMMY_SP).as_callee(),
          args: vec![Lit::Str(str.into()).as_arg()],
          type_args: None,
        }
        .into_stmt()
      }
    }

    let mut auto_import_stmts = vec![];
    if let Some(first_arg) = n.args.get(0) {
      match first_arg.expr.as_ref() {
        Expr::Lit(Lit::Str(str)) => {
          let value = str.value.to_string();
          // only visit module.hot.accept callback with harmony import
          if !self.esm_dependencies.contains(&value) {
            return;
          }
          auto_import_stmts.push(create_auto_import(self.module_bindings.get(&value), value));
        }
        Expr::Array(ArrayLit { elems, .. }) => {
          elems.iter().for_each(|e| {
            if let Some(ExprOrSpread {
              expr: box Expr::Lit(Lit::Str(str)),
              ..
            }) = e
            {
              {
                let value = str.value.to_string();
                // only visit module.hot.accept callback with harmony import
                if !self.esm_dependencies.contains(&value) {
                  return;
                }
                auto_import_stmts.push(create_auto_import(self.module_bindings.get(&value), value));
              }
            }
          });
        }
        _ => {}
      }
    }

    // exclude self accept
    if !auto_import_stmts.is_empty() {
      match n.args.len() {
        0 => {}
        // module.hot.accept without callback
        1 => n.args.push(
          FnExpr {
            function: Box::new(Function {
              params: vec![],
              decorators: vec![],
              span: DUMMY_SP,
              body: Some(BlockStmt {
                span: DUMMY_SP,
                stmts: auto_import_stmts,
              }),
              is_generator: false,
              is_async: false,
              type_params: None,
              return_type: None,
            }),
            ident: None,
          }
          .as_arg(),
        ),
        // module.hot.accept with callback
        _ => {
          if let Some(ExprOrSpread {
            expr:
              box Expr::Fn(FnExpr {
                function:
                  box Function {
                    body: Some(BlockStmt { stmts, .. }),
                    ..
                  },
                ..
              }),
            ..
          })
          | Some(ExprOrSpread {
            expr:
              box Expr::Arrow(ArrowExpr {
                body: box BlockStmtOrExpr::BlockStmt(BlockStmt { stmts, .. }),
                ..
              }),
            ..
          }) = n.args.get_mut(1)
          {
            auto_import_stmts.extend(std::mem::take(stmts));
            *stmts = auto_import_stmts;
          } else if let Some(ExprOrSpread {
            expr: box Expr::Arrow(ArrowExpr { body, .. }),
            ..
          }) = n.args.get_mut(1)
          {
            if let box BlockStmtOrExpr::Expr(box ref mut expr) = body {
              auto_import_stmts.push(
                std::mem::replace(expr, Expr::Invalid(Invalid { span: DUMMY_SP })).into_stmt(),
              );
              *body = Box::new(BlockStmtOrExpr::BlockStmt(BlockStmt {
                span: DUMMY_SP,
                stmts: auto_import_stmts,
              }));
            }
          }
        }
      }
    }
  }
}

impl<'a> VisitMut for HmrApiRewrite<'a> {
  noop_visit_mut_type!();

  fn visit_mut_call_expr(&mut self, n: &mut CallExpr) {
    if is_module_hot_accept_call(n) {
      self.rewrite_module_hot_accept(n);
    }
    if is_import_meta_hot_accept_call(n) {
      self.rewrite_module_hot_accept(n);
    }
    n.visit_mut_children_with(self);
  }

  fn visit_mut_member_expr(&mut self, n: &mut MemberExpr) {
    if matches!(&*n.obj, Expr::MetaProp(meta) if meta.kind == MetaPropKind::ImportMeta)
      && matches!(&n.prop, MemberProp::Ident(ident) if ident.sym.eq("webpackHot"))
    {
      if let Some(expr) = member_expr!(DUMMY_SP, module.hot).as_member() {
        *n = expr.to_owned();
      }
    }
    n.visit_mut_children_with(self);
  }
}
