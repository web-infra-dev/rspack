pub mod ast_sugar;
pub mod name_helpers;
pub mod side_effect;
use rspack_core::Plugin;
use rspack_plugin_css::plugin::CssSourcePlugin;

use swc_ecma_ast::{ModuleDecl, ModuleItem};

mod statement;
pub use statement::*;
pub mod log;

#[inline]
pub fn is_external_module(source: &str) -> bool {
  source.starts_with("node:")
}

#[inline]
pub fn is_decl_or_stmt(node: &ModuleItem) -> bool {
  matches!(
    node,
    ModuleItem::ModuleDecl(
      ModuleDecl::ExportDecl(_)
        | ModuleDecl::ExportDefaultExpr(_)
        | ModuleDecl::ExportDefaultDecl(_)
    ) | ModuleItem::Stmt(_)
  )
}

pub use rspack_core::get_swc_compiler;

pub fn inject_built_in_plugins(mut plugins: Vec<Box<dyn Plugin>>) -> Vec<Box<dyn Plugin>> {
  let css_plugin: Box<CssSourcePlugin> = std::default::Default::default();
  plugins.push(css_plugin);
  plugins
}
