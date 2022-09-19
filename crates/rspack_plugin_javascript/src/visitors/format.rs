// use crate::{cjs_runtime_helper, Bundle, ModuleGraph, Platform, ResolvedURI};
use ast::*;
use rspack_core::{Compilation, Dependency, ModuleDependency, ModuleGraphModule, ResolveKind};
use swc_atoms::{Atom, JsWord};
use swc_common::comments::SingleThreadedComments;
use swc_common::{Mark, DUMMY_SP};
use swc_ecma_transforms::hygiene;
use swc_ecma_transforms::modules::common_js;
use swc_ecma_transforms::modules::common_js::Config as CommonJsConfig;
use swc_ecma_transforms::{fixer, helpers::inject_helpers};
use swc_ecma_utils::{quote_ident, ExprFactory};
use swc_ecma_visit::{Fold, FoldWith, VisitMut, VisitMutWith};
use tracing::instrument;

use crate::utils::{is_dynamic_import_literal_expr, is_require_literal_expr};
use crate::{RSPACK_DYNAMIC_IMPORT, RSPACK_REQUIRE};
use {
  swc_atoms,
  swc_common,
  swc_ecma_ast as ast,
  // swc_ecma_utils::{self},
  swc_ecma_visit::{self, noop_visit_mut_type},
};

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
      .fold_with(&mut common_js::<SingleThreadedComments>(
        self.unresolved_mark,
        CommonJsConfig {
          ignore_dynamic: true,
          strict_mode: false, // 'use strict' will add in `wrap_module_function`
          ..Default::default()
        },
        Default::default(),
        None,
      ))
      .fold_with(&mut fixer::fixer(None))
      .fold_with(&mut inject_helpers())
      .fold_with(&mut hygiene());

    cjs_module.visit_mut_with(&mut RspackModuleFormatTransformer::new(
      self.unresolved_mark,
      self.module,
      self.compilation,
    ));

    let stmts = cjs_module
      .body
      .into_iter()
      .filter_map(|stmt| stmt.stmt())
      .map(|stmt| stmt.into())
      .collect();

    /***
     * generate wrapper module such as
     * "function(module, exports, __rspack_require__, __rspack_dynamic_require__) {
     *    module.exports = {}
     * }"
     */
    // let module_body = vec![Expr::Fn(FnExpr {
    //   ident: None,
    //   function: Function {
    //     params: vec![
    //       Param {
    //         span: DUMMY_SP,
    //         decorators: Default::default(),
    //         pat: quote_ident!("module").into(),
    //       },
    //       Param {
    //         span: DUMMY_SP,
    //         decorators: Default::default(),
    //         pat: quote_ident!("exports").into(),
    //       },
    //       Param {
    //         span: DUMMY_SP,
    //         decorators: Default::default(),
    //         // keep require mark same as swc common_js used
    //         pat: quote_ident!(RSPACK_REQUIRE).into(),
    //       },
    //       Param {
    //         span: DUMMY_SP,
    //         decorators: Default::default(),
    //         // keep require mark same as swc common_js used
    //         pat: quote_ident!(RSPACK_DYNAMIC_IMPORT).into(),
    //       },
    //     ],
    //     decorators: Default::default(),
    //     span: DUMMY_SP,
    //     body: Some(BlockStmt {
    //       span: DUMMY_SP,
    //       stmts,
    //     }),
    //     is_generator: false,
    //     is_async: false,
    //     type_params: Default::default(),
    //     return_type: Default::default(),
    //   },
    // })
    // .into_stmt()
    // .into()];

    Module {
      span: Default::default(),
      body: stmts,
      shebang: None,
    }
  }
}

pub struct RspackModuleFormatTransformer<'a> {
  require_id: Id,
  unresolved_mark: Mark,
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
      unresolved_mark,
      module,
      compilation: bundle,
    }
  }

  fn get_rspack_import_callee(&self) -> Callee {
    Ident::new(RSPACK_REQUIRE.into(), DUMMY_SP).as_callee()
  }

  fn get_rspack_dynamic_import_callee(&self, chunk_id: &str) -> Callee {
    MemberExpr {
      span: DUMMY_SP,
      obj: Box::new(swc_ecma_ast::Expr::Call(CallExpr {
        span: DUMMY_SP,
        callee: Ident::new(RSPACK_DYNAMIC_IMPORT.into(), DUMMY_SP).as_callee(),
        args: vec![Lit::Str(chunk_id.into()).as_arg()],
        type_args: None,
      })),
      prop: MemberProp::Ident(Ident::new("then".into(), DUMMY_SP)),
    }
    .as_callee()
  }

  #[instrument(skip_all)]
  fn rewrite_static_import(&mut self, n: &mut CallExpr) -> Option<()> {
    if is_require_literal_expr(n, self.unresolved_mark, &self.require_id) {
      if let Callee::Expr(box Expr::Ident(_ident)) = &mut n.callee {
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
              span: Some(n.span.into()),
            },
          };
          // FIXME: No need to say this is a ugly workaround
          let import_dep = Dependency {
            importer: Some(self.module.uri.clone()),
            detail: ModuleDependency {
              specifier: str.value.to_string(),
              kind: ResolveKind::Import,
              span: Some(n.span.into()),
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
          str.raw = Some(Atom::from(format!("\"{}\"", js_module?.id.as_str())));
        };
        n.callee = self.get_rspack_import_callee();
      }
    }
    Some(())
  }

  #[instrument(skip_all)]
  fn rewrite_dyn_import(&mut self, n: &mut CallExpr) -> Option<()> {
    if is_dynamic_import_literal_expr(n) {
      if let Lit::Str(Str { value: literal, .. }) = n.args.first()?.expr.as_lit()? {
        // If the import module is not exsit in module graph, we need to leave it as it is
        let dep = Dependency {
          importer: Some(self.module.uri.clone()),
          detail: ModuleDependency {
            specifier: literal.to_string(),
            kind: ResolveKind::DynamicImport,
            span: Some(n.span.into()),
          },
        };

        let js_module = self.compilation.module_graph.module_by_dependency(&dep)?;
        let js_module_id = js_module.id.as_str();
        let args = vec![Expr::Call(CallExpr {
          span: DUMMY_SP,
          callee: MemberExpr {
            span: DUMMY_SP,
            obj: Box::new(Expr::Ident(Ident::new(RSPACK_REQUIRE.into(), DUMMY_SP))),
            prop: MemberProp::Ident(Ident::new("bind".into(), DUMMY_SP)),
          }
          .as_callee(),
          args: vec![
            Ident::new(RSPACK_REQUIRE.into(), DUMMY_SP).as_arg(),
            // Ident::new(RSPACK_REQUIRE.into(), DUMMY_SP),
            Lit::Str(js_module_id.into()).as_arg(),
          ],
          type_args: None,
        })
        .as_arg()];

        let chunk_id = if let Some(chunk) = self
          .compilation
          .chunk_graph
          .chunk_by_split_point_module_uri(&js_module.uri, &self.compilation.chunk_by_ukey)
        {
          chunk.id.as_str()
        } else {
          js_module_id
        };

        n.callee = self.get_rspack_dynamic_import_callee(chunk_id);
        // n.callee = if self.compilation.options.chunk_loading.is_jsonp() {
        // n.callee = if true {
        //   cjs_runtime_helper!(jsonp, rs.dynamic_require)
        // } else if false {
        //   // } else if self.compilation.options.platform == Platform::Node {
        //   cjs_runtime_helper!(dynamic_node, rs.dynamic_require)
        // } else {
        //   cjs_runtime_helper!(dynamic_browser, rs.dynamic_require)
        // };
        n.args = args;
      };
    }
    Some(())
  }
}

impl<'a> VisitMut for RspackModuleFormatTransformer<'a> {
  noop_visit_mut_type!();

  fn visit_mut_call_expr(&mut self, n: &mut CallExpr) {
    if n.callee.is_import() {
      // transform "require('react')" into "__rspack_require__('chunks/react.js')"
      self.rewrite_dyn_import(n);
    } else {
      self.rewrite_static_import(n);
    }
    n.visit_mut_children_with(self);
  }
}
