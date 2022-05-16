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

pub struct InjectReactRefreshEntryFloder {}

impl Fold for InjectReactRefreshEntryFloder {
  fn fold_module(&mut self, mut module: Module) -> Module {
    let mut body = vec![];

    body.push(ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
      span: DUMMY_SP,
      specifiers: vec![],
      src: quote_str!(hmr_entry_path.to_string()),
      type_only: false,
      asserts: None,
    })));
    body.append(&mut module.body);

    Module { body, ..module }
  }
}

pub static hmr_runtime_path: &str = "/@react-refresh.js";

pub static hmr_entry_path: &str = "/react-hmr-entry.js";

pub static hmr_entry: &str = r#"import RefreshRuntime from "/@react-refresh.js";
RefreshRuntime.injectIntoGlobalHook(window);
window.$RefreshReg$ = () => {};
window.$RefreshSig$ = () => (type) => type;"#;

static hmr_header: &str = r#"import RefreshRuntime from "/@react-refresh.js";
var prevRefreshReg;
var prevRefreshSig;
prevRefreshReg = window.$RefreshReg$;
prevRefreshSig = window.$RefreshSig$;
window.$RefreshReg$ = (type, id) => {
  RefreshRuntime.register(type, "__SOURCE__" + "" + id);
};
window.$RefreshSig$ = RefreshRuntime.createSignatureFunctionForTransform;"#;

static hmr_footer: &str = r#"window.$RefreshReg$ = prevRefreshReg;
window.$RefreshSig$ = prevRefreshSig;
__ACCEPT__
if (!window.__vite_plugin_react_timeout) {
  window.__vite_plugin_react_timeout = setTimeout(() => {
    window.__vite_plugin_react_timeout = 0;
    RefreshRuntime.performReactRefresh();
  }, 30);
}"#;

pub struct ReactHmrFolder {
  pub id: String,
}

impl Fold for ReactHmrFolder {
  fn fold_module(&mut self, mut module: Module) -> Module {
    let hmr_header_ast = parse_file(
      hmr_header
        .replace("__SOURCE__", self.id.as_str())
        .to_string(),
      "",
      &rspack_core::Loader::Js,
    );
    let hmr_footer_ast = parse_file(
      hmr_footer
        .replace("__ACCEPT__", "module.hot.accept();")
        .to_string(),
      "",
      &rspack_core::Loader::Js,
    );

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
        exports.performReactRefresh = debounce(exports.performReactRefresh, 16)
        export default exports
        "#
      )
    }
    _ => {
      panic!("Not found react-refresh, please install it.");
    }
  }
}
