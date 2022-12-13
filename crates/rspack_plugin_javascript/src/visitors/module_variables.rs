use rspack_core::{Compilation, Module};
use {
  swc_core::common::Mark,
  swc_core::ecma::ast::*,
  swc_core::ecma::minifier::globals_defs,
  swc_core::ecma::utils::{quote_ident, quote_str},
  swc_core::ecma::visit::{as_folder, Fold},
};

pub fn module_variables<'a>(
  module: &'a dyn Module,
  unresolved_mark: Mark,
  top_level_mark: Mark,
  _compilation: &'a Compilation,
) -> impl Fold + 'a {
  let mut defs = vec![];
  if let Some(normal_module) = module.as_normal_module() {
    if let Some(resource_query) = &normal_module.resource_resolved_data().resource_query {
      defs.push((
        Box::new(Expr::Ident(quote_ident!("__resourceQuery"))),
        Box::new(Expr::Lit(Lit::Str(quote_str!(resource_query.as_str())))),
      ));
    }
  }
  as_folder(globals_defs(defs, unresolved_mark, top_level_mark))
}
