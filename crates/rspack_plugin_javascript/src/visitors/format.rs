use once_cell::sync::Lazy;
use regex::Regex;
use rspack_core::{
  runtime_globals, Compilation, Dependency, DependencyCategory, DependencyType, Module,
  ModuleDependency, ModuleGraphModule, ModuleIdentifier,
};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use swc_core::ecma::utils::{member_expr, ExprFactory};
use {
  swc_core::common::{Mark, SyntaxContext, DUMMY_SP},
  swc_core::ecma::ast::{self, *},
  swc_core::ecma::atoms::{Atom, JsWord},
  swc_core::ecma::visit::{noop_visit_mut_type, Fold, VisitMut, VisitMutWith},
};

use super::{
  is_import_meta_hot_accept_call, is_import_meta_hot_decline_call, is_module_hot_accept_call,
  is_module_hot_decline_call,
};

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
    let mut module_bindings = HashMap::default();
    // TODO: should use dependency's code generation
    module.visit_mut_with(&mut RspackModuleFormatTransformer::new(
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
          .filter(|dep| DependencyCategory::Esm.eq(dep.category()))
          .map(|dep| dep.user_request().to_string())
          .collect::<HashSet<_>>()
      })
      .expect("Failed to get module graph module");

    module.visit_mut_with(&mut HmrApiRewrite {
      module: self.module,
      compilation: self.compilation,
      module_bindings: &mut module_bindings,
      esm_dependencies: &esm_dependencies,
    });
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
  module_bindings: &'a mut HashMap<String, (JsWord, SyntaxContext, bool)>,
}

impl<'a> RspackModuleFormatTransformer<'a> {
  pub fn new(
    unresolved_mark: Mark,
    module: &'a dyn Module,
    compilation: &'a Compilation,
    module_bindings: &'a mut HashMap<String, (JsWord, SyntaxContext, bool)>,
  ) -> Self {
    Self {
      unresolved_ctxt: SyntaxContext::empty().apply_mark(unresolved_mark),
      module,
      compilation,
      module_bindings,
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
}

impl<'a> VisitMut for RspackModuleFormatTransformer<'a> {
  noop_visit_mut_type!();

  fn visit_mut_ident(&mut self, ident: &mut Ident) {
    if "require".eq(&ident.sym) && ident.span.ctxt == self.unresolved_ctxt {
      ident.sym = runtime_globals::REQUIRE.into();
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
            if runtime_globals::REQUIRE.eq(&obj_ident.sym)
              && runtime_globals::INTEROP_REQUIRE.eq(&prop_ident.sym)
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
  compilation: &'a Compilation,
  module: &'a dyn Module,
  module_bindings: &'a HashMap<String, (JsWord, SyntaxContext, bool)>,
  esm_dependencies: &'a HashSet<String>,
}

impl<'a> HmrApiRewrite<'a> {
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

  fn rewrite_module_hot_accept(&mut self, n: &mut CallExpr, dependency_type: &DependencyType) {
    let mut module_id_tuple: (String, String) = Default::default();
    if let Some(Lit::Str(str)) = n
      .args
      .get_mut(0)
      .and_then(|first_arg| first_arg.expr.as_mut_lit())
    {
      if let Some(module) =
        self.resolve_module_legacy(&self.module.identifier(), &str.value, dependency_type)
      {
        let origin_value: String = str.value.to_string();
        let module_id = module.id(&self.compilation.chunk_graph);
        str.value = JsWord::from(module_id);
        str.raw = Some(Atom::from(format!("\"{module_id}\"")));
        // only visit module.hot.accpet callback with harmony import
        if !self.esm_dependencies.contains(&origin_value) {
          return;
        }
        module_id_tuple = (module_id.to_string(), str.value.to_string());
      }
    }

    fn create_auto_import_assign(
      value: &(JsWord, SyntaxContext, bool),
      str: String,
    ) -> Box<AssignExpr> {
      let (sym, ctxt, inter_op) = value;
      let no_inter_op_call_expr = CallExpr {
        span: DUMMY_SP,
        callee: Ident::new(runtime_globals::REQUIRE.into(), DUMMY_SP).as_callee(),
        args: vec![Lit::Str(str.into()).as_arg()],
        type_args: None,
      };
      let call_expr = match *inter_op {
        true => Box::new(Expr::Call(CallExpr {
          span: DUMMY_SP,
          callee: MemberExpr {
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
          .as_callee(),
          args: vec![no_inter_op_call_expr.as_arg()],
          type_args: None,
        })),
        false => Box::new(Expr::Call(no_inter_op_call_expr)),
      };
      Box::new(AssignExpr {
        span: DUMMY_SP,
        op: op!("="),
        left: Pat::Ident(BindingIdent {
          id: Ident::new(sym.clone(), DUMMY_SP.with_ctxt(*ctxt)),
          type_ann: None,
        })
        .into(),
        right: call_expr,
      })
    }

    // module.hot.accpet with callback
    if n.args.len() > 1 {
      if let Some(value) = self.module_bindings.get(&module_id_tuple.1) {
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
              body: BlockStmtOrExpr::BlockStmt(BlockStmt { stmts, .. }),
              ..
            }),
          ..
        }) = n.args.get_mut(1)
        {
          stmts.insert(
            0,
            create_auto_import_assign(value, module_id_tuple.0.clone()).into_stmt(),
          );
        } else if let Some(ExprOrSpread {
          expr: box Expr::Arrow(ArrowExpr { body, .. }),
          ..
        }) = n.args.get_mut(1)
        {
          if let BlockStmtOrExpr::Expr(box expr) = body {
            *body = BlockStmtOrExpr::BlockStmt(BlockStmt {
              span: DUMMY_SP,
              stmts: vec![
                create_auto_import_assign(value, module_id_tuple.0.clone()).into_stmt(),
                std::mem::replace(expr, Expr::Invalid(Invalid { span: DUMMY_SP })).into_stmt(),
              ],
            });
          }
        }
      }
    }
    // module.hot.accept without callback
    if n.args.len() == 1 {
      if let Some(value) = self.module_bindings.get(&module_id_tuple.1) {
        n.args.push(
          FnExpr {
            function: Box::new(Function {
              params: vec![],
              decorators: vec![],
              span: DUMMY_SP,
              body: Some(BlockStmt {
                span: DUMMY_SP,
                stmts: vec![create_auto_import_assign(value, module_id_tuple.0.clone()).into_stmt()],
              }),
              is_generator: false,
              is_async: false,
              type_params: None,
              return_type: None,
            }),
            ident: None,
          }
          .as_arg(),
        );
      }
    }
  }

  fn rewrite_module_hot_decline(&mut self, n: &mut CallExpr, dependency_type: &DependencyType) {
    if let Some(Lit::Str(str)) = n
      .args
      .get_mut(0)
      .and_then(|first_arg| first_arg.expr.as_mut_lit())
    {
      if let Some(module) =
        self.resolve_module_legacy(&self.module.identifier(), &str.value, dependency_type)
      {
        let module_id = module.id(&self.compilation.chunk_graph);
        str.value = JsWord::from(module_id);
        str.raw = Some(Atom::from(format!("\"{module_id}\"")));
      }
    }
  }
}

impl<'a> VisitMut for HmrApiRewrite<'a> {
  noop_visit_mut_type!();

  fn visit_mut_call_expr(&mut self, n: &mut CallExpr) {
    if is_module_hot_accept_call(n) {
      self.rewrite_module_hot_accept(n, &DependencyType::ModuleHotAccept);
    }
    if is_module_hot_decline_call(n) {
      self.rewrite_module_hot_decline(n, &DependencyType::ModuleHotDecline);
    }
    if is_import_meta_hot_accept_call(n) {
      self.rewrite_module_hot_accept(n, &DependencyType::ImportMetaHotAccept);
    }
    if is_import_meta_hot_decline_call(n) {
      self.rewrite_module_hot_decline(n, &DependencyType::ImportMetaHotDecline);
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
