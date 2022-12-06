// use crate::{cjs_runtime_helper, Bundle, ModuleGraph, Platform, ResolvedURI};
use once_cell::sync::Lazy;
use regex::Regex;
use rspack_core::{
  runtime_globals, Compilation, Dependency, Module, ModuleDependency, ResolveKind,
};
use swc_core::ecma::utils::ExprFactory;
use tracing::instrument;

use crate::utils::{is_dynamic_import_literal_expr, is_require_literal_expr};

use super::is_module_hot_accept_call;
use {
  swc_core::common::{Mark, SyntaxContext, DUMMY_SP},
  swc_core::ecma::ast::{self, *},
  swc_core::ecma::atoms::{Atom, JsWord},
  swc_core::ecma::visit::{noop_visit_mut_type, Fold, VisitMut, VisitMutWith},
};

static SWC_HELPERS_REG: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"@swc/helpers/lib/(\w*)\.js$").unwrap());

pub struct RspackModuleFinalizer<'a> {
  pub module: &'a dyn Module,
  pub unresolved_mark: Mark,
  // pub resolved_ids: &'a HashMap<JsWord, ResolvedURI>,
  // pub entry_flag: bool,
  pub compilation: &'a Compilation,
}

impl<'a> Fold for RspackModuleFinalizer<'a> {
  fn fold_module(&mut self, mut module: ast::Module) -> ast::Module {
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

  fn get_rspack_dynamic_import_callee(&self, chunk_ids: Vec<&str>) -> Callee {
    MemberExpr {
      span: DUMMY_SP,
      obj: Box::new(ast::Expr::Call(CallExpr {
        span: DUMMY_SP,
        callee: Ident::new(runtime_globals::ENSURE_CHUNK.into(), DUMMY_SP).as_callee(),
        args: vec![Expr::Array(ArrayLit {
          span: DUMMY_SP,
          elems: chunk_ids
            .iter()
            .map(|chunk_id| Some(Lit::Str(chunk_id.to_string().into()).as_arg()))
            .collect::<Vec<Option<ExprOrSpread>>>(),
        })
        .as_arg()],

        type_args: None,
      })),
      prop: MemberProp::Ident(Ident::new("then".into(), DUMMY_SP)),
    }
    .as_callee()
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

          // FIXME: currently uri equals to specifier, but this will be changed later.
          let require_dep = Dependency {
            parent_module_identifier: Some(self.module.identifier()),
            detail: ModuleDependency {
              specifier: specifier.clone(),
              kind: ResolveKind::Require,
              span: Some(n.span.into()),
            },
          };
          // FIXME: No need to say this is a ugly workaround
          let import_dep = Dependency {
            parent_module_identifier: Some(self.module.identifier()),
            detail: ModuleDependency {
              specifier,
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
      }
    }
    Some(())
  }

  #[instrument(skip_all)]
  fn rewrite_dyn_import(&mut self, n: &mut CallExpr) -> Option<()> {
    if is_dynamic_import_literal_expr(n) {
      if let Lit::Str(Str { value: literal, .. }) = n.args.first()?.expr.as_lit()? {
        // If the import module is not exsit in module graph, we need to leave it as it is
        // FIXME: currently uri equals to specifier, but this will be changed later.
        let dep = Dependency {
          parent_module_identifier: Some(self.module.identifier()),
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
            obj: Box::new(Expr::Ident(Ident::new(
              runtime_globals::REQUIRE.into(),
              DUMMY_SP,
            ))),
            prop: MemberProp::Ident(Ident::new("bind".into(), DUMMY_SP)),
          }
          .as_callee(),
          args: vec![
            Ident::new(runtime_globals::REQUIRE.into(), DUMMY_SP).as_arg(),
            // Ident::new(RSPACK_REQUIRE.into(), DUMMY_SP),
            Lit::Str(js_module_id.into()).as_arg(),
          ],
          type_args: None,
        })
        .as_arg()];

        let mut chunk_ids = {
          let chunk_group_ukey = self.compilation.chunk_graph.get_module_chunk_group(
            &js_module.module_identifier,
            &self.compilation.chunk_by_ukey,
          );
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
              chunk.id.as_str()
            })
            .collect::<Vec<_>>()
        };
        chunk_ids.sort();

        n.callee = self.get_rspack_dynamic_import_callee(chunk_ids);
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

  fn rewrite_module_hot_accept_import(&mut self, n: &mut CallExpr) {
    if let Some(Lit::Str(str)) = n
      .args
      .get_mut(0)
      .and_then(|first_arg| first_arg.expr.as_mut_lit())
    {
      let dep = Dependency {
        parent_module_identifier: Some(self.module.identifier()),
        detail: ModuleDependency {
          specifier: str.value.to_string(),
          kind: ResolveKind::ModuleHotAccept,
          span: Some(n.span.into()),
        },
      };
      if let Some(module) = self.compilation.module_graph.module_by_dependency(&dep) {
        str.value = JsWord::from(module.id.as_str());
        str.raw = Some(Atom::from(format!("\"{}\"", module.id.as_str())));
      }
    }
  }
}

impl<'a> VisitMut for RspackModuleFormatTransformer<'a> {
  noop_visit_mut_type!();

  fn visit_mut_call_expr(&mut self, n: &mut CallExpr) {
    if is_module_hot_accept_call(n) {
      self.rewrite_module_hot_accept_import(n);
    } else if n.callee.is_import() {
      // transform "require('react')" into "__rspack_require__('chunks/react.js')"
      self.rewrite_dyn_import(n);
    } else {
      self.rewrite_static_import(n);
    }
    n.visit_mut_children_with(self);
  }

  fn visit_mut_ident(&mut self, ident: &mut Ident) {
    if "require".eq(&ident.sym) && ident.span.ctxt == self.unresolved_ctxt {
      ident.sym = runtime_globals::REQUIRE.into();
    }
  }
}
