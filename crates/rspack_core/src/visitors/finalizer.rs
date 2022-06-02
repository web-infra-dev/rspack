use std::collections::HashMap;

use crate::{Bundle, ModuleGraph, ResolvedURI};
use ast::*;
use rspack_swc::{
  swc_atoms, swc_common, swc_ecma_ast as ast, swc_ecma_transforms_base, swc_ecma_transforms_module,
  swc_ecma_utils::{self, private_ident},
  swc_ecma_visit::{self},
};
use swc_atoms::JsWord;
use swc_common::{Mark, DUMMY_SP};
use swc_ecma_transforms_base::helpers::inject_helpers;
use swc_ecma_transforms_module::common_js;
use swc_ecma_transforms_module::common_js::Config;
use swc_ecma_utils::{member_expr, quote_ident, quote_str, ExprFactory};
use swc_ecma_visit::{Fold, FoldWith, VisitMut, VisitMutWith};

pub const RS_DYNAMIC_REQUIRE: &str = "rs.dynamic_require";
pub const RS_REQUIRE: &str = "__rspack_require__";
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
      let args;
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
