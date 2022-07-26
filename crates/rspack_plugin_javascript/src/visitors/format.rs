// use crate::{cjs_runtime_helper, Bundle, ModuleGraph, Platform, ResolvedURI};
use ast::*;
use rspack_core::{Compilation, Dependency, ModuleDependency, ModuleGraphModule, ResolveKind};
use swc_atoms::JsWord;
use swc_common::{Mark, DUMMY_SP};
use swc_ecma_transforms::modules::common_js;
use swc_ecma_transforms::modules::common_js::Config as CommonJsConfig;
use swc_ecma_transforms::{fixer, helpers::inject_helpers};
use swc_ecma_utils::{member_expr, quote_ident, quote_str, ExprFactory};
use swc_ecma_visit::{Fold, FoldWith, VisitMut, VisitMutWith};
use tracing::instrument;

use crate::{
  cjs_runtime_helper, get_rspack_register_callee, RSPACK_DYNAMIC_IMPORT, RSPACK_REQUIRE,
};
use {
  swc_atoms,
  swc_common,
  swc_ecma_ast as ast,
  // swc_ecma_utils::{self},
  swc_ecma_visit::{self, noop_visit_mut_type},
};

pub const RS_REQUIRE: &str = "__rspack_require__";
pub struct RspackModuleFinalizer<'a> {
  pub module: &'a ModuleGraphModule,
  pub unresolved_mark: Mark,
  // pub resolved_ids: &'a HashMap<JsWord, ResolvedURI>,
  pub require_ident: Ident,
  pub module_ident: Ident,
  // pub entry_flag: bool,
  pub compilation: &'a Compilation,
}

impl<'a> Fold for RspackModuleFinalizer<'a> {
  fn fold_module(&mut self, module: Module) -> Module {
    let mut cjs_module = module
      .fold_with(&mut common_js(
        self.unresolved_mark,
        CommonJsConfig {
          ignore_dynamic: true,
          ..Default::default()
        },
        None,
      ))
      .fold_with(&mut fixer::fixer(None))
      .fold_with(&mut inject_helpers());

    cjs_module.visit_mut_with(&mut RspackModuleFormatTransformer::new(
      self.unresolved_mark,
      self.module,
      self.compilation,
    ));

    let stmts = cjs_module
      .body
      .into_iter()
      .filter_map(|stmt| stmt.stmt())
      .collect();

    let namespace = &self.compilation.options.output.namespace;

    let module_body = vec![CallExpr {
      span: DUMMY_SP,
      callee: get_rspack_register_callee(namespace),
      args: vec![
        Expr::Array(ArrayLit {
          span: DUMMY_SP,
          elems: vec![Some(ExprOrSpread {
            spread: None,
            expr: Box::new(Expr::Lit(Lit::Str(quote_str!(self.module.id.clone())))),
          })],
        })
        .as_arg(),
        Expr::Object(ObjectLit {
          span: DUMMY_SP,
          props: vec![swc_ecma_ast::PropOrSpread::Prop(Box::new(Prop::KeyValue(
            KeyValueProp {
              key: PropName::Str(quote_str!(self.module.id.clone())),
              value: Box::new(Expr::Fn(FnExpr {
                ident: None,
                function: Function {
                  params: vec![
                    Param {
                      span: DUMMY_SP,
                      decorators: Default::default(),
                      pat: quote_ident!("module").into(),
                    },
                    Param {
                      span: DUMMY_SP,
                      decorators: Default::default(),
                      pat: quote_ident!("exports").into(),
                    },
                    Param {
                      span: DUMMY_SP,
                      decorators: Default::default(),
                      // keep require mark same as swc common_js used
                      pat: quote_ident!(RSPACK_REQUIRE).into(),
                    },
                    Param {
                      span: DUMMY_SP,
                      decorators: Default::default(),
                      // keep require mark same as swc common_js used
                      pat: quote_ident!(RSPACK_DYNAMIC_IMPORT).into(),
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
              })),
            },
          )))],
        })
        .as_arg(),
      ],
      type_args: Default::default(),
    }
    .into_stmt()
    .into()];

    // if self.entry_flag {
    //   let is_hmr_enabled = self.compilation.options.dev_server.hmr;
    //   let callee = if is_hmr_enabled {
    //     cjs_runtime_helper!(require_hot, rs.require)
    //   } else {
    //     cjs_runtime_helper!(require, rs.require)
    //   };

    //   module_body.push(
    //     CallExpr {
    //       span: DUMMY_SP,
    //       callee,
    //       args: vec![Expr::Lit(Lit::Str(quote_str!(self.module.id.clone()))).as_arg()],
    //       type_args: Default::default(),
    //     }
    //     .into_stmt()
    //     .into(),
    //   );
    // }

    Module {
      span: Default::default(),
      body: module_body,
      shebang: None,
    }
  }
}

pub struct RspackModuleFormatTransformer<'a> {
  require_id: Id,
  compilation: &'a Compilation,
  module: &'a ModuleGraphModule,
  // resolved_ids: &'a HashMap<JsWord, ResolvedURI>,
}
impl<'a> RspackModuleFormatTransformer<'a> {
  pub fn new(
    unresolved_mark: Mark,
    module: &'a ModuleGraphModule,
    bundle: &'a Compilation,
  ) -> Self {
    Self {
      require_id: quote_ident!(DUMMY_SP.apply_mark(unresolved_mark), "require").to_id(),
      module,
      compilation: bundle,
    }
  }

  #[instrument(skip_all)]
  fn rewrite_static_import(&mut self, n: &mut CallExpr) -> Option<()> {
    if let Callee::Expr(box Expr::Ident(ident)) = &mut n.callee {
      if self.require_id == ident.to_id() {
        if let ExprOrSpread {
          spread: None,
          expr: box Expr::Lit(Lit::Str(str)),
        } = n.args.first_mut()?
        {
          let require_dep = Dependency {
            importer: Some(self.module.uri.clone()),
            detail: ModuleDependency {
              specifier: str.value.to_string(),
              kind: ResolveKind::Require,
            },
          };
          // FIXME: No need to say this is a ugly workaround
          let import_dep = Dependency {
            importer: Some(self.module.uri.clone()),
            detail: ModuleDependency {
              specifier: str.value.to_string(),
              kind: ResolveKind::Import,
            },
          };
          let mut js_module = self
            .compilation
            .module_graph
            .module_by_dependency(&require_dep);

          if js_module.is_none() {
            js_module = self
              .compilation
              .module_graph
              .module_by_dependency(&import_dep)
          }

          str.value = JsWord::from(js_module?.id.as_str());
          str.raw = Some(JsWord::from(format!("\"{}\"", js_module?.id)));
        };
      }
    }
    Some(())
  }

  #[instrument(skip_all)]
  fn rewrite_dyn_import(&mut self, n: &mut CallExpr) -> Option<()> {
    if let Lit::Str(Str { value: literal, .. }) = n.args.first()?.expr.as_lit()? {
      // If the import module is not exsit in module graph, we need to leave it as it is
      let dep = Dependency {
        importer: Some(self.module.uri.clone()),
        detail: ModuleDependency {
          specifier: literal.to_string(),
          kind: ResolveKind::Require,
        },
      };
      let js_module = self.compilation.module_graph.module_by_dependency(&dep)?;
      let args;
      let js_module_id = js_module.id.as_str();
      if let Some(chunk) = self
        .compilation
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

      // n.callee = if self.compilation.options.chunk_loading.is_jsonp() {
      n.callee = if true {
        cjs_runtime_helper!(jsonp, rs.dynamic_require)
      } else if false {
        // } else if self.compilation.options.platform == Platform::Node {
        cjs_runtime_helper!(dynamic_node, rs.dynamic_require)
      } else {
        cjs_runtime_helper!(dynamic_browser, rs.dynamic_require)
      };
      n.args = args;
    };
    Some(())
  }
}

impl<'a> VisitMut for RspackModuleFormatTransformer<'a> {
  noop_visit_mut_type!();

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
