use swc_common::{chain, Mark, DUMMY_SP};
use swc_ecma_ast::*;
use swc_ecma_transforms_module::common_js;
use swc_ecma_transforms_module::util::Config;
use swc_ecma_utils::{member_expr, quote_ident, quote_str, ExprFactory};
use swc_ecma_visit::{Fold, FoldWith};

pub struct HmrModuleFolder {
  pub file_name: String,
  pub top_level_mark: Mark,
}

impl Fold for HmrModuleFolder {
  fn fold_module(&mut self, module: Module) -> Module {
    let cjs_module = module.fold_with(&mut common_js(
      self.top_level_mark.clone(),
      Default::default(),
      None,
    ));

    let mut stmts = vec![];

    for body in cjs_module.body {
      match body {
        ModuleItem::Stmt(stmt) => stmts.push(stmt),
        _ => {}
      }
    }

    Module {
      span: Default::default(),
      body: vec![CallExpr {
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
                  pat: quote_ident!(DUMMY_SP.apply_mark(self.top_level_mark.clone()), "require")
                    .into(),
                },
                Param {
                  span: DUMMY_SP,
                  decorators: Default::default(),
                  pat: quote_ident!(DUMMY_SP.apply_mark(self.top_level_mark.clone()), "module")
                    .into(),
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
      .into()],
      shebang: None,
    }
  }
}
