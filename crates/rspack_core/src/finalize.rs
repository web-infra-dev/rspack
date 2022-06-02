use std::collections::HashMap;

use crate::{visitors::ClearMark, Bundle, ModuleGraph, ResolvedURI};
use ast::*;
use rspack_swc::{
  swc_atoms, swc_common, swc_ecma_ast as ast, swc_ecma_transforms_base, swc_ecma_transforms_module,
  swc_ecma_utils::{self, private_ident},
  swc_ecma_visit::{self, as_folder},
};
use swc_atoms::JsWord;
use swc_common::{chain, EqIgnoreSpan, Mark, DUMMY_SP};
use swc_ecma_transforms_base::{helpers::inject_helpers, resolver};
use swc_ecma_transforms_module::common_js;
use swc_ecma_transforms_module::common_js::Config;
use swc_ecma_utils::{member_expr, quote_ident, quote_str, ExprFactory};
use swc_ecma_visit::{Fold, FoldWith, VisitMut, VisitMutWith};

pub fn finalize<'a>(
  file_name: String,
  resolved_ids: &'a HashMap<JsWord, ResolvedURI>,
  entry_flag: bool,
  modules: &'a ModuleGraph,
  bundle: &'a Bundle,
) -> impl Fold + 'a {
  let finalize_pass = chain!(
    as_folder(ClearMark),
    resolver(
      bundle.context.unresolved_mark,
      bundle.context.top_level_mark,
      false,
    ),
    RspackModuleFinalizer {
      file_name,
      resolved_ids,
      require_ident: quote_ident!(
        DUMMY_SP.apply_mark(bundle.context.unresolved_mark),
        "__rspack_require__"
      ),
      module_ident: quote_ident!(
        DUMMY_SP.apply_mark(bundle.context.unresolved_mark),
        "module"
      ),
      entry_flag,
      modules,
      bundle,
    },
    as_folder(HmrModuleIdReWriter {
      resolved_ids,
      rewriting: false,
      bundle,
    })
  );
  finalize_pass
}

pub struct RspackModuleFinalizer<'a> {
  pub modules: &'a ModuleGraph,
  pub file_name: String,
  pub resolved_ids: &'a HashMap<JsWord, ResolvedURI>,
  pub require_ident: Ident,
  pub module_ident: Ident,
  pub entry_flag: bool,
  pub bundle: &'a Bundle,
}

impl<'a> Fold for RspackModuleFinalizer<'a> {
  fn fold_module(&mut self, module: Module) -> Module {
    let mut cjs_module = module
      .fold_with(&mut common_js(
        self.bundle.context.unresolved_mark,
        Config {
          ignore_dynamic: true,
          ..Default::default()
        },
        None,
      ))
      .fold_with(&mut swc_ecma_transforms_base::fixer::fixer(None))
      .fold_with(&mut inject_helpers());

    cjs_module.visit_mut_with(&mut RspackModuleFormatTransformer::new(
      self.bundle.context.unresolved_mark,
      self.resolved_ids,
      self.bundle,
    ));

    let stmts = cjs_module
      .body
      .into_iter()
      .filter_map(|stmt| stmt.stmt())
      .collect();

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
  pub bundle: &'a Bundle,
}

pub const RS_DYNAMIC_REQUIRE: &str = "rs.dynamic_require";
pub const RS_REQUIRE: &str = "__rspack_require__";

impl<'a> VisitMut for HmrModuleIdReWriter<'a> {
  fn visit_mut_call_expr(&mut self, call_expr: &mut CallExpr) {
    if let Callee::Expr(expr) = &mut call_expr.callee {
      match &mut **expr {
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
        let uri = &rid.uri;
        let js_module = self
          .bundle
          .module_graph_container
          .module_graph
          .module_by_uri(uri)
          .unwrap();
        str.value = JsWord::from(js_module.id.as_str());
        str.raw = Some(JsWord::from(format!("\"{}\"", js_module.id)));
      }
    }
  }
}

pub struct RspackModuleFormatTransformer<'a> {
  require_id: Id,
  resolved_ids: &'a HashMap<JsWord, ResolvedURI>,
  bundle: &'a Bundle,
}
impl<'a> RspackModuleFormatTransformer<'a> {
  pub fn new(
    unresolved_mark: Mark,
    resolved_ids: &'a HashMap<JsWord, ResolvedURI>,
    bundle: &'a Bundle,
  ) -> Self {
    Self {
      require_id: quote_ident!(DUMMY_SP.apply_mark(unresolved_mark), "require").to_id(),
      resolved_ids,
      bundle,
    }
  }

  fn rewrite_static_import(&mut self, n: &mut CallExpr) -> Option<()> {
    if let Callee::Expr(box Expr::Ident(ident)) = &mut n.callee {
      if self.require_id == ident.to_id() {
        if let ExprOrSpread {
          spread: None,
          expr: box Expr::Lit(Lit::Str(str)),
        } = n.args.first_mut()?
        {
          let rid = self.resolved_ids.get(&str.value)?;
          let uri = &rid.uri;
          let js_module = self
            .bundle
            .module_graph_container
            .module_graph
            .module_by_uri(uri)?;
          str.value = JsWord::from(js_module.id.as_str());
          str.raw = Some(JsWord::from(format!("\"{}\"", js_module.id)));
        };
      }
    }
    Some(())
  }

  fn rewrite_dyn_import(&mut self, n: &mut CallExpr) -> Option<()> {
    if let Lit::Str(Str { value: literal, .. }) = n.args.first()?.expr.as_lit()? {
      let r_uri = self.resolved_ids.get(literal)?;
      // If the import module is not exsit in module graph, we need to leave it as it is
      let js_module = self
        .bundle
        .module_graph_container
        .module_graph
        .module_by_uri(&r_uri.uri)?;
      let mut args;
      let js_module_id = js_module.id.as_str();
      if let Some(chunk) = self
        .bundle
        .chunk_graph
        .chunk_by_split_point_module_uri(&js_module.uri)
      {
        args = vec![
          Lit::Str(js_module_id.into()).as_arg(),
          Lit::Str(chunk.id.as_str().into()).as_arg(),
        ];
        args.push(Lit::Str(js_module_id.into()).as_arg());
        args.push(Lit::Str(chunk.id.as_str().into()).as_arg());
      } else {
        args = vec![Lit::Str(js_module_id.into()).as_arg()];
      }
      n.callee = private_ident!(RS_DYNAMIC_REQUIRE).as_callee();
      n.args = args;
    };
    Some(())
  }
}

impl<'a> VisitMut for RspackModuleFormatTransformer<'a> {
  fn visit_mut_call_expr(&mut self, n: &mut CallExpr) {
    if n.callee.is_import() {
      self.rewrite_dyn_import(n);
    } else {
      self.rewrite_static_import(n);
    }
    n.visit_mut_children_with(self);
  }

  fn visit_mut_ident(&mut self, n: &mut Ident) {
    if n.to_id() == self.require_id {
      n.sym = RS_REQUIRE.into();
    } else {
      // println!("n.to_id() {:?}", n.to_id());
    }
  }
}
