use std::fs;
use std::path::Path;

use nodejs_resolver::ResolveResult;
use nodejs_resolver::Resolver;
use rspack_core::parse_file;
use rspack_swc::swc_common::DUMMY_SP;
use rspack_swc::swc_ecma_ast::{
  CallExpr, Callee, Expr, ImportDecl, Module, ModuleDecl, ModuleItem, Program,
};
use rspack_swc::swc_ecma_utils::quote_str;
use rspack_swc::swc_ecma_visit::{Fold, Visit};

pub struct FoundReactRefreshVisitor {
  pub is_refresh_boundary: bool,
}

impl Visit for FoundReactRefreshVisitor {
  fn visit_call_expr(&mut self, call_expr: &CallExpr) {
    if let Callee::Expr(expr) = &call_expr.callee {
      if let Expr::Ident(ident) = &**expr {
        if "$RefreshReg$".eq(&ident.sym) {
          self.is_refresh_boundary = true;
        }
      }
    }
  }
}

pub struct ReactRefreshEntryRuntimeInjector;

impl Fold for ReactRefreshEntryRuntimeInjector {
  fn fold_module(&mut self, mut module: Module) -> Module {
    let mut body = vec![];

    body.push(ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
      span: DUMMY_SP,
      specifiers: vec![],
      src: quote_str!(HMR_ENTRY_PATH.to_string()),
      type_only: false,
      asserts: None,
    })));
    body.append(&mut module.body);

    Module { body, ..module }
  }
}

pub static HMR_RUNTIME_PATH: &str = "/@react-refresh.js";

pub static HMR_ENTRY_PATH: &str = "/react-hmr-entry.js";

pub static HMR_ENTRY: &str = r#"import RefreshRuntime from "/@react-refresh.js";
RefreshRuntime.injectIntoGlobalHook(globalThis);
globalThis.$RefreshReg$ = () => {};
globalThis.$RefreshSig$ = () => (type) => type;"#;

static HMR_HEADER: &str = r#"import RefreshRuntime from "/@react-refresh.js";
var prevRefreshReg;
var prevRefreshSig;
prevRefreshReg = globalThis.$RefreshReg$;
prevRefreshSig = globalThis.$RefreshSig$;
globalThis.$RefreshReg$ = (type, id) => {
  RefreshRuntime.register(type, "__SOURCE__" + "" + id);
};
globalThis.$RefreshSig$ = RefreshRuntime.createSignatureFunctionForTransform;"#;

static HMR_FOOTER: &str = r#"globalThis.$RefreshReg$ = prevRefreshReg;
globalThis.$RefreshSig$ = prevRefreshSig;
module.hot.accept();
RefreshRuntime.queueUpdate();
"#;

pub struct ReactHmrFolder {
  pub id: String,
}

impl Fold for ReactHmrFolder {
  fn fold_module(&mut self, mut module: Module) -> Module {
    let hmr_header_ast = parse_file(
      &HMR_HEADER.replace("__SOURCE__", self.id.as_str()),
      "",
      &rspack_core::Loader::Js,
    );
    let hmr_footer_ast = parse_file(HMR_FOOTER, "", &rspack_core::Loader::Js);

    let mut body = vec![];
    body.append(&mut match hmr_header_ast {
      Program::Module(m) => m.body,
      _ => vec![],
    });
    body.append(&mut module.body);
    body.append(&mut match hmr_footer_ast {
      Program::Module(m) => m.body,
      _ => vec![],
    });

    Module { body, ..module }
  }
}

pub fn load_hmr_runtime_path(root: &String) -> String {
  let resolver = Resolver::default();
  match resolver.resolve(Path::new(&root), "react-refresh/package.json") {
    Ok(ResolveResult::Path(path)) => {
      format!(
        "{}\n{}",
        fs::read_to_string(
          path
            .parent()
            .unwrap()
            .join("cjs/react-refresh-runtime.development.js")
            .to_str()
            .unwrap()
        )
        .unwrap(),
        r#"function debounce(fn, delay) {
          var handle
          return () => {
            clearTimeout(handle)
            handle = setTimeout(fn, delay)
          }
        }
        exports.queueUpdate = debounce(exports.performReactRefresh, 16)
        export default exports
        "#
      )
    }
    _ => {
      panic!("Not found react-refresh, please install it.");
    }
  }
}
