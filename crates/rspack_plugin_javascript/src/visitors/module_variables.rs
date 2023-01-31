use rspack_core::{runtime_globals, Compilation, Module};
use sugar_path::SugarPath;
use swc_core::common::sync::Lrc;
use swc_core::{common::DUMMY_SP, ecma::utils::ExprFactory};
use {
  swc_core::ecma::ast::*,
  swc_core::ecma::transforms::optimization::inline_globals2,
  swc_core::ecma::utils::{quote_ident, quote_str},
  swc_core::ecma::visit::Fold,
};

pub static WEBPACK_HASH: &str = "__webpack_hash__";
pub static WEBPACK_PUBLIC_PATH: &str = "__webpack_public_path__";
pub static DIR_NAME: &str = "__dirname";
pub static WEBPACK_MODULES: &str = "__webpack_modules__";

pub fn module_variables<'a>(
  module: &'a dyn Module,
  compilation: &'a Compilation,
) -> impl Fold + 'a {
  let mut defs = vec![];
  if let Some(normal_module) = module.as_normal_module() {
    let resource_resolved_data = normal_module.resource_resolved_data();
    if let Some(resource_query) = &resource_resolved_data.resource_query {
      defs.push((
        Expr::Ident(quote_ident!("__resourceQuery")),
        Expr::Lit(Lit::Str(quote_str!(resource_query.as_str()))),
      ));
    }

    let dirname = compilation.options.node.dirname.as_str();
    if dirname.eq("mock") || dirname.eq("warn-mock") || dirname.eq("true") {
      defs.push((
        Expr::Ident(quote_ident!(DIR_NAME)),
        Expr::Lit(Lit::Str(quote_str!(match dirname {
          "mock" => "/".to_string(),
          "warn-mock" => "/".to_string(),
          "true" => resource_resolved_data
            .resource_path
            .parent()
            .expect("TODO:")
            .relative(compilation.options.context.as_ref())
            .to_string_lossy()
            .to_string(),
          _ => unreachable!("dirname should be one of mock, warn-mock, true"),
        }))),
      ));
    }
  }
  defs.push((
    Expr::Ident(quote_ident!(WEBPACK_HASH)),
    Expr::Call(CallExpr {
      span: DUMMY_SP,
      callee: Expr::Ident(quote_ident!(runtime_globals::GET_FULL_HASH)).as_callee(),
      args: vec![],
      type_args: None,
    }),
  ));
  defs.push((
    Expr::Ident(quote_ident!(WEBPACK_MODULES)),
    Expr::Ident(quote_ident!(runtime_globals::MODULE_FACTORIES)),
  ));
  defs.push((
    Expr::Ident(quote_ident!(WEBPACK_PUBLIC_PATH)),
    Expr::Ident(quote_ident!(runtime_globals::PUBLIC_PATH)),
  ));

  inline_globals2(
    Default::default(),
    Default::default(),
    Lrc::new(defs),
    Default::default(),
  )
}
