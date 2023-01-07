use once_cell::sync::Lazy;
use regex::Regex;
use rspack_core::{
  runtime_globals, Compilation, Dependency, DependencyType, Module, ModuleDependency,
  ModuleGraphModule, ModuleIdentifier,
};
use swc_core::ecma::utils::{quote_ident, ExprFactory};
use tracing::instrument;
use {
  swc_core::common::{Mark, SyntaxContext, DUMMY_SP},
  swc_core::ecma::ast::{self, *},
  swc_core::ecma::atoms::{Atom, JsWord},
  swc_core::ecma::visit::{noop_visit_mut_type, Fold, VisitMut, VisitMutWith},
};

use super::{is_module_hot_accept_call, is_module_hot_decline_call};
use crate::utils::{is_dynamic_import_literal_expr, is_require_literal_expr};

pub static SWC_HELPERS_REG: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"@swc/helpers/lib/(\w*)\.js$").expect("TODO:"));

pub struct RspackModuleFinalizer<'a> {
  pub module: &'a dyn Module,
  pub unresolved_mark: Mark,
  // pub resolved_ids: &'a HashMap<JsWord, ResolvedURI>,
  // pub entry_flag: bool,
  pub compilation: &'a Compilation,
}

impl<'a> Fold for RspackModuleFinalizer<'a> {
  fn fold_module(&mut self, mut module: ast::Module) -> ast::Module {
    // TODO: should use dependency's code generation
    module.visit_mut_with(&mut RspackModuleFormatTransformer::new(
      self.unresolved_mark,
      self.module,
      self.compilation,
    ));

    let body = module
      .body
      .into_iter()
      .filter_map(|stmt| stmt.stmt())
      .map(|stmt| stmt.into())
      .collect();

    ast::Module {
      span: Default::default(),
      body,
      shebang: None,
    }
  }
}

pub struct RspackModuleFormatTransformer<'a> {
  compilation: &'a Compilation,
  module: &'a dyn Module,
  unresolved_ctxt: SyntaxContext,
}

impl<'a> RspackModuleFormatTransformer<'a> {
  pub fn new(unresolved_mark: Mark, module: &'a dyn Module, bundle: &'a Compilation) -> Self {
    Self {
      unresolved_ctxt: SyntaxContext::empty().apply_mark(unresolved_mark),
      module,
      compilation: bundle,
    }
  }

  /// Try to get the module_identifier from `src`, `dependency_type`, and `importer`, it's a legacy way and has performance issue, which should be removed.
  /// TODO: remove this in the future
  fn resolve_module_legacy(
    &self,
    module_identifier: &ModuleIdentifier,
    src: &str,
    dependency_type: &DependencyType,
  ) -> Option<&ModuleGraphModule> {
    self
      .compilation
      .module_graph
      .module_graph_module_by_identifier(module_identifier)
      .and_then(|mgm| {
        mgm.dependencies.iter().find_map(|dep| {
          if dep.request() == src && dep.dependency_type() == dependency_type {
            self.compilation.module_graph.module_by_dependency(dep)
          } else {
            None
          }
        })
      })
  }

  fn rewrite_static_import(&mut self, n: &mut CallExpr) -> Option<()> {
    if is_require_literal_expr(n, &self.unresolved_ctxt) {
      if let Callee::Expr(box Expr::Ident(_ident)) = &mut n.callee {
        if let ExprOrSpread {
          spread: None,
          expr: box Expr::Lit(Lit::Str(str)),
        } = n.args.first_mut()?
        {
          // swc will automatically replace @swc/helpers/src/xx.mjs with @swc/helpers/lib/xx.js when it transform code to commonjs
          // so we need replace it to original specifier to find module
          // this is a temporary solution
          let specifier = match SWC_HELPERS_REG.captures(&str.value) {
            Some(cap) => match cap.get(1) {
              Some(cap) => format!(r#"@swc/helpers/src/{}.mjs"#, cap.as_str()),
              None => str.value.to_string(),
            },
            None => str.value.to_string(),
          };
          // let importer_module = self
          //   .compilation
          //   .module_graph
          //   .module_by_identifier(&self.module.uri)
          //   .expect("Module not found");

          let module_identifier = self.module.identifier();

          let cjs_require_module =
            self.resolve_module_legacy(&module_identifier, &specifier, &DependencyType::CjsRequire);

          let esm_import_module =
            self.resolve_module_legacy(&module_identifier, &specifier, &DependencyType::EsmImport);

          let js_module = cjs_require_module.or(esm_import_module);

          let module_id = js_module?.id(&self.compilation.chunk_graph);
          str.value = JsWord::from(module_id);
          str.raw = Some(Atom::from(format!("\"{module_id}\"")));
        };
      }
    }
    Some(())
  }

  #[instrument(skip_all)]
  fn rewrite_dyn_import(&mut self, n: &mut CallExpr) -> Option<()> {
    if is_dynamic_import_literal_expr(n) {
      if let Lit::Str(Str { value: literal, .. }) = n.args.first()?.expr.as_lit()? {
        // If the import module is not exsit in module graph, we need to leave it as it is
        let js_module = self.resolve_module_legacy(
          &self.module.identifier(),
          literal,
          &DependencyType::DynamicImport,
        )?;

        let js_module_id = js_module.id(&self.compilation.chunk_graph);

        let mut chunk_ids = {
          let chunk_group_ukey = self
            .compilation
            .chunk_graph
            .get_module_chunk_group(js_module.module_identifier, &self.compilation.chunk_by_ukey);
          let chunk_group = self.compilation.chunk_group_by_ukey.get(chunk_group_ukey)?;
          chunk_group
            .chunks
            .iter()
            .map(|chunk_ukey| {
              let chunk = self
                .compilation
                .chunk_by_ukey
                .get(chunk_ukey)
                .unwrap_or_else(|| panic!("chunk should exist"));
              chunk.expect_id()
            })
            .collect::<Vec<_>>()
        };
        chunk_ids.sort();

        if chunk_ids.len() == 1 {
          n.callee = MemberExpr {
            span: DUMMY_SP,
            obj: Box::new(Expr::Call(CallExpr {
              span: DUMMY_SP,
              callee: MemberExpr {
                span: DUMMY_SP,
                obj: Box::new(Expr::Call(CallExpr {
                  span: DUMMY_SP,
                  callee: Ident::new(runtime_globals::ENSURE_CHUNK.into(), DUMMY_SP).as_callee(),
                  args: vec![Expr::Lit(Lit::Str(chunk_ids.first()?.to_string().into())).as_arg()],
                  type_args: None,
                })),
                prop: MemberProp::Ident(Ident::new("then".into(), DUMMY_SP)),
              }
              .as_callee(),
              args: vec![CallExpr {
                span: DUMMY_SP,
                callee: MemberExpr {
                  span: DUMMY_SP,
                  obj: Box::new(Expr::Ident(Ident::new(
                    runtime_globals::REQUIRE.into(),
                    DUMMY_SP,
                  ))),
                  prop: MemberProp::Ident(Ident::new("bind".into(), DUMMY_SP)),
                }
                .as_callee(),
                args: vec![
                  Ident::new(runtime_globals::REQUIRE.into(), DUMMY_SP).as_arg(),
                  Lit::Str(js_module_id.into()).as_arg(),
                ],
                type_args: None,
              }
              .as_arg()],
              type_args: None,
            })),
            prop: MemberProp::Ident(Ident::new("then".into(), DUMMY_SP)),
          }
          .as_callee();
          n.args = vec![MemberExpr {
            span: DUMMY_SP,
            obj: Box::new(Expr::Ident(Ident::new(
              runtime_globals::REQUIRE.into(),
              DUMMY_SP,
            ))),
            prop: MemberProp::Ident(Ident::new(
              runtime_globals::INTEROP_REQUIRE.into(),
              DUMMY_SP,
            )),
          }
          .as_arg()];
        } else {
          n.callee = quote_ident!("Promise.all").as_callee();
          n.args = vec![Expr::Array(ArrayLit {
            span: DUMMY_SP,
            elems: chunk_ids
              .iter()
              .map(|chunk_id| {
                Some(
                  Expr::Call(CallExpr {
                    span: DUMMY_SP,
                    callee: MemberExpr {
                      span: DUMMY_SP,
                      obj: Box::new(Expr::Call(CallExpr {
                        span: DUMMY_SP,
                        callee: MemberExpr {
                          span: DUMMY_SP,
                          obj: Box::new(Expr::Call(CallExpr {
                            span: DUMMY_SP,
                            callee: Ident::new(runtime_globals::ENSURE_CHUNK.into(), DUMMY_SP)
                              .as_callee(),
                            args: vec![Expr::Lit(Lit::Str(chunk_id.to_string().into())).as_arg()],
                            type_args: None,
                          })),
                          prop: MemberProp::Ident(Ident::new("then".into(), DUMMY_SP)),
                        }
                        .as_callee(),
                        args: vec![CallExpr {
                          span: DUMMY_SP,
                          callee: MemberExpr {
                            span: DUMMY_SP,
                            obj: Box::new(Expr::Ident(Ident::new(
                              runtime_globals::REQUIRE.into(),
                              DUMMY_SP,
                            ))),
                            prop: MemberProp::Ident(Ident::new("bind".into(), DUMMY_SP)),
                          }
                          .as_callee(),
                          args: vec![
                            Ident::new(runtime_globals::REQUIRE.into(), DUMMY_SP).as_arg(),
                            Lit::Str(js_module_id.into()).as_arg(),
                          ],
                          type_args: None,
                        }
                        .as_arg()],
                        type_args: None,
                      })),
                      prop: MemberProp::Ident(Ident::new("then".into(), DUMMY_SP)),
                    }
                    .as_callee(),
                    args: vec![MemberExpr {
                      span: DUMMY_SP,
                      obj: Box::new(Expr::Ident(Ident::new(
                        runtime_globals::REQUIRE.into(),
                        DUMMY_SP,
                      ))),
                      prop: MemberProp::Ident(Ident::new(
                        runtime_globals::INTEROP_REQUIRE.into(),
                        DUMMY_SP,
                      )),
                    }
                    .as_arg()],
                    type_args: None,
                  })
                  .as_arg(),
                )
              })
              .collect::<Vec<Option<ExprOrSpread>>>(),
          })
          .as_arg()];
        };
      };
    }
    Some(())
  }

  fn rewrite_module_hot_accept_import(&mut self, n: &mut CallExpr) {
    let mut accpet_module_id: String = Default::default();
    if let Some(Lit::Str(str)) = n
      .args
      .get_mut(0)
      .and_then(|first_arg| first_arg.expr.as_mut_lit())
    {
      if let Some(module) = self.resolve_module_legacy(
        &self.module.identifier(),
        &str.value,
        &DependencyType::ModuleHotAccept,
      ) {
        let module_id = module.id(&self.compilation.chunk_graph);
        str.value = JsWord::from(module_id);
        str.raw = Some(Atom::from(format!("\"{module_id}\"")));
        accpet_module_id = module_id.to_string();
      }
    }

    // TODO: add assign expr with module require
    // module.hot.accept without callback
    if !accpet_module_id.is_empty() && n.args.len() == 1 {
      n.args.push(
        FnExpr {
          ident: None,
          function: Box::new(Function {
            span: DUMMY_SP,
            decorators: Default::default(),
            is_async: false,
            is_generator: false,
            params: vec![],
            body: Some(BlockStmt {
              span: DUMMY_SP,
              stmts: vec![CallExpr {
                span: DUMMY_SP,
                callee: Ident::new(runtime_globals::REQUIRE.into(), DUMMY_SP).as_callee(),
                args: vec![Lit::Str(accpet_module_id.into()).as_arg()],
                type_args: None,
              }
              .into_stmt()],
            }),
            type_params: None,
            return_type: None,
          }),
        }
        .as_arg(),
      );
    }
  }

  fn rewrite_module_hot_decline_import(&mut self, n: &mut CallExpr) {
    if let Some(Lit::Str(str)) = n
      .args
      .get_mut(0)
      .and_then(|first_arg| first_arg.expr.as_mut_lit())
    {
      if let Some(module) = self.resolve_module_legacy(
        &self.module.identifier(),
        &str.value,
        &DependencyType::ModuleHotAccept,
      ) {
        let module_id = module.id(&self.compilation.chunk_graph);
        str.value = JsWord::from(module_id);
        str.raw = Some(Atom::from(format!("\"{module_id}\"")));
      }
    }
  }
}

impl<'a> VisitMut for RspackModuleFormatTransformer<'a> {
  noop_visit_mut_type!();

  fn visit_mut_call_expr(&mut self, n: &mut CallExpr) {
    if is_module_hot_accept_call(n) {
      self.rewrite_module_hot_accept_import(n);
    } else if is_module_hot_decline_call(n) {
      self.rewrite_module_hot_decline_import(n);
    } else if n.callee.is_import() {
      // transform "require('react')" into "__rspack_require__('chunks/react.js')"
      self.rewrite_dyn_import(n);
    } else {
      // self.rewrite_static_import(n);
    }
    n.visit_mut_children_with(self);
  }

  fn visit_mut_ident(&mut self, ident: &mut Ident) {
    if "require".eq(&ident.sym) && ident.span.ctxt == self.unresolved_ctxt {
      ident.sym = runtime_globals::REQUIRE.into();
    }
  }
}
