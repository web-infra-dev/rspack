use std::collections::{HashMap, HashSet};

use crate::{
  path::normalize_path, swc_builder::dynamic_import_with_literal, JsModule, ResolvedURI,
};
use ast::*;
use rspack_swc::{
  swc_atoms, swc_common, swc_ecma_ast as ast, swc_ecma_transforms_base, swc_ecma_transforms_module,
  swc_ecma_utils::{self, private_ident},
  swc_ecma_visit,
};
use swc_atoms::JsWord;
use swc_common::{EqIgnoreSpan, Mark, DUMMY_SP};
use swc_ecma_transforms_base::helpers::inject_helpers;
use swc_ecma_transforms_module::common_js;
use swc_ecma_transforms_module::common_js::Config;
use swc_ecma_utils::{member_expr, quote_ident, quote_str, ExprFactory};
use swc_ecma_visit::{Fold, FoldWith, VisitMut, VisitMutWith};

pub fn hmr_module<'a>(
  file_name: String,
  top_level_mark: Mark,
  resolved_ids: &'a HashMap<JsWord, ResolvedURI>,
  entry_flag: bool,
  modules: &'a HashMap<String, JsModule>,
  code_splitting: bool,
) -> HmrModuleFolder<'a> {
  HmrModuleFolder {
    file_name,
    top_level_mark,
    resolved_ids,
    require_ident: quote_ident!(DUMMY_SP.apply_mark(top_level_mark), "require"),
    module_ident: quote_ident!(DUMMY_SP.apply_mark(top_level_mark), "module"),
    entry_flag,
    modules,
    code_splitting,
  }
}

pub struct HmrModuleFolder<'a> {
  pub modules: &'a HashMap<String, JsModule>,
  pub file_name: String,
  pub top_level_mark: Mark,
  pub resolved_ids: &'a HashMap<JsWord, ResolvedURI>,
  pub require_ident: Ident,
  pub module_ident: Ident,
  pub entry_flag: bool,
  pub code_splitting: bool,
}

impl<'a> Fold for HmrModuleFolder<'a> {
  fn fold_module(&mut self, module: Module) -> Module {
    let mut cjs_module = module
      .fold_with(&mut common_js(
        self.top_level_mark,
        Config {
          ignore_dynamic: true,
          ..Default::default()
        },
        None,
      ))
      .fold_with(&mut swc_ecma_transforms_base::fixer::fixer(None))
      .fold_with(&mut inject_helpers());

    cjs_module.visit_mut_with(&mut HmrModuleIdReWriter {
      resolved_ids: self.resolved_ids,
      rewriting: false,
      modules: self.modules,
      code_splitting: self.code_splitting,
    });

    let mut stmts = vec![];

    for body in cjs_module.body {
      if let ModuleItem::Stmt(stmt) = body {
        stmts.push(stmt);
      }
    }

    let mut module_body = vec![CallExpr {
      span: DUMMY_SP,
      callee: member_expr!(DUMMY_SP, rs.define).as_callee(),
      args: vec![
        Expr::Lit(Lit::Str(quote_str!(self.file_name.clone()))).as_arg(),
        FnExpr {
          ident: None,
          function: Function {
            params: vec![
              Param {
                span: DUMMY_SP,
                decorators: Default::default(),
                // keep require mark same as swc common_js used
                pat: self.require_ident.clone().into(),
              },
              Param {
                span: DUMMY_SP,
                decorators: Default::default(),
                pat: self.module_ident.clone().into(),
              },
              Param {
                span: DUMMY_SP,
                decorators: Default::default(),
                pat: quote_ident!("exports").into(),
              },
            ],
            decorators: Default::default(),
            span: DUMMY_SP,
            body: Some(BlockStmt {
              span: DUMMY_SP,
              stmts,
            }),
            is_generator: false,
            is_async: false,
            type_params: Default::default(),
            return_type: Default::default(),
          },
        }
        .as_arg(),
      ],
      type_args: Default::default(),
    }
    .into_stmt()
    .into()];

    if self.entry_flag {
      module_body.push(
        CallExpr {
          span: DUMMY_SP,
          callee: member_expr!(DUMMY_SP, rs.require).as_callee(),
          args: vec![Expr::Lit(Lit::Str(quote_str!(self.file_name.clone()))).as_arg()],
          type_args: Default::default(),
        }
        .into_stmt()
        .into(),
      );
    }

    Module {
      span: Default::default(),
      body: module_body,
      shebang: None,
    }
  }
}

pub struct HmrModuleIdReWriter<'a> {
  pub resolved_ids: &'a HashMap<JsWord, ResolvedURI>,
  pub rewriting: bool,
  pub modules: &'a HashMap<String, JsModule>,
  pub code_splitting: bool,
}

pub const RS_DYNAMIC_REQUIRE: &str = "rs.dynamic_require";
pub const RS_REQUIRE: &str = "require";

impl<'a> VisitMut for HmrModuleIdReWriter<'a> {
  fn visit_mut_call_expr(&mut self, call_expr: &mut CallExpr) {
    if let Some(str) = dynamic_import_with_literal(call_expr) {
      let id = JsWord::from(str.clone());
      let rid = self.resolved_ids.get(&id).unwrap();

      let mut args = vec![];
      if let Some(js_module) = self.modules.get(&rid.uri) {
        let js_module_id = js_module.id.as_str();
        if let Some(chunk_id) = js_module.chunk_ids.iter().next() {
          args.push(Lit::Str(js_module_id.into()).as_arg());
          args.push(Lit::Str(chunk_id.as_str().into()).as_arg());
        } else {
          args.push(Lit::Str(js_module_id.into()).as_arg());
        }
      }

      call_expr.callee = private_ident!(RS_DYNAMIC_REQUIRE).as_callee();
      call_expr.args = args;

      return;
    }

    if let Callee::Expr(expr) = &call_expr.callee {
      match &**expr {
        Expr::Ident(ident) => {
          if "require".eq(&ident.sym) {
            // require(xxx)
            self.rewriting = true;
            call_expr.visit_mut_children_with(self);
            self.rewriting = false;
          } else {
            // some_function(require(xxxx))
            call_expr.visit_mut_children_with(self);
          }
        }
        Expr::Member(member_expr) => {
          if let Expr::Member(expr) = *member_expr!(DUMMY_SP, module.hot.accpet) {
            if expr.eq_ignore_span(member_expr) {
              self.rewriting = true;

              let call_expr_len = call_expr.args.len();
              // exclude last elements of `module.hot.accpet`
              for expr_or_spread in call_expr.args.iter_mut().take(call_expr_len - 1).rev() {
                expr_or_spread.visit_mut_with(self);
              }

              call_expr.visit_mut_children_with(self);
              self.rewriting = false;
            } else {
              call_expr.visit_mut_children_with(self);
            }
          } else {
            call_expr.visit_mut_children_with(self);
          }
        }
        _ => call_expr.visit_mut_children_with(self),
      }
    } else {
      call_expr.visit_mut_children_with(self)
    }
  }
  fn visit_mut_str(&mut self, str: &mut Str) {
    if self.rewriting {
      if let Some(rid) = self.resolved_ids.get(&str.value) {
        let id = &rid.uri;
        let js_module = self.modules.get(id).unwrap();
        str.value = JsWord::from(js_module.id.as_str());
        str.raw = Some(JsWord::from(format!("\"{}\"", js_module.id.to_string())));
      }
    }
  }
}
